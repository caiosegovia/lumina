use std::path::Path;
pub fn replicate(master: &Path, backup: &Path, hash: &str) -> Result<(), String> {
    crate::storage::copy_verified(master, backup, hash)
}
pub fn verify(path: &Path, hash: &str) -> bool {
    crate::storage::sha256(path).ok().as_deref() == Some(hash)
}
