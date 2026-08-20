$ErrorActionPreference = "Stop"

function Invoke-Gate([string]$Name, [scriptblock]$Command) {
    Write-Host "[GATE] $Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name falhou com código $LASTEXITCODE"
    }
}

$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$vcvars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$devEnv = cmd.exe /d /s /c ('"' + $vcvars + '" >nul && set')
foreach ($line in $devEnv) {
    if ($line -match '^([^=]+)=(.*)$') {
        [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

Invoke-Gate "rust-format" { cargo fmt --manifest-path src-tauri\Cargo.toml -- --check }
Invoke-Gate "rust-tests" { cargo test --manifest-path src-tauri\Cargo.toml --lib }
Invoke-Gate "frontend-tests" { npm.cmd test -- --run }
Invoke-Gate "frontend-build" { npm.cmd run build }
