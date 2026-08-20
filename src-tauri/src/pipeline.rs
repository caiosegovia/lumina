use walkdir::DirEntry;
pub const MEDIA: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "tif", "tiff", "bmp", "dng", "cr2", "cr3",
    "nef", "arw", "raf", "orf", "rw2", "mp4", "mov", "avi", "mkv", "mts", "m2ts", "3gp", "wmv",
];
const VIDEO: &[&str] = &["mp4", "mov", "avi", "mkv", "mts", "m2ts", "3gp", "wmv"];
const RAW: &[&str] = &["dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Analyzing,
    Ready,
    WaitingSpace,
    BatchPending,
    Consolidating,
    ProtectionPending,
    Protecting,
    WaitingBackupSpace,
    BackupError,
    Pausing,
    Paused,
    Canceling,
    Canceled,
    Interrupted,
    Failed,
    Completed,
}
impl JobState {
    #[cfg(test)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Analyzing => "analyzing",
            Self::Ready => "ready",
            Self::WaitingSpace => "waiting_space",
            Self::BatchPending => "batch_pending",
            Self::Consolidating => "consolidating",
            Self::ProtectionPending => "protection_pending",
            Self::Protecting => "protecting",
            Self::WaitingBackupSpace => "waiting_backup_space",
            Self::BackupError => "backup_error",
            Self::Pausing => "pausing",
            Self::Paused => "paused",
            Self::Canceling => "canceling",
            Self::Canceled => "canceled",
            Self::Interrupted => "interrupted",
            Self::Failed => "failed",
            Self::Completed => "completed",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkState {
    Pending,
    Processing,
    Completed,
    Failed,
}
impl WorkState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}
impl TryFrom<&str> for WorkState {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => return Err(()),
        })
    }
}
impl TryFrom<&str> for JobState {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "queued" => Self::Queued,
            "analyzing" => Self::Analyzing,
            "ready" => Self::Ready,
            "waiting_space" => Self::WaitingSpace,
            "batch_pending" => Self::BatchPending,
            "consolidating" => Self::Consolidating,
            "protection_pending" => Self::ProtectionPending,
            "protecting" => Self::Protecting,
            "waiting_backup_space" => Self::WaitingBackupSpace,
            "backup_error" => Self::BackupError,
            "pausing" => Self::Pausing,
            "paused" => Self::Paused,
            "canceling" => Self::Canceling,
            "canceled" => Self::Canceled,
            "interrupted" => Self::Interrupted,
            "failed" => Self::Failed,
            "completed" => Self::Completed,
            _ => return Err(()),
        })
    }
}
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
    let (Ok(from), Ok(to)) = (JobState::try_from(from), JobState::try_from(to)) else {
        return false;
    };
    matches!(
        (from, to),
        (JobState::Queued, JobState::Analyzing)
            | (JobState::Analyzing, JobState::Ready)
            | (JobState::Ready, JobState::Consolidating)
            | (JobState::Ready, JobState::WaitingSpace)
            | (JobState::WaitingSpace, JobState::Consolidating)
            | (JobState::BatchPending, JobState::Consolidating)
            | (JobState::ProtectionPending, JobState::Protecting)
            | (JobState::WaitingBackupSpace, JobState::Protecting)
            | (JobState::BackupError, JobState::Protecting)
            | (JobState::Protecting, JobState::Completed)
            | (JobState::Ready, JobState::Pausing)
            | (JobState::Consolidating, JobState::Pausing)
            | (JobState::Pausing, JobState::Paused)
            | (JobState::Pausing, JobState::Analyzing)
            | (JobState::Pausing, JobState::Consolidating)
            | (JobState::Pausing, JobState::Protecting)
            | (JobState::Paused, JobState::Analyzing)
            | (JobState::Paused, JobState::Consolidating)
            | (JobState::Paused, JobState::Protecting)
            | (_, JobState::Canceling)
            | (JobState::Canceling, JobState::Canceled)
            | (_, JobState::Interrupted)
            | (JobState::Interrupted, JobState::Analyzing)
            | (JobState::Interrupted, JobState::Consolidating)
            | (_, JobState::Failed)
            | (JobState::Consolidating, JobState::Completed)
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
    #[test]
    fn every_persisted_state_round_trips() {
        for value in [
            "queued",
            "analyzing",
            "ready",
            "waiting_space",
            "batch_pending",
            "consolidating",
            "protection_pending",
            "protecting",
            "waiting_backup_space",
            "backup_error",
            "pausing",
            "paused",
            "canceling",
            "canceled",
            "interrupted",
            "failed",
            "completed",
        ] {
            assert_eq!(JobState::try_from(value).unwrap().as_str(), value)
        }
        assert!(JobState::try_from("typo").is_err());
        for value in ["pending", "processing", "completed", "failed"] {
            assert_eq!(WorkState::try_from(value).unwrap().as_str(), value);
        }
        assert!(WorkState::try_from("typo").is_err());
    }
}
