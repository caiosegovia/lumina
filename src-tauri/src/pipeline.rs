use walkdir::DirEntry;
pub const MEDIA: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "tif", "tiff", "bmp", "dng", "cr2", "cr3",
    "nef", "arw", "raf", "orf", "rw2", "mp4", "mov", "avi", "mkv", "mts", "m2ts", "3gp", "wmv",
];
const VIDEO: &[&str] = &["mp4", "mov", "avi", "mkv", "mts", "m2ts", "3gp", "wmv"];
const RAW: &[&str] = &["dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2"];
pub fn media_type(ext: &str) -> &'static str {
    if VIDEO.contains(&ext) {
        "video"
    } else if RAW.contains(&ext) {
        "raw"
    } else {
        "photo"
    }
}
pub fn ignored(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if entry.depth() == 0 {
        return false;
    }
    name.starts_with('.')
        || matches!(
            name.to_ascii_lowercase().as_str(),
            "node_modules"
                | "appdata"
                | "$recycle.bin"
                | "system volume information"
                | "thumbs"
                | "cache"
                | "tmp"
                | "temp"
        )
        || (entry.file_type().is_dir() && entry.path().join(".nomedia").exists())
}
pub fn valid_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("queued", "analyzing")
            | ("analyzing", "ready")
            | ("ready", "consolidating")
            | ("ready", "waiting_space")
            | ("waiting_space", "consolidating")
            | ("batch_pending", "consolidating")
            | ("protection_pending", "protecting")
            | ("waiting_backup_space", "protecting")
            | ("backup_error", "protecting")
            | ("protecting", "completed")
            | ("ready", "pausing")
            | ("consolidating", "pausing")
            | ("pausing", "paused")
            | ("pausing", "analyzing")
            | ("pausing", "consolidating")
            | ("pausing", "protecting")
            | ("paused", "analyzing")
            | ("paused", "consolidating")
            | ("paused", "protecting")
            | (_, "canceling")
            | ("canceling", "canceled")
            | (_, "interrupted")
            | ("interrupted", "analyzing")
            | ("interrupted", "consolidating")
            | (_, "failed")
            | ("consolidating", "completed")
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_state_jump() {
        assert!(!valid_transition("queued", "completed"));
        assert!(valid_transition("paused", "consolidating"));
    }
}
