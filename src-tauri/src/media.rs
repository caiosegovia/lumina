use crate::{
    catalog,
    models::{CacheResult, LibraryConfig, ThumbnailAudit},
    process::{self, CancellationToken, ProcessErrorKind, ProcessSpec},
};
use base64::Engine;
use image::ImageReader;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub const THUMBNAIL_VERSION: i64 = 2;
const VIDEO: &[&str] = &["mp4", "mov", "avi", "mkv", "mts", "m2ts", "3gp", "wmv"];
const RAW: &[&str] = &["dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2"];
const INTERNAL_IMAGE: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "tif", "tiff", "bmp"];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationState {
    Valid,
    ValidWithoutPreview,
    UnsupportedFormat,
    Corrupted,
    Unreadable,
    Timeout,
    MissingDependency,
}
impl ValidationState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::ValidWithoutPreview => "valid_without_preview",
            Self::UnsupportedFormat => "unsupported_format",
            Self::Corrupted => "corrupted",
            Self::Unreadable => "unreadable",
            Self::Timeout => "timeout",
            Self::MissingDependency => "missing_dependency",
        }
    }
    pub fn accepted(&self) -> bool {
        matches!(self, Self::Valid | Self::ValidWithoutPreview)
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResult {
    pub state: ValidationState,
    pub tool: String,
    pub details: String,
}

fn process_failure(tool: &str, error: process::ProcessError) -> ValidationResult {
    let state = match error.kind {
        ProcessErrorKind::MissingDependency => ValidationState::MissingDependency,
        ProcessErrorKind::Timeout => ValidationState::Timeout,
        ProcessErrorKind::Canceled => ValidationState::Unreadable,
        _ => ValidationState::Corrupted,
    };
    ValidationResult {
        state,
        tool: tool.into(),
        details: error.message,
    }
}

pub fn validate(path: &Path, extension: &str, cancel: &CancellationToken) -> ValidationResult {
    if fs::metadata(path).map(|m| m.len() == 0).unwrap_or(true) {
        return ValidationResult {
            state: ValidationState::Unreadable,
            tool: "filesystem".into(),
            details: "Arquivo vazio ou ilegível".into(),
        };
    }
    let ext = extension.to_ascii_lowercase();
    if INTERNAL_IMAGE.contains(&ext.as_str()) {
        return match ImageReader::open(path)
            .and_then(|reader| reader.with_guessed_format())
            .map_err(|e| e.to_string())
            .and_then(|reader| reader.decode().map_err(|e| e.to_string()))
        {
            Ok(_) => ValidationResult {
                state: ValidationState::Valid,
                tool: "image".into(),
                details: "Imagem decodificada".into(),
            },
            Err(error) => ValidationResult {
                state: if error.contains("does not support")
                    || error.contains("Memory limit exceeded")
                    || error.contains("Unsupported")
                {
                    ValidationState::UnsupportedFormat
                } else {
                    ValidationState::Corrupted
                },
                tool: "image".into(),
                details: error,
            },
        };
    }
    if VIDEO.contains(&ext.as_str()) {
        let probe = ProcessSpec::new("FFprobe", "ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=codec_name,width,height,duration",
                "-of",
                "json",
                path.to_string_lossy().as_ref(),
            ])
            .timeout(Duration::from_secs(30))
            .logical("FFprobe validation");
        if let Err(error) = process::run(probe, cancel) {
            return process_failure("ffprobe", error);
        }
        let decode = ProcessSpec::new("FFmpeg", "ffmpeg")
            .args([
                "-v",
                "error",
                "-i",
                path.to_string_lossy().as_ref(),
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .timeout(Duration::from_secs(60))
            .logical("FFmpeg frame validation");
        return match process::run(decode, cancel) {
            Ok(_) => ValidationResult {
                state: ValidationState::Valid,
                tool: "ffprobe+ffmpeg".into(),
                details: "Stream e frame decodificados".into(),
            },
            Err(error) => process_failure("ffmpeg", error),
        };
    }
    if RAW.contains(&ext.as_str()) {
        let spec = ProcessSpec::new("ExifTool", "exiftool")
            .args([
                "-validate",
                "-warning",
                "-error",
                "-json",
                path.to_string_lossy().as_ref(),
            ])
            .timeout(Duration::from_secs(30))
            .logical("ExifTool RAW validation");
        return match process::run(spec, cancel) {
            Ok(out) => {
                let body = String::from_utf8_lossy(&out.stdout);
                let state = if body.contains("Error") {
                    ValidationState::Corrupted
                } else {
                    let preview = ProcessSpec::new("ExifTool", "exiftool")
                        .args(["-b", "-PreviewImage", path.to_string_lossy().as_ref()])
                        .timeout(Duration::from_secs(30))
                        .logical("ExifTool RAW preview");
                    match process::run(preview, cancel) {
                        Ok(value) if !value.stdout.is_empty() => ValidationState::Valid,
                        Ok(_) => ValidationState::ValidWithoutPreview,
                        Err(error) if error.kind == ProcessErrorKind::MissingDependency => {
                            ValidationState::MissingDependency
                        }
                        Err(_) => ValidationState::ValidWithoutPreview,
                    }
                };
                ValidationResult {
                    state,
                    tool: "exiftool".into(),
                    details: format!(
                        "{} · {} ms · {}",
                        out.logical_command,
                        out.duration_ms,
                        process::sanitize(&body)
                    ),
                }
            }
            Err(error) => process_failure("exiftool", error),
        };
    }
    if matches!(ext.as_str(), "heic" | "heif") {
        let spec = ProcessSpec::new("FFmpeg", "ffmpeg")
            .args([
                "-v",
                "error",
                "-i",
                path.to_string_lossy().as_ref(),
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .timeout(Duration::from_secs(45))
            .logical("FFmpeg HEIC validation");
        return match process::run(spec, cancel) {
            Ok(_) => ValidationResult {
                state: ValidationState::Valid,
                tool: "ffmpeg".into(),
                details: "Imagem HEIC decodificada".into(),
            },
            Err(error) => process_failure("ffmpeg", error),
        };
    }
    ValidationResult {
        state: ValidationState::UnsupportedFormat,
        tool: "lumina".into(),
        details: format!("Formato .{ext} não suportado"),
    }
}

pub fn thumbnail_path(cache_root: &Path, hash: &str) -> PathBuf {
    cache_root
        .join("thumbnails")
        .join(format!("v{THUMBNAIL_VERSION}"))
        .join(&hash[..2])
        .join(format!("{hash}.jpg"))
}

fn read_orientation(source: &Path, cancel: &CancellationToken) -> u8 {
    process::run(
        ProcessSpec::new("ExifTool", "exiftool")
            .args(["-Orientation#", "-s3", source.to_string_lossy().as_ref()])
            .timeout(Duration::from_secs(10))
            .logical("ExifTool orientation"),
        cancel,
    )
    .ok()
    .and_then(|value| String::from_utf8(value.stdout).ok())
    .and_then(|value| value.trim().parse::<u8>().ok())
    .filter(|value| (1..=8).contains(value))
    .unwrap_or(1)
}

fn apply_orientation(image: image::DynamicImage, orientation: u8) -> image::DynamicImage {
    match orientation {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.rotate90().fliph(),
        6 => image.rotate90(),
        7 => image.rotate270().fliph(),
        8 => image.rotate270(),
        _ => image,
    }
}

pub fn generate_thumbnail(
    source: &Path,
    extension: &str,
    hash: &str,
    cache_root: &Path,
    cancel: &CancellationToken,
) -> Result<PathBuf, String> {
    let destination = thumbnail_path(cache_root, hash);
    if destination.exists() {
        return Ok(destination);
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let temporary = destination.with_extension("job-part.jpg");
    let ext = extension.to_ascii_lowercase();
    if INTERNAL_IMAGE.contains(&ext.as_str()) {
        let mut image = ImageReader::open(source)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;
        image = apply_orientation(image, read_orientation(source, cancel));
        image
            .thumbnail(640, 640)
            .save_with_format(&temporary, image::ImageFormat::Jpeg)
            .map_err(|e| e.to_string())?;
    } else if RAW.contains(&ext.as_str()) {
        let preview = process::run(
            ProcessSpec::new("ExifTool", "exiftool")
                .args(["-b", "-PreviewImage", source.to_string_lossy().as_ref()])
                .timeout(Duration::from_secs(30))
                .logical("ExifTool RAW thumbnail"),
            cancel,
        )
        .map_err(|e| e.message)?;
        if preview.stdout.is_empty() {
            return Err("RAW sem prévia embarcada".into());
        }
        let preview_path = temporary.with_extension("preview.jpg");
        fs::write(&preview_path, &preview.stdout).map_err(|e| e.to_string())?;
        let orientation = read_orientation(source, cancel);
        let result = ImageReader::open(&preview_path)
            .map_err(|e| e.to_string())?
            .with_guessed_format()
            .map_err(|e| e.to_string())?
            .decode()
            .map_err(|e| e.to_string())?;
        let result = apply_orientation(result, orientation)
            .thumbnail(640, 640)
            .save_with_format(&temporary, image::ImageFormat::Jpeg)
            .map_err(|e| e.to_string());
        let _ = fs::remove_file(preview_path);
        result?;
    } else {
        let mut args = vec!["-y".to_string(), "-v".into(), "error".into()];
        if VIDEO.contains(&ext.as_str()) {
            args.extend(["-ss".into(), "1".into()])
        }
        args.extend([
            "-i".into(),
            source.to_string_lossy().into_owned(),
            "-frames:v".into(),
            "1".into(),
            "-vf".into(),
            "scale=640:640:force_original_aspect_ratio=decrease".into(),
            temporary.to_string_lossy().into_owned(),
        ]);
        let refs = args.iter().map(String::as_str);
        process::run(
            ProcessSpec::new("FFmpeg", "ffmpeg")
                .args(refs)
                .timeout(Duration::from_secs(60))
                .logical("FFmpeg thumbnail"),
            cancel,
        )
        .map_err(|e| e.message)?;
    }
    if !temporary.exists() {
        return Err("O gerador não produziu a miniatura".into());
    }
    fs::rename(&temporary, &destination).map_err(|e| e.to_string())?;
    Ok(destination)
}

pub fn thumbnail_data(cfg: &LibraryConfig, asset: &str) -> Result<Option<String>, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let row = conn
        .query_row(
            "SELECT a.master_path,a.extension,a.hash,t.path,t.generator_version,t.state FROM assets a LEFT JOIN thumbnails t ON t.asset_id=a.id WHERE a.id=?1",
            [asset],
            |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<i64>>(4)?,r.get::<_,Option<String>>(5)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    match row {
        Some((master, extension, hash, stored, version, state)) => {
            let allowed_root = Path::new(&cfg.master_path).join(".lumina/cache/thumbnails");
            if let Some(stored_path) = stored
                .as_ref()
                .filter(|p| !p.is_empty())
                .map(Path::new)
                .filter(|p| p.exists())
            {
                let resolved = fs::canonicalize(stored_path).map_err(|e| e.to_string())?;
                let allowed = fs::canonicalize(&allowed_root).map_err(|e| e.to_string())?;
                if !resolved.starts_with(&allowed) {
                    return Err("Caminho de miniatura fora do cache permitido".into());
                }
            }
            let valid = stored
                .as_ref()
                .filter(|_| state.as_deref() == Some("ready") && version == Some(THUMBNAIL_VERSION))
                .map(Path::new)
                .filter(|p| p.exists() && image::image_dimensions(p).is_ok())
                .map(Path::to_path_buf);
            let value = match valid {
                Some(p) => p,
                None => {
                    let generated = generate_thumbnail(
                        Path::new(&master),
                        &extension,
                        &hash,
                        &Path::new(&cfg.master_path).join(".lumina/cache"),
                        &CancellationToken::default(),
                    )?;
                    conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,last_error,updated_at)VALUES(?1,?2,?3,'ready',NULL,datetime('now'))ON CONFLICT(asset_id)DO UPDATE SET generator_version=excluded.generator_version,path=excluded.path,state='ready',last_error=NULL,updated_at=excluded.updated_at",params![asset,THUMBNAIL_VERSION,generated.to_string_lossy()]).map_err(|e|e.to_string())?;
                    generated
                }
            };
            let allowed = Path::new(&cfg.master_path).join(".lumina/cache/thumbnails");
            let resolved = fs::canonicalize(&value).map_err(|e| e.to_string())?;
            let allowed = fs::canonicalize(allowed).map_err(|e| e.to_string())?;
            if !resolved.starts_with(allowed) {
                return Err("Caminho de miniatura fora do cache permitido".into());
            }
            let bytes = fs::read(resolved).map_err(|e| e.to_string())?;
            Ok(Some(format!(
                "data:image/jpeg;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            )))
        }
        None => Ok(None),
    }
}
pub fn clear_cache(cfg: &LibraryConfig) -> Result<i64, String> {
    let cache = Path::new(&cfg.master_path).join(".lumina/cache/thumbnails");
    let lumina =
        fs::canonicalize(Path::new(&cfg.master_path).join(".lumina")).map_err(|e| e.to_string())?;
    if cache.exists() {
        let resolved = fs::canonicalize(&cache).map_err(|e| e.to_string())?;
        if !resolved.starts_with(&lumina) || resolved == lumina {
            return Err("Destino de cache inválido".into());
        }
        fs::remove_dir_all(&resolved).map_err(|e| e.to_string())?
    }
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "UPDATE thumbnails SET state='missing',path='',updated_at=datetime('now')",
            [],
        )
        .map_err(|e| e.to_string())?;
    Ok(changed as i64)
}
pub fn rebuild_cache(cfg: &LibraryConfig) -> Result<CacheResult, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id,master_path,extension,hash FROM assets")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    let cache = Path::new(&cfg.master_path).join(".lumina/cache");
    let mut generated = 0;
    let mut failed = 0;
    for (id, path, ext, hash) in rows {
        match generate_thumbnail(
            Path::new(&path),
            &ext,
            &hash,
            &cache,
            &CancellationToken::default(),
        ) {
            Ok(thumb) => {
                generated += 1;
                conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES(?1,?2,?3,'ready',datetime('now'))ON CONFLICT(asset_id)DO UPDATE SET generator_version=excluded.generator_version,path=excluded.path,state='ready',last_error=NULL,updated_at=excluded.updated_at",params![id,THUMBNAIL_VERSION,thumb.to_string_lossy()]).ok();
            }
            Err(error) => {
                failed += 1;
                conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,last_error,updated_at)VALUES(?1,?2,'','failed',?3,datetime('now'))ON CONFLICT(asset_id)DO UPDATE SET generator_version=excluded.generator_version,path='',state='failed',last_error=excluded.last_error,updated_at=excluded.updated_at",params![id,THUMBNAIL_VERSION,error]).ok();
            }
        }
    }
    Ok(CacheResult { generated, failed })
}

pub fn audit_thumbnails(cfg: &LibraryConfig, repair: bool) -> Result<ThumbnailAudit, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let mut stmt=conn.prepare("SELECT a.id,a.master_path,a.extension,a.hash,t.path,t.generator_version,t.state FROM assets a LEFT JOIN thumbnails t ON t.asset_id=a.id ORDER BY a.id").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    let mut out = ThumbnailAudit {
        total: rows.len() as i64,
        valid: 0,
        missing: 0,
        stale: 0,
        corrupt: 0,
        regenerated: 0,
        failed: 0,
    };
    let cache = Path::new(&cfg.master_path).join(".lumina/cache");
    for (id, master, ext, hash, path, version, state) in rows {
        let status = if path
            .as_ref()
            .is_none_or(|p| p.is_empty() || !Path::new(p).exists())
            || state.as_deref() != Some("ready")
        {
            "missing"
        } else if version != Some(THUMBNAIL_VERSION) {
            "stale"
        } else if image::image_dimensions(path.as_ref().unwrap()).is_err() {
            "corrupt"
        } else {
            "valid"
        };
        match status {
            "valid" => out.valid += 1,
            "missing" => out.missing += 1,
            "stale" => out.stale += 1,
            _ => out.corrupt += 1,
        }
        if repair && status != "valid" {
            match generate_thumbnail(
                Path::new(&master),
                &ext,
                &hash,
                &cache,
                &CancellationToken::default(),
            ) {
                Ok(p) => {
                    out.regenerated += 1;
                    conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,last_error,updated_at)VALUES(?1,?2,?3,'ready',NULL,datetime('now'))ON CONFLICT(asset_id)DO UPDATE SET generator_version=excluded.generator_version,path=excluded.path,state='ready',last_error=NULL,updated_at=excluded.updated_at",params![id,THUMBNAIL_VERSION,p.to_string_lossy()]).map_err(|e|e.to_string())?;
                }
                Err(e) => {
                    out.failed += 1;
                    conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,last_error,updated_at)VALUES(?1,?2,'','failed',?3,datetime('now'))ON CONFLICT(asset_id)DO UPDATE SET state='failed',path='',last_error=excluded.last_error,updated_at=excluded.updated_at",params![id,THUMBNAIL_VERSION,e]).ok();
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use uuid::Uuid;
    #[test]
    fn rejects_fake_image_and_accepts_real_image() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let fake = root.join("fake.jpg");
        fs::write(&fake, b"fake").unwrap();
        assert_eq!(
            validate(&fake, "jpg", &CancellationToken::default()).state,
            ValidationState::Corrupted
        );
        let real = root.join("real.png");
        ImageBuffer::<Rgb<u8>, _>::from_pixel(8, 8, Rgb([1, 2, 3]))
            .save(&real)
            .unwrap();
        assert_eq!(
            validate(&real, "png", &CancellationToken::default()).state,
            ValidationState::Valid
        );
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn thumbnail_is_versioned_and_reused() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let real = root.join("real.png");
        ImageBuffer::<Rgb<u8>, _>::from_pixel(20, 10, Rgb([1, 2, 3]))
            .save(&real)
            .unwrap();
        let hash = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let first =
            generate_thumbnail(&real, "png", hash, &root, &CancellationToken::default()).unwrap();
        let second =
            generate_thumbnail(&real, "png", hash, &root, &CancellationToken::default()).unwrap();
        assert_eq!(first, second);
        assert!(first.to_string_lossy().contains("v2"));
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    #[cfg(windows)]
    fn validates_video_and_generates_real_frame_thumbnail() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let video = root.join("sample.mp4");
        process::run(
            ProcessSpec::new("FFmpeg", "ffmpeg")
                .args([
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=blue:s=160x90:d=2",
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                    video.to_string_lossy().as_ref(),
                ])
                .timeout(Duration::from_secs(30))
                .logical("FFmpeg test video"),
            &CancellationToken::default(),
        )
        .unwrap();
        let validation = validate(&video, "mp4", &CancellationToken::default());
        assert_eq!(
            validation.state,
            ValidationState::Valid,
            "{}",
            validation.details
        );
        let hash = crate::storage::sha256(&video).unwrap();
        let thumbnail = generate_thumbnail(
            &video,
            "mp4",
            &hash,
            &root.join("cache"),
            &CancellationToken::default(),
        )
        .unwrap();
        assert!(image::image_dimensions(thumbnail).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn thumbnail_respects_exif_orientation() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let photo = root.join("oriented.jpg");
        image::RgbImage::from_pixel(100, 50, image::Rgb([20, 40, 60]))
            .save(&photo)
            .unwrap();
        process::run(
            ProcessSpec::new("ExifTool", "exiftool")
                .args([
                    "-overwrite_original",
                    "-Orientation#=6",
                    photo.to_string_lossy().as_ref(),
                ])
                .logical("ExifTool test orientation"),
            &CancellationToken::default(),
        )
        .unwrap();
        let hash = crate::storage::sha256(&photo).unwrap();
        let thumbnail = generate_thumbnail(
            &photo,
            "jpg",
            &hash,
            &root.join("cache"),
            &CancellationToken::default(),
        )
        .unwrap();
        let (width, height) = image::image_dimensions(thumbnail).unwrap();
        assert!(
            height > width,
            "a orientação EXIF 6 deve produzir uma miniatura vertical"
        );
        assert_eq!(height / width, 2);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn every_exif_orientation_has_the_expected_geometry() {
        for orientation in 1..=8 {
            let image = image::DynamicImage::ImageRgb8(image::RgbImage::new(100, 50));
            let transformed = apply_orientation(image, orientation);
            if orientation >= 5 {
                assert_eq!((transformed.width(), transformed.height()), (50, 100));
            } else {
                assert_eq!((transformed.width(), transformed.height()), (100, 50));
            }
        }
    }
    #[test]
    #[cfg(windows)]
    fn raw_thumbnail_respects_orientation_from_the_raw_container() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.dng");
        let raw = root.join("oriented.dng");
        fs::copy(source, &raw).unwrap();
        process::run(
            ProcessSpec::new("ExifTool", "exiftool")
                .args([
                    "-overwrite_original",
                    "-Orientation#=8",
                    raw.to_string_lossy().as_ref(),
                ])
                .logical("ExifTool RAW orientation fixture"),
            &CancellationToken::default(),
        )
        .unwrap();
        assert_eq!(read_orientation(&raw, &CancellationToken::default()), 8);
        let hash = crate::storage::sha256(&raw).unwrap();
        let thumbnail = generate_thumbnail(
            &raw,
            "dng",
            &hash,
            &root.join("cache"),
            &CancellationToken::default(),
        )
        .unwrap();
        let (width, height) = image::image_dimensions(thumbnail).unwrap();
        assert!(
            height > width,
            "RAW EXIF 8 deve produzir miniatura vertical"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(windows)]
    #[ignore = "usa LUMINA_REAL_RAW_FIXTURE para a regressão no acervo local"]
    fn real_raw_fixture_keeps_portrait_geometry() {
        let source = PathBuf::from(std::env::var("LUMINA_REAL_RAW_FIXTURE").unwrap());
        let orientation = read_orientation(&source, &CancellationToken::default());
        assert!(
            (5..=8).contains(&orientation),
            "fixture deve ser um RAW vertical"
        );
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let hash = crate::storage::sha256(&source).unwrap();
        let thumbnail = generate_thumbnail(
            &source,
            source.extension().unwrap().to_string_lossy().as_ref(),
            &hash,
            &root,
            &CancellationToken::default(),
        )
        .unwrap();
        let (width, height) = image::image_dimensions(thumbnail).unwrap();
        assert!(height > width, "RAW vertical produziu {width}x{height}");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[cfg(windows)]
    fn validates_real_heic_and_raw_fixtures() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let cache = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let heic = fixtures.join("sample.heic");
        let raw = fixtures.join("sample.dng");
        let heic_validation = validate(&heic, "heic", &CancellationToken::default());
        assert_eq!(
            heic_validation.state,
            ValidationState::Valid,
            "{}",
            heic_validation.details
        );
        let raw_validation = validate(&raw, "dng", &CancellationToken::default());
        assert!(
            matches!(
                raw_validation.state,
                ValidationState::Valid | ValidationState::ValidWithoutPreview
            ),
            "{}",
            raw_validation.details
        );
        for (path, extension) in [(&heic, "heic"), (&raw, "dng")] {
            let hash = crate::storage::sha256(path).unwrap();
            let thumbnail = generate_thumbnail(
                path,
                extension,
                &hash,
                &cache,
                &CancellationToken::default(),
            )
            .unwrap();
            assert!(image::image_dimensions(thumbnail).is_ok());
        }
        fs::remove_dir_all(cache).unwrap();
    }
    #[test]
    fn internal_thumbnail_reader_rejects_cataloged_path_outside_cache() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(master.join(".lumina/cache/thumbnails")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let outside = root.join("private.jpg");
        fs::write(&outside, b"secret").unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'file',1,?3,?2)",params!["a".repeat(64),chrono::Utc::now().to_rfc3339(),outside.to_string_lossy()]).unwrap();
        conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES('a',1,?1,'ready',?2)",params![outside.to_string_lossy(),chrono::Utc::now().to_rfc3339()]).unwrap();
        drop(conn);
        assert!(thumbnail_data(&cfg, "a")
            .unwrap_err()
            .contains("fora do cache"));
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn audit_repairs_missing_and_corrupt_thumbnails_for_every_asset() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(&master).unwrap();
        fs::create_dir_all(&backup).unwrap();
        fs::create_dir_all(master.join(".lumina")).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        for (index, id) in ["a", "b"].into_iter().enumerate() {
            let image_path = master.join(format!("{id}.png"));
            image::RgbImage::from_pixel(12, 8, image::Rgb([2 + index as u8, 3, 4]))
                .save(&image_path)
                .unwrap();
            let hash = crate::storage::sha256(&image_path).unwrap();
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','png',?4,'file',1,?5,?4)",params![id,hash,format!("{id}.png"),chrono::Utc::now().to_rfc3339(),image_path.to_string_lossy()]).unwrap();
        }
        let corrupt = master.join(".lumina/cache/thumbnails/bad.jpg");
        fs::create_dir_all(corrupt.parent().unwrap()).unwrap();
        fs::write(&corrupt, b"not an image").unwrap();
        conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES('a',?1,?2,'ready',datetime('now'))",params![THUMBNAIL_VERSION,corrupt.to_string_lossy()]).unwrap();
        drop(conn);
        let audit = audit_thumbnails(&cfg, true).unwrap();
        assert_eq!(audit.total, 2);
        assert_eq!(audit.corrupt, 1);
        assert_eq!(audit.missing, 1);
        assert_eq!(audit.regenerated, 2);
        assert_eq!(audit.failed, 0);
        let final_audit = audit_thumbnails(&cfg, false).unwrap();
        assert_eq!(final_audit.valid, 2);
        fs::remove_dir_all(root).unwrap();
    }
}
