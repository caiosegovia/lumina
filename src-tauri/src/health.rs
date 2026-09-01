use crate::{catalog, models::*};
use std::path::Path;

pub fn inspect(cfg: &LibraryConfig) -> Result<LibraryHealth, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let integrity: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    let offline = conn
        .query_row(
            "SELECT COUNT(*)FROM sources WHERE path NOT LIKE 'lumina://%' AND available=0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let thumbnail_pending=conn.query_row("SELECT COUNT(*)FROM assets a LEFT JOIN thumbnails t ON t.asset_id=a.id WHERE COALESCE(t.state,'missing')!='ready'",[],|row|row.get(0)).unwrap_or(0);
    let failed_jobs = conn
        .query_row(
            "SELECT COUNT(*)FROM jobs WHERE state IN('failed','backup_error')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let pending_protection = conn
        .query_row(
            "SELECT COUNT(*)FROM assets WHERE protection_state!='replica_verified'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let sync_errors = conn
        .query_row(
            "SELECT COUNT(*)FROM source_sync_settings WHERE last_state='failed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let checks = vec![
        HealthCheck {
            key: "catalog".into(),
            label: "Integridade do catálogo".into(),
            state: if integrity == "ok" {
                "healthy"
            } else {
                "error"
            }
            .into(),
            detail: if integrity == "ok" {
                "SQLite íntegro".into()
            } else {
                integrity
            },
        },
        HealthCheck {
            key: "master".into(),
            label: "Acervo mestre".into(),
            state: if Path::new(&cfg.master_path).is_dir() {
                "healthy"
            } else {
                "error"
            }
            .into(),
            detail: "Local principal da biblioteca".into(),
        },
        HealthCheck {
            key: "backup".into(),
            label: "Réplica local".into(),
            state: if Path::new(&cfg.backup_path).is_dir() {
                "healthy"
            } else {
                "warning"
            }
            .into(),
            detail: format!("{pending_protection} mídias aguardando proteção"),
        },
        HealthCheck {
            key: "sources".into(),
            label: "Fontes".into(),
            state: if offline == 0 { "healthy" } else { "warning" }.into(),
            detail: format!("{offline} fontes offline"),
        },
        HealthCheck {
            key: "thumbnails".into(),
            label: "Miniaturas".into(),
            state: if thumbnail_pending == 0 {
                "healthy"
            } else {
                "warning"
            }
            .into(),
            detail: format!("{thumbnail_pending} pendentes"),
        },
        HealthCheck {
            key: "jobs".into(),
            label: "Trabalhos".into(),
            state: if failed_jobs == 0 && sync_errors == 0 {
                "healthy"
            } else {
                "warning"
            }
            .into(),
            detail: format!("{} falhas que merecem revisão", failed_jobs + sync_errors),
        },
    ];
    Ok(LibraryHealth {
        overall: if checks.iter().any(|check| check.state == "error") {
            "error"
        } else if checks.iter().any(|check| check.state == "warning") {
            "attention"
        } else {
            "healthy"
        }
        .into(),
        checks,
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn healthy_catalog_is_reported_without_mutating_library() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let backup = root.join("backup");
        fs::create_dir_all(root.join(".lumina")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "health".into(),
            name: "Health".into(),
            master_path: root.to_string_lossy().into_owned(),
            backup_path: backup.to_string_lossy().into_owned(),
            created_at: Utc::now().to_rfc3339(),
        };
        catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        let health = inspect(&cfg).unwrap();
        assert_eq!(
            health
                .checks
                .iter()
                .find(|check| check.key == "catalog")
                .unwrap()
                .state,
            "healthy"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
