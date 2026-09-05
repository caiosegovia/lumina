use crate::{
    catalog,
    models::{
        DiscoveryGroup, DiscoveryIndexResult, DiscoveryItem, DiscoveryOverview, LibraryConfig,
    },
};
use chrono::{Datelike, NaiveDateTime, Utc};
use image::{GenericImageView, ImageReader};
use rusqlite::params;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

const ALGORITHM_VERSION: i64 = 1;

fn db(cfg: &LibraryConfig) -> Result<rusqlite::Connection, String> {
    catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())
}

struct VisualFeatures {
    hash: u64,
    width: u32,
    height: u32,
    brightness: f64,
    colorfulness: f64,
    sharpness: f64,
    quality: f64,
    labels: String,
}
fn analyze(path: &Path) -> Result<VisualFeatures, String> {
    let image = ImageReader::open(path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let (width, height) = image.dimensions();
    let sample = image
        .resize(256, 256, image::imageops::FilterType::Triangle)
        .to_rgb8();
    let mut brightness = 0.0;
    let mut colorfulness = 0.0;
    let mut sharpness = 0.0;
    let mut edges = 0_u64;
    for (x, y, pixel) in sample.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        brightness += (r as f64 + g as f64 + b as f64) / 3.0;
        colorfulness += (r.max(g).max(b) - r.min(g).min(b)) as f64;
        if x > 0 {
            let left = sample.get_pixel(x - 1, y).0;
            sharpness += ((r as i32 - left[0] as i32).abs()
                + (g as i32 - left[1] as i32).abs()
                + (b as i32 - left[2] as i32).abs()) as f64
                / 3.0;
            edges += 1
        }
    }
    let pixels = (sample.width() as u64 * sample.height() as u64).max(1) as f64;
    brightness /= pixels;
    colorfulness /= pixels;
    sharpness /= edges.max(1) as f64;
    let mut labels = Vec::new();
    if width > height * 4 / 3 {
        labels.push("paisagem")
    } else if height > width * 4 / 3 {
        labels.push("retrato")
    } else {
        labels.push("quadrada")
    };
    if brightness < 75.0 {
        labels.push("escura")
    } else if brightness > 180.0 {
        labels.push("clara")
    };
    if colorfulness > 55.0 {
        labels.push("colorida")
    } else {
        labels.push("tons suaves")
    };
    let warmth = sample
        .pixels()
        .map(|p| p[0] as i64 - p[2] as i64)
        .sum::<i64>() as f64
        / pixels;
    if warmth > 12.0 {
        labels.push("quente")
    } else if warmth < -12.0 {
        labels.push("fria")
    };
    let exposure = (1.0 - ((brightness - 127.5).abs() / 127.5)).clamp(0.0, 1.0);
    let resolution = ((width as f64 * height as f64).ln() / 20.0).clamp(0.0, 1.0);
    let quality =
        (resolution * 0.35 + (sharpness / 45.0).clamp(0.0, 1.0) * 0.45 + exposure * 0.20) * 100.0;
    let gray = image
        .resize_exact(9, 8, image::imageops::FilterType::Triangle)
        .to_luma8();
    let mut value = 0_u64;
    for y in 0..8 {
        for x in 0..8 {
            value <<= 1;
            if gray.get_pixel(x, y)[0] > gray.get_pixel(x + 1, y)[0] {
                value |= 1;
            }
        }
    }
    Ok(VisualFeatures {
        hash: value,
        width,
        height,
        brightness,
        colorfulness,
        sharpness,
        quality,
        labels: labels.join(","),
    })
}

pub fn build_index(cfg: &LibraryConfig) -> Result<DiscoveryIndexResult, String> {
    let mut conn = db(cfg)?;
    let rows = {
        let mut stmt = conn.prepare("SELECT a.id,a.master_path FROM assets a LEFT JOIN asset_visual_fingerprints f ON f.asset_id=a.id AND f.algorithm_version=?1 LEFT JOIN asset_visual_traits t ON t.asset_id=a.id AND t.algorithm_version=?1 WHERE a.media_type IN('photo','raw') AND (f.asset_id IS NULL OR t.asset_id IS NULL) ORDER BY a.captured_at DESC").map_err(|e| e.to_string())?;
        let mapped = stmt
            .query_map([ALGORITHM_VERSION], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    let mut result = DiscoveryIndexResult {
        indexed: 0,
        skipped: 0,
        failed: 0,
    };
    for (id, raw_path) in rows {
        let path = PathBuf::from(raw_path);
        if !path.is_file() {
            result.skipped += 1;
            continue;
        }
        match analyze(&path) {
            Ok(features) => {
                let hash = features.hash;
                let bands = [
                    (hash & 0xffff) as i64,
                    ((hash >> 16) & 0xffff) as i64,
                    ((hash >> 32) & 0xffff) as i64,
                    ((hash >> 48) & 0xffff) as i64,
                ];
                transaction.execute("INSERT INTO asset_visual_fingerprints(asset_id,dhash,band0,band1,band2,band3,algorithm_version,indexed_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(asset_id) DO UPDATE SET dhash=excluded.dhash,band0=excluded.band0,band1=excluded.band1,band2=excluded.band2,band3=excluded.band3,algorithm_version=excluded.algorithm_version,indexed_at=excluded.indexed_at", params![id,hash as i64,bands[0],bands[1],bands[2],bands[3],ALGORITHM_VERSION,Utc::now().to_rfc3339()]).map_err(|e| e.to_string())?;
                transaction.execute("INSERT INTO asset_visual_traits(asset_id,width,height,brightness,colorfulness,sharpness,quality_score,labels,algorithm_version,indexed_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10) ON CONFLICT(asset_id) DO UPDATE SET width=excluded.width,height=excluded.height,brightness=excluded.brightness,colorfulness=excluded.colorfulness,sharpness=excluded.sharpness,quality_score=excluded.quality_score,labels=excluded.labels,algorithm_version=excluded.algorithm_version,indexed_at=excluded.indexed_at",params![id,features.width,features.height,features.brightness,features.colorfulness,features.sharpness,features.quality,features.labels,ALGORITHM_VERSION,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                result.indexed += 1;
            }
            Err(_) => result.failed += 1,
        }
    }
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

fn item(row: &rusqlite::Row<'_>) -> rusqlite::Result<DiscoveryItem> {
    Ok(DiscoveryItem {
        id: row.get(0)?,
        filename: row.get(1)?,
        media_type: row.get(2)?,
        captured_at: row.get(3)?,
        camera: row.get(4)?,
        quality_score: row.get(5)?,
        visual_labels: row
            .get::<_, Option<String>>(6)?
            .unwrap_or_default()
            .split(',')
            .filter(|x| !x.is_empty())
            .map(str::to_string)
            .collect(),
    })
}

fn all_items(conn: &rusqlite::Connection) -> Result<Vec<DiscoveryItem>, String> {
    let mut stmt=conn.prepare("SELECT a.id,a.filename,a.media_type,a.captured_at,a.camera,t.quality_score,t.labels FROM assets a LEFT JOIN asset_visual_traits t ON t.asset_id=a.id AND t.algorithm_version=1 ORDER BY a.captured_at DESC,a.id").map_err(|e|e.to_string())?;
    let mapped = stmt.query_map([], item).map_err(|e| e.to_string())?;
    mapped
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn parse_date(value: &str) -> Option<chrono::NaiveDateTime> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|x| x.naive_local())
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok())
        .or_else(|| {
            NaiveDateTime::parse_from_str(&format!("{value} 00:00:00"), "%Y-%m-%d %H:%M:%S").ok()
        })
}

pub fn overview(cfg: &LibraryConfig) -> Result<DiscoveryOverview, String> {
    let conn = db(cfg)?;
    let indexable = conn
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE media_type IN('photo','raw')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let indexed = conn
        .query_row(
            "SELECT COUNT(*) FROM asset_visual_fingerprints f JOIN asset_visual_traits t ON t.asset_id=f.asset_id AND t.algorithm_version=f.algorithm_version WHERE f.algorithm_version=?1",
            [ALGORITHM_VERSION],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let assets = all_items(&conn)?;
    let by_id: HashMap<String, DiscoveryItem> =
        assets.iter().cloned().map(|x| (x.id.clone(), x)).collect();
    let fingerprints = {
        let mut stmt=conn.prepare("SELECT asset_id,dhash,band0,band1,band2,band3 FROM asset_visual_fingerprints WHERE algorithm_version=?1").map_err(|e|e.to_string())?;
        let mapped = stmt
            .query_map([ALGORITHM_VERSION], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)? as u64,
                    [
                        r.get::<_, i64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, i64>(5)?,
                    ],
                ))
            })
            .map_err(|e| e.to_string())?;
        mapped
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let mut buckets: HashMap<(usize, i64), Vec<usize>> = HashMap::new();
    for (idx, (_, _, bands)) in fingerprints.iter().enumerate() {
        for (band, value) in bands.iter().enumerate() {
            buckets.entry((band, *value)).or_default().push(idx)
        }
    }
    let mut similar = Vec::new();
    let mut emitted = HashSet::new();
    for (a, (_, hash, bands)) in fingerprints.iter().enumerate() {
        let mut candidates = HashSet::new();
        for (band, value) in bands.iter().enumerate() {
            if let Some(indexes) = buckets.get(&(band, *value)) {
                for candidate in indexes.iter().rev().take(64) {
                    if *candidate != a {
                        candidates.insert(*candidate);
                    }
                }
            }
        }
        let nearest = candidates
            .into_iter()
            .map(|b| (b, (*hash ^ fingerprints[b].1).count_ones()))
            .filter(|(_, distance)| *distance <= 10)
            .min_by_key(|(_, distance)| *distance);
        if let Some((b, distance)) = nearest {
            let pair = (a.min(b), a.max(b));
            if !emitted.insert(pair) {
                continue;
            }
            if let (Some(left), Some(right)) =
                (by_id.get(&fingerprints[a].0), by_id.get(&fingerprints[b].0))
            {
                similar.push(DiscoveryGroup {
                    id: format!("similar-{}-{}", left.id, right.id),
                    title: "Possível variação".into(),
                    detail: format!(
                        "{}% de proximidade visual",
                        100 - (distance as i64 * 100 / 64)
                    ),
                    score: 1.0 - distance as f64 / 64.0,
                    items: vec![left.clone(), right.clone()],
                    recommended_id: [left, right]
                        .into_iter()
                        .max_by(|a, b| {
                            a.quality_score
                                .unwrap_or(0.0)
                                .total_cmp(&b.quality_score.unwrap_or(0.0))
                        })
                        .map(|item| item.id.clone()),
                    recommendation: Some(
                        "Melhor equilíbrio estimado entre detalhe, resolução e luminosidade".into(),
                    ),
                })
            }
        }
    }
    similar.sort_by(|a, b| b.score.total_cmp(&a.score));
    similar.truncate(80);
    let mut sequences = Vec::new();
    let mut current: Vec<DiscoveryItem> = Vec::new();
    let mut chronological = assets.clone();
    chronological.sort_by_key(|x| parse_date(&x.captured_at));
    for asset in chronological {
        let joins = current
            .last()
            .and_then(|last| {
                Some(
                    (parse_date(&asset.captured_at)? - parse_date(&last.captured_at)?)
                        .num_seconds()
                        .abs()
                        <= 15
                        && asset.camera == last.camera,
                )
            })
            .unwrap_or(false);
        if !joins && current.len() >= 3 {
            let taken = std::mem::take(&mut current);
            sequences.push(DiscoveryGroup {
                id: format!("sequence-{}", taken[0].id),
                title: format!("Sequência de {} registros", taken.len()),
                detail: taken[0]
                    .camera
                    .clone()
                    .unwrap_or_else(|| "Mesmo momento".into()),
                score: taken.len() as f64,
                items: taken,
                recommended_id: None,
                recommendation: None,
            });
            if let Some(group) = sequences.last_mut() {
                group.recommended_id = group
                    .items
                    .iter()
                    .max_by(|a, b| {
                        a.quality_score
                            .unwrap_or(0.0)
                            .total_cmp(&b.quality_score.unwrap_or(0.0))
                    })
                    .map(|item| item.id.clone());
                group.recommendation =
                    Some("Sugestão técnica; sua escolha continua soberana".into())
            }
        } else if !joins {
            current.clear();
        }
        current.push(asset);
    }
    if current.len() >= 3 {
        sequences.push(DiscoveryGroup {
            id: format!("sequence-{}", current[0].id),
            title: format!("Sequência de {} registros", current.len()),
            detail: current[0]
                .camera
                .clone()
                .unwrap_or_else(|| "Mesmo momento".into()),
            score: current.len() as f64,
            items: current,
            recommended_id: None,
            recommendation: None,
        });
        if let Some(group) = sequences.last_mut() {
            group.recommended_id = group
                .items
                .iter()
                .max_by(|a, b| {
                    a.quality_score
                        .unwrap_or(0.0)
                        .total_cmp(&b.quality_score.unwrap_or(0.0))
                })
                .map(|item| item.id.clone());
            group.recommendation = Some("Sugestão técnica; sua escolha continua soberana".into())
        }
    }
    sequences.sort_by(|a, b| b.score.total_cmp(&a.score));
    sequences.truncate(40);
    let now = Utc::now();
    let mut memory_map: HashMap<String, Vec<DiscoveryItem>> = HashMap::new();
    for asset in &assets {
        if let Some(date) = parse_date(&asset.captured_at) {
            if date.year() < now.year() && date.month() == now.month() {
                memory_map
                    .entry(date.format("%Y-%m").to_string())
                    .or_default()
                    .push(asset.clone());
            }
        }
    }
    let mut memories = memory_map
        .into_iter()
        .filter(|(_, items)| !items.is_empty())
        .map(|(key, mut items)| {
            items.truncate(12);
            let year = key.get(0..4).unwrap_or("");
            DiscoveryGroup {
                id: format!("memory-{key}"),
                title: format!("Memórias de {year}"),
                detail: format!("{} registros deste período", items.len()),
                score: items.len() as f64,
                items,
                recommended_id: None,
                recommendation: None,
            }
        })
        .collect::<Vec<_>>();
    memories.sort_by(|a, b| b.id.cmp(&a.id));
    memories.truncate(12);
    Ok(DiscoveryOverview {
        indexed,
        indexable,
        similar,
        sequences,
        memories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn perceptual_distance_is_stable() {
        assert_eq!((0b1010_u64 ^ 0b1110_u64).count_ones(), 1);
    }

    #[test]
    fn date_parser_accepts_catalog_formats() {
        assert!(parse_date("2025-01-02T03:04:05+00:00").is_some());
        assert!(parse_date("2025-01-02").is_some());
    }

    #[test]
    fn local_index_finds_a_visual_pair_without_touching_sources() {
        let root = std::env::temp_dir().join(format!("lumina-discovery-{}", Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        std::fs::create_dir_all(&master).unwrap();
        std::fs::create_dir_all(&backup).unwrap();
        let first = master.join("first.png");
        let second = master.join("second.png");
        for (path, shift) in [(&first, 0_u8), (&second, 1_u8)] {
            let mut image = image::GrayImage::new(90, 80);
            for (x, y, pixel) in image.enumerate_pixels_mut() {
                *pixel = image::Luma([((x * 2 + y + shift as u32) % 255) as u8]);
            }
            image.save(path).unwrap();
        }
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "Teste".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = db(&cfg).unwrap();
        for (id, name, path, date) in [
            ("a", "first.png", &first, "2024-09-05"),
            ("b", "second.png", &second, "2024-09-05"),
        ] {
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','png',?4,'metadata',100,?5,?4)",params![id,id.repeat(64),name,date,path.to_string_lossy()]).unwrap();
        }
        drop(conn);
        let indexed = build_index(&cfg).unwrap();
        assert_eq!(indexed.indexed, 2);
        let found = overview(&cfg).unwrap();
        assert_eq!(found.indexed, 2);
        assert_eq!(found.similar.len(), 1);
        assert!(first.exists() && second.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
