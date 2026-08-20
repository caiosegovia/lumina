$ErrorActionPreference = "Stop"
function Gate([string]$name, [scriptblock]$command) {
    Write-Host "[GATE] $name"
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$name falhou: $LASTEXITCODE" }
}
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:NODE_OPTIONS = "--use-system-ca"
$vcvars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$devEnv = cmd.exe /d /s /c ('"' + $vcvars + '" >nul && set')
foreach ($line in $devEnv) { if ($line -match '^([^=]+)=(.*)$') { [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process") } }
$env:TAURI_CONFIG = '{"bundle":{"resources":[]}}'
Gate "rust-format" { cargo fmt --manifest-path src-tauri\Cargo.toml -- --check }
Gate "rust-tests" { cargo test --manifest-path src-tauri\Cargo.toml --lib }
Remove-Item Env:TAURI_CONFIG
Gate "frontend-tests" { npm.cmd test -- --run }
Gate "frontend-build" { npm.cmd run build }
Gate "dependency-audit" { npm.cmd audit --audit-level=moderate }
