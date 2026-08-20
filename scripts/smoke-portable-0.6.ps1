$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$zip = (Resolve-Path (Join-Path $workspace "artifacts\Lumina-0.6.0-portable-windows-x64.zip")).Path
$extract = Join-Path $env:TEMP ("Lumina-06-Smoke-" + [guid]::NewGuid().ToString("N"))
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
$deadline = (Get-Date).AddSeconds(20)
do { Start-Sleep -Milliseconds 250; $app.Refresh() } while (-not $app.HasExited -and $app.MainWindowTitle -ne "Lumina Ready" -and (Get-Date) -lt $deadline)
if ($app.HasExited) { throw "Aplicativo encerrou: $($app.ExitCode)" }
if ($app.MainWindowHandle -eq 0 -or $app.MainWindowTitle -ne "Lumina Ready" -or -not $app.Responding) { $app.Kill(); throw "O frontend React nao sinalizou prontidao" }
$result = [pscustomobject]@{ ManifestEntries=$manifest.Count; WindowHandle=$app.MainWindowHandle; WindowTitle=$app.MainWindowTitle; FrontendReady=$true; Responding=$app.Responding; Sha256=(Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant() }
$app.Kill(); $app.WaitForExit()
$resolved = (Resolve-Path -LiteralPath $extract).Path
$temp = (Resolve-Path -LiteralPath $env:TEMP).Path
if (-not $resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase)) { throw "Limpeza recusada" }
Start-Sleep -Seconds 2
for ($attempt = 1; $attempt -le 10; $attempt++) { try { Remove-Item -LiteralPath $resolved -Recurse -Force; break } catch { if ($attempt -eq 10) { throw }; Start-Sleep -Seconds 1 } }
$result
