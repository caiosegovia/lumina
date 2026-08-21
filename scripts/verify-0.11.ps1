$ErrorActionPreference = "Stop"
function Gate([string]$name,[scriptblock]$command){Write-Host "[GATE] $name";& $command;if ($LASTEXITCODE -ne 0) {throw "$name falhou: $LASTEXITCODE"}}
$workspace=(Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$env:NODE_OPTIONS="--use-system-ca"
Push-Location $workspace
try {
  Gate "rust-format" { cargo fmt --manifest-path src-tauri\Cargo.toml -- --check }
  Gate "rust-tests" { cargo test --manifest-path src-tauri\Cargo.toml --lib -- --test-threads=1 }
  Gate "dashboard-scale" { cargo test --manifest-path src-tauri\Cargo.toml dashboard_rollups_scale_to_100k_and_500k_with_concurrent_reads -- --ignored --nocapture }
  Gate "frontend-tests" { npm.cmd test -- --run }
  Gate "frontend-build" { npm.cmd run build }
  Gate "dependency-audit" { npm.cmd audit --audit-level=moderate }
  Gate "diff-whitespace" { git diff --check }
} finally { Pop-Location }
