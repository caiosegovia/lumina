$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$zip = (Resolve-Path (Join-Path $workspace "artifacts\Lumina-0.5.0-portable-windows-x64.zip")).Path
$extract = Join-Path $env:TEMP ("Lumina-05-Smoke-" + [guid]::NewGuid().ToString("N"))
Expand-Archive -LiteralPath $zip -DestinationPath $extract
$manifest = Get-Content (Join-Path $extract "MANIFEST.json") -Raw | ConvertFrom-Json
foreach ($entry in $manifest) {
    $path = Join-Path $extract $entry.path.Replace("/", "\")
    if (-not (Test-Path -LiteralPath $path)) { throw "Ausente: $($entry.path)" }
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Hash invalido: $($entry.path)" }
}
$data = Join-Path $extract "local-data"
New-Item -ItemType Directory -Path $data | Out-Null
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = Join-Path $extract "Lumina.exe"
$info.WorkingDirectory = $extract
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.Environment["LOCALAPPDATA"] = $data
$info.Environment["Path"] = "$env:SystemRoot\System32;$env:SystemRoot"
$app = [Diagnostics.Process]::Start($info)
$deadline = (Get-Date).AddSeconds(15)
do { Start-Sleep -Milliseconds 250; $app.Refresh() } while (-not $app.HasExited -and $app.MainWindowTitle -ne "Lumina Ready" -and (Get-Date) -lt $deadline)
if ($app.HasExited) { throw "Aplicativo encerrou: $($app.ExitCode)" }
if ($app.MainWindowHandle -eq 0 -or $app.MainWindowTitle -ne "Lumina Ready" -or -not $app.Responding) { $app.Kill(); throw "O frontend React nao sinalizou prontidao" }
$handle = $app.MainWindowHandle
$title = $app.MainWindowTitle
$app.Kill()
$app.WaitForExit()
$sha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
$resolved = (Resolve-Path -LiteralPath $extract).Path
$temp = (Resolve-Path -LiteralPath $env:TEMP).Path
if (-not $resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) { throw "Limpeza recusada" }
Start-Sleep -Seconds 1
for ($attempt = 1; $attempt -le 5; $attempt++) {
    try { Remove-Item -LiteralPath $resolved -Recurse -Force; break } catch { if ($attempt -eq 5) { throw }; Start-Sleep -Seconds 1 }
}
[pscustomobject]@{ ManifestEntries=$manifest.Count; WindowHandle=$handle; WindowTitle=$title; FrontendReady=$true; Responding=$true; Sha256=$sha }
