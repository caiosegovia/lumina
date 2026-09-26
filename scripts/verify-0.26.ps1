param([switch]$SkipDebugResourceCopy)
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$previousProfile = $env:LUMINA_DATA_DIR
$previousConfig = $env:TAURI_CONFIG
Push-Location $workspace
try {
    $env:LUMINA_DATA_DIR = Join-Path $workspace ('artifacts\0.26\verify-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $env:LUMINA_DATA_DIR | Out-Null
    # Only opt in when Windows denies replacing existing debug resource copies.
    # Never use this override for the distribution build.
    if ($SkipDebugResourceCopy) { $env:TAURI_CONFIG = '{"bundle":{"resources":[]}}' }
    & npm.cmd test
    if ($LASTEXITCODE -ne 0) { throw 'Frontend tests failed' }
    & npm.cmd run build
    if ($LASTEXITCODE -ne 0) { throw 'Frontend build failed' }
    & cargo fmt --manifest-path src-tauri/Cargo.toml --check
    if ($LASTEXITCODE -ne 0) { throw 'Rust formatting failed' }
    & cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Rust lint failed' }
    & cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Backend tests failed' }
    Write-Host "Validation passed. Isolated diagnostics: $env:LUMINA_DATA_DIR"
} finally {
    $env:LUMINA_DATA_DIR = $previousProfile
    $env:TAURI_CONFIG = $previousConfig
    Pop-Location
}
