use std::path::PathBuf;

// Explicit opt-in for isolated QA. Without the variable, preserve the exact
// historical Windows application-data location. Relative overrides are rejected.
pub fn local_data() -> PathBuf {
    if let Some(path) = std::env::var_os("LUMINA_DATA_DIR").map(PathBuf::from) {
        assert!(
            path.is_absolute(),
            "LUMINA_DATA_DIR must be an absolute path; refusing to open the default profile"
        );
        return path;
    }
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
}
