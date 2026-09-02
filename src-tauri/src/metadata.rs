use crate::{catalog, formats, models::AssetDetails, process, process::CancellationToken};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::Value;
use std::{path::Path, time::Duration};

fn text(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|item| match item {
            Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
    })
}

fn number(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|key| {
        value.get(*key).and_then(|item| {
            item.as_f64().or_else(|| {
                item.as_str()
                    .and_then(|raw| raw.split_whitespace().next())
                    .and_then(|raw| raw.parse::<f64>().ok())
            })
        })
    })
}

fn integer(value: &Value, keys: &[&str]) -> Option<i64> {
    number(value, keys).map(|value| value.round() as i64)
}

fn exposure(value: &Value) -> Option<String> {
    let raw = value.get("ExposureTime")?;
    if let Some(label) = raw.as_str() {
        return Some(label.to_string());
    }
    let seconds = raw.as_f64()?;
    if seconds > 0.0 && seconds < 1.0 {
        Some(format!("1/{:.0}", 1.0 / seconds))
    } else {
        Some(format!("{seconds:.3} s"))
    }
}

fn read_details(conn: &rusqlite::Connection, asset_id: &str) -> Result<AssetDetails, String> {
    conn.query_row("SELECT a.camera,t.detected_format,t.mime,t.container,t.codec,t.audio_codec,t.frame_rate,t.bitrate,t.pixel_format,t.lens,t.iso,t.aperture,t.exposure,t.focal_length,t.orientation,t.color_profile,t.support_level,t.inventory_state,t.inventory_error,t.enriched_at FROM assets a LEFT JOIN asset_technical_metadata t ON t.asset_id=a.id WHERE a.id=?1",[asset_id],|row|Ok(AssetDetails{
        camera:row.get(0)?,detected_format:row.get(1)?,mime:row.get(2)?,container:row.get(3)?,codec:row.get(4)?,audio_codec:row.get(5)?,frame_rate:row.get(6)?,bitrate:row.get(7)?,pixel_format:row.get(8)?,lens:row.get(9)?,iso:row.get(10)?,aperture:row.get(11)?,exposure:row.get(12)?,focal_length:row.get(13)?,orientation:row.get(14)?,color_profile:row.get(15)?,support_level:row.get(16)?,inventory_state:row.get(17)?,inventory_error:row.get(18)?,enriched_at:row.get(19)?
    })).map_err(|_|"Mídia não encontrada".to_string())
}

pub fn details(cfg: &crate::models::LibraryConfig, asset_id: &str) -> Result<AssetDetails, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let row: Option<(String, String, String)> = conn
        .query_row(
            "SELECT master_path,extension,media_type FROM assets WHERE id=?1",
            [asset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((path, extension, media_type)) = row else {
        return Err("Mídia não encontrada".into());
    };
    let current = read_details(&conn, asset_id)?;
    let capture_missing = current.camera.is_none()
        && current.lens.is_none()
        && current.iso.is_none()
        && current.aperture.is_none()
        && current.exposure.is_none()
        && current.focal_length.is_none();
    if media_type == "video"
        || (!capture_missing && current.inventory_state.as_deref() == Some("complete"))
    {
        return Ok(current);
    }

    let spec = process::ProcessSpec::new("ExifTool", "exiftool")
        .args([
            "-json",
            "-n",
            "-Make",
            "-Model",
            "-LensModel",
            "-LensID",
            "-ISO",
            "-FNumber",
            "-ExposureTime",
            "-FocalLength",
            "-Orientation#",
            "-ColorSpace",
            "-ICCProfileName",
            "-MIMEType",
            "-ImageWidth",
            "-ImageHeight",
            path.as_str(),
        ])
        .timeout(Duration::from_secs(30))
        .logical("On-demand photo metadata");
    let now = Utc::now().to_rfc3339();
    match process::run(spec, &CancellationToken::default()) {
        Ok(output) => {
            let parsed = serde_json::from_slice::<Vec<Value>>(&output.stdout)
                .ok()
                .and_then(|values| values.into_iter().next());
            if let Some(value) = parsed {
                let make = text(&value, &["Make"]);
                let model = text(&value, &["Model"]);
                let camera = match (make, model) {
                    (Some(make), Some(model))
                        if !model.to_lowercase().contains(&make.to_lowercase()) =>
                    {
                        Some(format!("{make} {model}"))
                    }
                    (_, model) => model,
                };
                let lens = text(&value, &["LensModel", "LensID"]);
                let iso = integer(&value, &["ISO"]);
                let aperture = number(&value, &["FNumber"]);
                let exposure = exposure(&value);
                let focal = number(&value, &["FocalLength"]);
                let orientation = integer(&value, &["Orientation"]);
                let color = text(&value, &["ICCProfileName", "ColorSpace"]);
                let mime = text(&value, &["MIMEType"]);
                let width = integer(&value, &["ImageWidth"]);
                let height = integer(&value, &["ImageHeight"]);
                let (detected, matches) = formats::detected_format(Path::new(&path), &extension);
                let descriptor = formats::descriptor(&extension);
                conn.execute("INSERT INTO asset_technical_metadata(asset_id,declared_extension,detected_format,family,mime,lens,iso,aperture,exposure,focal_length,orientation,color_profile,inventory_state,inventory_error,support_level,extension_matches,metadata_supported,thumbnail_supported,preview_supported,enriched_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'complete',NULL,?13,?14,?15,?16,?17,?18)ON CONFLICT(asset_id)DO UPDATE SET detected_format=excluded.detected_format,mime=COALESCE(excluded.mime,asset_technical_metadata.mime),lens=excluded.lens,iso=excluded.iso,aperture=excluded.aperture,exposure=excluded.exposure,focal_length=excluded.focal_length,orientation=excluded.orientation,color_profile=excluded.color_profile,inventory_state='complete',inventory_error=NULL,support_level=excluded.support_level,extension_matches=excluded.extension_matches,metadata_supported=excluded.metadata_supported,thumbnail_supported=excluded.thumbnail_supported,preview_supported=excluded.preview_supported,enriched_at=excluded.enriched_at",params![asset_id,extension,detected,descriptor.family.as_str(),mime,lens,iso,aperture,exposure,focal,orientation,color,descriptor.support.as_str(),matches,descriptor.metadata,descriptor.thumbnail,descriptor.preview,now]).map_err(|error|error.to_string())?;
                conn.execute("UPDATE assets SET camera=COALESCE(?2,camera),width=COALESCE(?3,width),height=COALESCE(?4,height) WHERE id=?1",params![asset_id,camera,width,height]).map_err(|error|error.to_string())?;
            } else {
                conn.execute("UPDATE asset_technical_metadata SET inventory_state='partial',inventory_error='ExifTool retornou dados inválidos',enriched_at=?2 WHERE asset_id=?1",params![asset_id,now]).ok();
            }
        }
        Err(error) => {
            conn.execute("UPDATE asset_technical_metadata SET inventory_state='partial',inventory_error=?2,enriched_at=?3 WHERE asset_id=?1",params![asset_id,process::sanitize(&error.message),now]).ok();
        }
    }
    read_details(&conn, asset_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use uuid::Uuid;
    #[test]
    fn parses_numeric_and_textual_exif_values() {
        let value = serde_json::json!({"ISO":"400","FNumber":2.8,"ExposureTime":0.004,"LensModel":"24-70mm"});
        assert_eq!(integer(&value, &["ISO"]), Some(400));
        assert_eq!(number(&value, &["FNumber"]), Some(2.8));
        assert_eq!(exposure(&value).as_deref(), Some("1/250"));
        assert_eq!(text(&value, &["LensModel"]).as_deref(), Some("24-70mm"));
    }

    #[test]
    fn on_demand_metadata_is_extracted_and_persisted() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let backup = root.join("backup");
        std::fs::create_dir_all(root.join(".lumina")).unwrap();
        std::fs::create_dir_all(&backup).unwrap();
        let source = root.join("capture.jpg");
        ImageBuffer::<Rgb<u8>, _>::from_pixel(32, 20, Rgb([3, 4, 5]))
            .save(&source)
            .unwrap();
        process::run(
            process::ProcessSpec::new("ExifTool", "exiftool")
                .args([
                    "-overwrite_original",
                    "-Make=Lumina",
                    "-Model=Test Camera",
                    "-LensModel=Prime 35mm",
                    "-ISO=400",
                    "-FNumber=2.8",
                    source.to_string_lossy().as_ref(),
                ])
                .timeout(Duration::from_secs(15)),
            &CancellationToken::default(),
        )
        .unwrap();
        let cfg = crate::models::LibraryConfig {
            id: "metadata".into(),
            name: "Metadata".into(),
            master_path: root.to_string_lossy().into_owned(),
            backup_path: backup.to_string_lossy().into_owned(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('capture',?1,'capture.jpg','photo','jpg',?2,'file',1,?3,?2)",params!["f".repeat(64),Utc::now().to_rfc3339(),source.to_string_lossy()]).unwrap();
        drop(conn);
        let first = details(&cfg, "capture").unwrap();
        assert_eq!(first.camera.as_deref(), Some("Lumina Test Camera"));
        assert_eq!(first.lens.as_deref(), Some("Prime 35mm"));
        assert_eq!(first.iso, Some(400));
        assert_eq!(first.aperture, Some(2.8));
        let second = details(&cfg, "capture").unwrap();
        assert_eq!(second.inventory_state.as_deref(), Some("complete"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
