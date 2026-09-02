use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryConfig {
    pub id: String,
    pub name: String,
    pub master_path: String,
    pub backup_path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_assets: i64,
    pub photos: i64,
    pub videos: i64,
    pub bytes: i64,
    pub protected: i64,
    pub pending: i64,
    pub duplicate_groups: i64,
    pub duplicate_bytes: i64,
    pub reclaimable_bytes: i64,
    pub errors: i64,
    pub offline_sources: i64,
    pub oldest: Option<String>,
    pub newest: Option<String>,
    pub master_available_bytes: u64,
    pub backup_available_bytes: u64,
    pub types: Vec<DashboardBreakdown>,
    pub years: Vec<DashboardBreakdown>,
    pub protection: Vec<DashboardBreakdown>,
    #[serde(default)]
    pub protection_years: Vec<DashboardBreakdown>,
    #[serde(default)]
    pub protection_sources: Vec<DashboardBreakdown>,
    pub sources: Vec<DashboardSource>,
    pub months: Vec<DashboardBreakdown>,
    pub formats: Vec<DashboardFormat>,
    pub cameras: Vec<DashboardBreakdown>,
    pub insights: Vec<DashboardInsight>,
    pub latest_benchmark: Option<DashboardBenchmark>,
    pub snapshot_generated_at: String,
    pub stale: bool,
    pub timings: Vec<DashboardTiming>,
    #[serde(default)]
    pub storage: DashboardStorage,
    #[serde(default)]
    pub technical: DashboardTechnical,
    #[serde(default)]
    pub codecs: Vec<DashboardBreakdown>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStorage {
    pub master_total_bytes: u64,
    pub master_used_bytes: u64,
    pub master_free_bytes: u64,
    pub library_bytes: u64,
    pub cache_bytes: u64,
    pub temporary_bytes: u64,
    pub backup_total_bytes: u64,
    pub backup_used_bytes: u64,
    pub backup_free_bytes: u64,
    pub pending_backup_bytes: u64,
    pub projected_backup_free_bytes: i64,
    pub reserve_bytes: u64,
    pub estimated_additional_items: i64,
    pub average_asset_bytes: i64,
    pub p90_asset_bytes: i64,
    pub backup_available: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTechnical {
    pub enriched: i64,
    pub complete: i64,
    pub partial: i64,
    pub preservation: i64,
    pub unknown: i64,
    pub mismatches: i64,
    pub codec_known: i64,
    pub codec_missing: i64,
    pub thumbnails_ready: i64,
    pub thumbnails_pending: i64,
    pub thumbnails_failed: i64,
    pub metadata_complete: i64,
    pub review_items: i64,
    pub review_bytes: i64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardBreakdown {
    pub key: String,
    pub items: i64,
    pub bytes: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSource {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub items: i64,
    pub bytes: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardFormat {
    pub key: String,
    pub label: String,
    pub family: String,
    pub support: String,
    pub items: i64,
    pub bytes: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardTiming {
    pub section: String,
    pub milliseconds: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardInsight {
    pub kind: String,
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub value: i64,
    pub bytes: i64,
    pub action: String,
    pub action_label: String,
    pub confidence: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardBenchmark {
    pub job_id: String,
    pub items: i64,
    pub bytes: i64,
    pub analysis_ms: i64,
    pub hashing_ms: i64,
    pub copy_ms: i64,
    pub thumbnails_ms: i64,
    pub hash_workers: i64,
    pub hashed_bytes: i64,
    pub deferred_hash_items: i64,
    pub cache_hits: i64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub name: String,
    pub path: String,
    pub volume_label: String,
    pub available: bool,
    pub last_scan: Option<String>,
    pub asset_count: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSyncSummary {
    pub job_id: String,
    pub source_id: String,
    pub discovered: i64,
    pub present: i64,
    pub new_files: i64,
    pub duplicates: i64,
    pub changed: i64,
    pub missing: i64,
    pub failed: i64,
    pub processed_bytes: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSummary {
    pub review_later: i64,
    pub suspicious_dates: i64,
    pub missing_previews: i64,
    pub incomplete_metadata: i64,
    pub pending_protection: i64,
    pub undecided_duplicates: i64,
    pub technical_failures: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheck {
    pub key: String,
    pub label: String,
    pub state: String,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryHealth {
    pub overall: String,
    pub checks: Vec<HealthCheck>,
    pub generated_at: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: String,
    pub filename: String,
    pub media_type: String,
    pub extension: String,
    pub captured_at: String,
    pub date_source: String,
    pub date_suspicious: bool,
    pub bytes: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub duration: Option<f64>,
    pub camera: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub thumbnail: Option<String>,
    pub master_path: String,
    pub hash: String,
    pub protection_state: String,
    pub occurrence_count: i64,
    pub source_names: Vec<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub rating: i64,
    pub review_later: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetails {
    pub camera: Option<String>,
    pub detected_format: Option<String>,
    pub mime: Option<String>,
    pub container: Option<String>,
    pub codec: Option<String>,
    pub audio_codec: Option<String>,
    pub frame_rate: Option<f64>,
    pub bitrate: Option<i64>,
    pub pixel_format: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<i64>,
    pub aperture: Option<f64>,
    pub exposure: Option<String>,
    pub focal_length: Option<f64>,
    pub orientation: Option<i64>,
    pub color_profile: Option<String>,
    pub support_level: Option<String>,
    pub inventory_state: Option<String>,
    pub inventory_error: Option<String>,
    pub enriched_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GalleryFilters {
    pub query: String,
    pub year: Option<i32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub media_type: Option<String>,
    pub camera: Option<String>,
    pub source_id: Option<String>,
    pub original_folder: Option<String>,
    pub extension: Option<String>,
    pub has_location: Option<bool>,
    pub tag_id: Option<String>,
    pub album_id: Option<String>,
    pub protection_state: Option<String>,
    pub date_suspicious: Option<bool>,
    pub favorite: Option<bool>,
    pub minimum_rating: Option<i64>,
    pub review_later: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedView {
    pub id: String,
    pub name: String,
    pub filters: GalleryFilters,
    pub smart_album: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStateUpdate {
    pub asset_ids: Vec<String>,
    pub favorite: Option<bool>,
    pub rating: Option<i64>,
    pub review_later: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GalleryRequest {
    pub filters: GalleryFilters,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryYearCount {
    pub year: String,
    pub count: i64,
    pub bytes: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GallerySummary {
    pub total: i64,
    pub bytes: i64,
    pub photos: i64,
    pub videos: i64,
    pub raw: i64,
    pub protected: i64,
    pub with_location: i64,
    pub duplicate_assets: i64,
    pub favorites: i64,
    pub incomplete_metadata: i64,
    pub pending_protection: i64,
    pub years: Vec<GalleryYearCount>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterOption {
    pub value: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryFilterOptions {
    pub cameras: Vec<FilterOption>,
    pub sources: Vec<FilterOption>,
    pub extensions: Vec<FilterOption>,
    pub tags: Vec<FilterOption>,
    pub albums: Vec<FilterOption>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryResult {
    pub assets: Vec<MediaAsset>,
    pub matched: i64,
    pub next_cursor: Option<String>,
    pub summary: GallerySummary,
    pub options: GalleryFilterOptions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailAudit {
    pub total: i64,
    pub valid: i64,
    pub missing: i64,
    pub stale: i64,
    pub corrupt: i64,
    pub regenerated: i64,
    pub failed: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRepairProgress {
    pub running: bool,
    pub processed: i64,
    pub total: i64,
    pub regenerated: i64,
    pub failed: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportIssue {
    pub kind: String,
    pub extension: String,
    pub items: i64,
    pub bytes: i64,
    pub message: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub job_id: String,
    pub source_id: String,
    pub source_path: String,
    pub discovered: i64,
    pub new_files: i64,
    pub duplicates: i64,
    pub invalid: i64,
    pub required_bytes: i64,
    pub available_bytes: u64,
    pub excluded: i64,
    pub issues: Vec<ImportIssue>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePlan {
    pub master_required_bytes: u64,
    pub backup_required_bytes: u64,
    pub reserve_bytes: u64,
    pub master_available_bytes: u64,
    pub backup_available_bytes: u64,
    pub same_volume: bool,
    pub can_consolidate: bool,
    pub can_protect: bool,
    pub missing_bytes: u64,
    pub backup_missing_bytes: u64,
    pub selected_items: i64,
    pub selected_bytes: u64,
    pub maximum_safe_bytes: u64,
    pub maximum_safe_items: i64,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRequest {
    pub job_id: String,
    pub mode: String,
    pub value: Option<String>,
    pub maximum_bytes: Option<u64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionResult {
    pub selected_items: i64,
    pub selected_bytes: u64,
    pub pending_items: i64,
    pub pending_bytes: u64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtectionQueueStats {
    pub pending: i64,
    pub processing: i64,
    pub completed: i64,
    pub failed: i64,
    pub pending_bytes: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationProgress {
    pub id: String,
    pub old_master: String,
    pub new_master: String,
    pub state: String,
    pub processed_items: i64,
    pub total_items: i64,
    pub processed_bytes: i64,
    pub total_bytes: i64,
    pub last_error: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportEvent {
    pub id: i64,
    pub job_id: String,
    pub at: String,
    pub path: String,
    pub state: String,
    pub details: String,
}
#[derive(Serialize)]
pub struct Occurrence {
    pub id: String,
    pub source: String,
    pub path: String,
    pub decision: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub asset_id: String,
    pub hash: String,
    pub filename: String,
    pub bytes: i64,
    pub additional_bytes: i64,
    pub reclaimable_bytes: i64,
    pub safety: String,
    pub occurrences: Vec<Occurrence>,
    pub decision: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateStatus {
    pub state: String,
    pub catalog_assets: i64,
    pub exact_groups: i64,
    pub occurrences: i64,
    pub connected_sources: i64,
    pub total_sources: i64,
    pub last_scan: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlanItem {
    pub occurrence_id: String,
    pub asset_id: String,
    pub filename: String,
    pub source: String,
    pub path: String,
    pub bytes: i64,
    pub eligibility: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub id: String,
    pub state: String,
    pub groups: i64,
    pub candidates: i64,
    pub bytes: i64,
    pub blocked: i64,
    pub items: Vec<CleanupPlanItem>,
    pub created_at: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub name: String,
    pub asset_count: i64,
    pub cover: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagInfo {
    pub id: String,
    pub name: String,
    pub asset_count: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobOverview {
    pub job_id: String,
    pub source_name: String,
    pub source_path: String,
    pub state: String,
    pub stage: String,
    pub processed_items: i64,
    pub total_items: i64,
    pub processed_bytes: i64,
    pub total_bytes: i64,
    pub overall_percent: f64,
    pub bytes_per_second: Option<f64>,
    pub estimated_seconds_remaining: Option<i64>,
    pub imported: i64,
    pub duplicates: i64,
    pub excluded: i64,
    pub failed: i64,
    pub created_at: String,
    pub updated_at: String,
    pub interruption_reason: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub affected: i64,
}
#[derive(Serialize)]
pub struct VerifyResult {
    pub checked: i64,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    pub job_id: String,
    pub state: String,
    pub stage: String,
    pub current_file: Option<String>,
    pub processed_items: i64,
    pub total_items: i64,
    pub processed_bytes: i64,
    pub total_bytes: i64,
    pub imported: i64,
    pub duplicates: i64,
    pub excluded: i64,
    pub failed: i64,
    pub stage_percent: f64,
    pub overall_percent: f64,
    pub bytes_per_second: Option<f64>,
    pub estimated_seconds_remaining: Option<i64>,
    pub library_state: String,
    pub backup_state: String,
}

impl JobProgress {
    #[cfg(test)]
    pub fn overall_percent(&self) -> f64 {
        if self.total_bytes > 0 {
            (self.processed_bytes as f64 / self.total_bytes as f64 * 100.0).clamp(0.0, 100.0)
        } else if self.total_items > 0 {
            (self.processed_items as f64 / self.total_items as f64 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_prefers_bytes_and_is_bounded() {
        let mut progress = JobProgress {
            job_id: "j".into(),
            state: "consolidating".into(),
            stage: "copying".into(),
            current_file: None,
            processed_items: 1,
            total_items: 2,
            processed_bytes: 75,
            total_bytes: 100,
            imported: 0,
            duplicates: 0,
            excluded: 0,
            failed: 0,
            stage_percent: 0.0,
            overall_percent: 0.0,
            bytes_per_second: None,
            estimated_seconds_remaining: None,
            library_state: "pending".into(),
            backup_state: "pending".into(),
        };
        assert_eq!(progress.overall_percent(), 75.0);
        progress.processed_bytes = 150;
        assert_eq!(progress.overall_percent(), 100.0);
        progress.total_bytes = 0;
        assert_eq!(progress.overall_percent(), 50.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableJob {
    pub job_id: String,
    pub source_path: String,
    pub state: String,
    pub stage: String,
    pub interruption_reason: Option<String>,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEventPage {
    pub events: Vec<ImportEvent>,
    pub next_cursor: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExport {
    pub path: String,
    pub rows: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheResult {
    pub generated: i64,
    pub failed: i64,
}
