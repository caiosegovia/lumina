use crate::{catalog, models::*};
use chrono::Utc;
use rusqlite::params;
use std::{collections::HashSet, fs, path::Path};
use uuid::Uuid;

pub fn create_plan(cfg: &LibraryConfig) -> Result<CleanupPlan, String> {
    let mut conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let mut statement=conn.prepare("SELECT o.id,o.asset_id,a.filename,s.name,o.path,a.bytes,CASE WHEN a.protection_state='replica_verified' THEN 'eligible' ELSE 'blocked' END,CASE WHEN a.protection_state='replica_verified' THEN 'Réplica verificada; ocorrência adicional' ELSE 'Réplica ainda não verificada' END FROM(SELECT ao.*,ROW_NUMBER()OVER(PARTITION BY ao.asset_id ORDER BY ao.seen_at,ao.id)position FROM active_occurrences ao)o JOIN assets a ON a.id=o.asset_id JOIN sources s ON s.id=o.source_id WHERE(SELECT COUNT(*)FROM active_occurrences x WHERE x.asset_id=o.asset_id)>1 AND o.position>1 ORDER BY a.filename,s.name,o.path").map_err(|error|error.to_string())?;
    let items = statement
        .query_map([], |row| {
            Ok(CleanupPlanItem {
                occurrence_id: row.get(0)?,
                asset_id: row.get(1)?,
                filename: row.get(2)?,
                source: row.get(3)?,
                path: row.get(4)?,
                bytes: row.get(5)?,
                eligibility: row.get(6)?,
                reason: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(statement);
    let groups = items
        .iter()
        .map(|item| &item.asset_id)
        .collect::<HashSet<_>>()
        .len() as i64;
    let candidates = items
        .iter()
        .filter(|item| item.eligibility == "eligible")
        .count() as i64;
    let blocked = items.len() as i64 - candidates;
    let bytes = items
        .iter()
        .filter(|item| item.eligibility == "eligible")
        .map(|item| item.bytes)
        .sum();
    let summary = serde_json::json!({"groups":groups,"candidates":candidates,"bytes":bytes,"blocked":blocked});
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE cleanup_plans SET state='superseded' WHERE state IN('draft','validated')",
        [],
    )
    .map_err(|error| error.to_string())?;
    tx.execute("INSERT INTO cleanup_plans(id,state,summary_json,created_at,validated_at)VALUES(?1,'validated',?2,?3,?3)",params![id,summary.to_string(),created_at]).map_err(|error|error.to_string())?;
    for item in &items {
        tx.execute("INSERT INTO cleanup_plan_items(plan_id,occurrence_id,asset_id,bytes,eligibility,reason)VALUES(?1,?2,?3,?4,?5,?6)",params![id,item.occurrence_id,item.asset_id,item.bytes,item.eligibility,item.reason]).map_err(|error|error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(CleanupPlan {
        id,
        state: "validated".into(),
        groups,
        candidates,
        bytes,
        blocked,
        items,
        created_at,
    })
}

pub fn export_plan(cfg: &LibraryConfig, plan_id: &str) -> Result<ReportExport, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let (state, summary, created_at): (String, String, String) = conn
        .query_row(
            "SELECT state,summary_json,created_at FROM cleanup_plans WHERE id=?1",
            [plan_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|_| "Plano de limpeza não encontrado".to_string())?;
    let mut statement = conn.prepare("SELECT i.occurrence_id,i.asset_id,a.filename,s.name,o.path,i.bytes,i.eligibility,i.reason FROM cleanup_plan_items i JOIN assets a ON a.id=i.asset_id JOIN occurrences o ON o.id=i.occurrence_id JOIN sources s ON s.id=o.source_id WHERE i.plan_id=?1 ORDER BY a.filename,s.name,o.path").map_err(|error|error.to_string())?;
    let items = statement.query_map([plan_id], |row| Ok(serde_json::json!({"occurrenceId":row.get::<_,String>(0)?,"assetId":row.get::<_,String>(1)?,"filename":row.get::<_,String>(2)?,"source":row.get::<_,String>(3)?,"path":row.get::<_,String>(4)?,"bytes":row.get::<_,i64>(5)?,"eligibility":row.get::<_,String>(6)?,"reason":row.get::<_,String>(7)?}))).map_err(|error|error.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())?;
    let reports = Path::new(&cfg.master_path).join(".lumina/reports");
    fs::create_dir_all(&reports).map_err(|error| error.to_string())?;
    let path = reports.join(format!("cleanup-plan-{plan_id}.json"));
    let document = serde_json::json!({"schema":"lumina.cleanup-plan.v1","planId":plan_id,"state":state,"createdAt":created_at,"summary":serde_json::from_str::<serde_json::Value>(&summary).unwrap_or_default(),"notice":"Relatório de simulação; nenhum arquivo foi removido.","items":items});
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(ReportExport {
        path: path.to_string_lossy().into_owned(),
        rows: items.len() as i64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn plan_never_marks_unprotected_occurrences_as_eligible() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(root.join(".lumina")).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: root.to_string_lossy().into(),
            backup_path: root.join("backup").to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO sources(id,name,path,volume_label)VALUES('s1','A','a','a'),('s2','B','b','b')",[]).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,protection_state,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'file',10,'m','source_only',?2)",params!["a".repeat(64),Utc::now().to_rfc3339()]).unwrap();
        conn.execute("INSERT INTO occurrences(id,asset_id,source_id,path,seen_at)VALUES('o1','a','s1','a.jpg',?1),('o2','a','s2','a.jpg',?1)",[Utc::now().to_rfc3339()]).unwrap();
        drop(conn);
        let plan = create_plan(&cfg).unwrap();
        assert_eq!(plan.candidates, 0);
        assert_eq!(plan.blocked, 1);
        fs::remove_dir_all(root).unwrap();
    }
}
