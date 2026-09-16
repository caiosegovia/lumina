use std::path::Path;

pub const MAX_MEDIA_FILE_BYTES: u64 = 64 * 1024 * 1024 * 1024;
pub const MAX_PROTOCOL_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_PROCESS_STREAM_BYTES: usize = 8 * 1024 * 1024;

pub fn ensure_supported_size(bytes: u64) -> Result<(), String> {
    if bytes > MAX_MEDIA_FILE_BYTES {
        Err(format!(
            "FILE_TOO_LARGE: {bytes} bytes excedem o limite de {} bytes desta versão",
            MAX_MEDIA_FILE_BYTES
        ))
    } else {
        Ok(())
    }
}

pub fn ensure_supported_file(path: &Path) -> Result<u64, String> {
    let bytes = path.metadata().map_err(|error| error.to_string())?.len();
    ensure_supported_size(bytes)?;
    Ok(bytes)
}

pub fn read_protocol_file(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = path.metadata().map_err(|error| error.to_string())?.len();
    if bytes > MAX_PROTOCOL_RESPONSE_BYTES {
        return Err(format!(
            "PROTOCOL_RESPONSE_TOO_LARGE: {bytes} bytes excedem {} bytes",
            MAX_PROTOCOL_RESPONSE_BYTES
        ));
    }
    std::fs::read(path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_exact_file_limit_and_rejects_the_next_byte() {
        assert!(ensure_supported_size(MAX_MEDIA_FILE_BYTES).is_ok());
        assert!(ensure_supported_size(MAX_MEDIA_FILE_BYTES + 1).is_err());
    }
}
