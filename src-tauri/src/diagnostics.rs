use chrono::Utc;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    panic,
    path::PathBuf,
};

fn root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Lumina/diagnostics")
}

fn append(kind: &str, detail: &str) {
    let directory = root();
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let log = directory.join("session.log");
    if fs::metadata(&log)
        .map(|meta| meta.len() > 512 * 1024)
        .unwrap_or(false)
    {
        let previous = directory.join("session.previous.log");
        let _ = fs::remove_file(&previous);
        let _ = fs::rename(&log, previous);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log) {
        let safe_detail = detail
            .replace(['\r', '\n'], " ")
            .chars()
            .take(500)
            .collect::<String>();
        let _ = writeln!(
            file,
            "{}\t{}\t{}",
            Utc::now().to_rfc3339(),
            kind,
            safe_detail
        );
    }
}

pub fn start_session() {
    let directory = root();
    let marker = directory.join("running.session");
    if marker.exists() {
        append(
            "previous_session_abnormal",
            "O aplicativo não registrou encerramento normal",
        );
    }
    let _ = fs::create_dir_all(&directory);
    let _ = fs::write(&marker, Utc::now().to_rfc3339());
    append("session_started", env!("CARGO_PKG_VERSION"));
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .and_then(|location| std::path::Path::new(location.file()).file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        append(
            "panic",
            &format!(
                "componente={location} linha={}",
                info.location().map(|value| value.line()).unwrap_or(0)
            ),
        );
        default_hook(info);
    }));
}

pub fn finish_session() {
    append("session_finished", "clean");
    let _ = fs::remove_file(root().join("running.session"));
}

pub fn client_error(kind: &str, detail: &str) {
    let safe_kind = match kind {
        "frontend_error" | "unhandled_rejection" | "media_error" => kind,
        _ => "client_error",
    };
    append(safe_kind, &crate::process::sanitize(detail));
}
