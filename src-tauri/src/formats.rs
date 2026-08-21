use serde::Serialize;
use std::{fs::File, io::Read, path::Path};

pub const PHOTO_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "png", "gif", "bmp", "dib", "tif", "tiff", "webp", "heic", "heif",
    "avif", "jp2", "j2k", "jpf", "jpx", "jxl",
];
pub const RAW_EXTENSIONS: &[&str] = &[
    "dng", "cr2", "cr3", "crw", "nef", "nrw", "arw", "srf", "sr2", "raf", "rw2", "rwl", "orf",
    "pef", "srw", "3fr", "fff", "iiq", "mef", "mrw", "dcr", "kdc", "erf", "x3f", "gpr", "mos",
];
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mov", "qt", "avi", "mkv", "webm", "mpeg", "mpg", "mpe", "vob", "ts", "mts",
    "m2ts", "3gp", "3g2", "wmv", "asf", "flv", "f4v", "ogv", "mxf", "dv", "mod", "tod",
];
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "jpe", "png", "gif", "bmp", "dib", "tif", "tiff", "webp", "heic", "heif",
    "avif", "jp2", "j2k", "jpf", "jpx", "jxl", "dng", "cr2", "cr3", "crw", "nef", "nrw", "arw",
    "srf", "sr2", "raf", "rw2", "rwl", "orf", "pef", "srw", "3fr", "fff", "iiq", "mef", "mrw",
    "dcr", "kdc", "erf", "x3f", "gpr", "mos", "mp4", "m4v", "mov", "qt", "avi", "mkv", "webm",
    "mpeg", "mpg", "mpe", "vob", "ts", "mts", "m2ts", "3gp", "3g2", "wmv", "asf", "flv", "f4v",
    "ogv", "mxf", "dv", "mod", "tod",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFamily {
    Photo,
    Raw,
    Video,
    Unknown,
}
impl MediaFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Photo => "photo",
            Self::Raw => "raw",
            Self::Video => "video",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    Complete,
    Partial,
    Preservation,
    Unknown,
    Invalid,
}
impl SupportLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Preservation => "preservation",
            Self::Unknown => "unknown",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDescriptor {
    pub key: &'static str,
    pub label: &'static str,
    pub family: MediaFamily,
    pub support: SupportLevel,
    pub metadata: bool,
    pub thumbnail: bool,
    pub preview: bool,
}

pub fn family(extension: &str) -> MediaFamily {
    let extension = extension.trim_start_matches('.').to_ascii_lowercase();
    if RAW_EXTENSIONS.contains(&extension.as_str()) {
        MediaFamily::Raw
    } else if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        MediaFamily::Video
    } else if PHOTO_EXTENSIONS.contains(&extension.as_str()) {
        MediaFamily::Photo
    } else {
        MediaFamily::Unknown
    }
}

pub fn descriptor(extension: &str) -> FormatDescriptor {
    let ext = extension.trim_start_matches('.').to_ascii_lowercase();
    let (key, label, support, thumbnail, preview) = match ext.as_str() {
        "invalid" => (
            "invalid",
            "Arquivo inválido",
            SupportLevel::Invalid,
            false,
            false,
        ),
        "jpg" | "jpeg" | "jpe" => ("jpeg", "JPEG", SupportLevel::Complete, true, true),
        "tif" | "tiff" => ("tiff", "TIFF", SupportLevel::Complete, true, true),
        "heic" | "heif" => ("heif", "HEIF/HEIC", SupportLevel::Partial, true, true),
        "jp2" | "j2k" | "jpf" | "jpx" => {
            ("jpeg2000", "JPEG 2000", SupportLevel::Partial, true, true)
        }
        "avif" => ("avif", "AVIF", SupportLevel::Partial, true, true),
        "jxl" => ("jxl", "JPEG XL", SupportLevel::Preservation, false, false),
        "cr2" | "cr3" | "crw" => ("canon_raw", "Canon RAW", SupportLevel::Partial, true, true),
        "nef" | "nrw" => ("nikon_raw", "Nikon RAW", SupportLevel::Partial, true, true),
        "arw" | "srf" | "sr2" => ("sony_raw", "Sony RAW", SupportLevel::Partial, true, true),
        "raf" => (
            "fujifilm_raw",
            "Fujifilm RAW",
            SupportLevel::Partial,
            true,
            true,
        ),
        "dng" => ("dng", "Adobe DNG", SupportLevel::Partial, true, true),
        "rw2" | "rwl" | "orf" | "pef" | "srw" | "3fr" | "fff" | "iiq" | "mef" | "mrw" | "dcr"
        | "kdc" | "erf" | "x3f" | "gpr" | "mos" => (
            "other_raw",
            "RAW de câmera",
            SupportLevel::Partial,
            true,
            true,
        ),
        "mp4" | "m4v" | "mov" | "qt" | "mkv" | "webm" | "avi" | "mpeg" | "mpg" | "mpe" | "vob"
        | "ts" | "mts" | "m2ts" | "3gp" | "3g2" | "wmv" | "asf" | "flv" | "f4v" | "ogv" | "mxf"
        | "dv" | "mod" | "tod" => ("video", "Vídeo", SupportLevel::Partial, true, true),
        "png" => ("png", "PNG", SupportLevel::Complete, true, true),
        "gif" => ("gif", "GIF", SupportLevel::Complete, true, true),
        "bmp" | "dib" => ("bmp", "Bitmap", SupportLevel::Complete, true, true),
        "webp" => ("webp", "WebP", SupportLevel::Complete, true, true),
        _ => (
            "unknown",
            "Formato desconhecido",
            SupportLevel::Unknown,
            false,
            false,
        ),
    };
    FormatDescriptor {
        key,
        label,
        family: family(&ext),
        support,
        metadata: key != "unknown",
        thumbnail,
        preview,
    }
}

pub fn canonical_from_signature(path: &Path) -> Option<&'static str> {
    let mut bytes = [0u8; 32];
    let read = File::open(path).ok()?.read(&mut bytes).ok()?;
    let b = &bytes[..read];
    if b.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("jpeg");
    }
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("png");
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        return Some("gif");
    }
    if b.starts_with(b"BM") {
        return Some("bmp");
    }
    if b.starts_with(b"II*\0") || b.starts_with(b"MM\0*") {
        return Some("tiff");
    }
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP") {
        return Some("webp");
    }
    if b.get(4..8) == Some(b"ftyp") {
        let brand = b.get(8..12).unwrap_or_default();
        if matches!(
            brand,
            b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1"
        ) {
            return Some("heif");
        }
        if matches!(brand, b"avif" | b"avis") {
            return Some("avif");
        }
        return Some("iso_bmff");
    }
    if b.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        return Some("matroska");
    }
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"AVI ") {
        return Some("avi");
    }
    None
}

pub fn detected_format(path: &Path, extension: &str) -> (&'static str, bool) {
    let declared = descriptor(extension);
    let signature = canonical_from_signature(path);
    if declared.family == MediaFamily::Raw {
        return (declared.key, true);
    }
    if declared.family == MediaFamily::Video {
        return match signature {
            Some("iso_bmff" | "matroska" | "avi") => (declared.key, true),
            Some(value) => (value, false),
            None => (declared.key, true),
        };
    }
    match signature {
        Some(value) => (value, value == declared.key),
        None => (declared.key, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_declared_extension_has_a_known_family() {
        for extension in SUPPORTED_EXTENSIONS {
            assert_ne!(family(extension), MediaFamily::Unknown, "{extension}");
        }
    }
    #[test]
    fn covers_major_photo_raw_and_video_families() {
        for extension in [
            "jpeg", "tiff", "avif", "jxl", "cr2", "cr3", "dng", "nef", "arw", "raf", "3fr", "mp4",
            "mov", "mxf", "webm", "mts",
        ] {
            assert_ne!(
                descriptor(extension).support,
                SupportLevel::Unknown,
                "{extension}"
            );
        }
    }
    #[test]
    fn preservation_is_not_invalid() {
        assert_eq!(descriptor("jxl").support, SupportLevel::Preservation);
        assert_ne!(descriptor("jxl").support, SupportLevel::Invalid);
    }
}
