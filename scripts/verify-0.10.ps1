$ErrorActionPreference = "Stop"
function Gate([string]$name, [scriptblock]$command) {
  Write-Host "[GATE] $name"
  & $command
  if ($LASTEXITCODE -ne 0) { throw "$name falhou: $LASTEXITCODE" }
}
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$env:NODE_OPTIONS = "--use-system-ca"
Push-Location $workspace
try {
  $env:TAURI_CONFIG = '{"bundle":{"resources":[]}}'
  Gate "rust-format" { cargo fmt --manifest-path src-tauri\Cargo.toml -- --check }
  Gate "rust-tests" { cargo test --manifest-path src-tauri\Cargo.toml --lib -- --test-threads=1 }
  Remove-Item Env:TAURI_CONFIG
  Gate "frontend-tests" { npm.cmd test -- --run }
  Gate "frontend-build" { npm.cmd run build }
  Gate "dependency-audit" { npm.cmd audit --audit-level=moderate }
  Gate "diff-whitespace" { git diff --check }
} finally {
  Remove-Item Env:TAURI_CONFIG -ErrorAction SilentlyContinue
  Pop-Location
}
