$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$zip = (Resolve-Path (Join-Path $workspace "artifacts\Lumina-0.9.1-portable-windows-x64.zip")).Path
$extract = Join-Path $env:TEMP ("Lumina-091-Smoke-" + [guid]::NewGuid().ToString("N"))
Expand-Archive $zip $extract
$manifest = Get-Content (Join-Path $extract "MANIFEST.json") -Raw | ConvertFrom-Json
foreach ($entry in $manifest) {
  $path = Join-Path $extract $entry.path.Replace("/", "\")
  if (-not (Test-Path $path)) { throw "Ausente: $($entry.path)" }
  if ((Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Hash inválido: $($entry.path)" }
}
$toolFiles = @(Get-ChildItem (Join-Path $extract "tools\exiftool_files") -Recurse | Where-Object { -not $_.PSIsContainer })
if ($toolFiles.Count -lt 500) { throw "ExifTool incompleto no ZIP." }
$exifVersion = & (Join-Path $extract "tools\exiftool.exe") -ver
if ($LASTEXITCODE -ne 0 -or -not $exifVersion) { throw "ExifTool não inicia após extração." }
$data = Join-Path $extract "local-data"; New-Item -ItemType Directory $data | Out-Null
$info = [Diagnostics.ProcessStartInfo]::new(); $info.FileName = Join-Path $extract "Lumina.exe"; $info.WorkingDirectory = $extract; $info.UseShellExecute = $false; $info.CreateNoWindow = $true; $info.Environment["LOCALAPPDATA"] = $data; $info.Environment["Path"] = "$env:SystemRoot\System32;$env:SystemRoot"
$app = [Diagnostics.Process]::Start($info); $deadline = (Get-Date).AddSeconds(20)
do { Start-Sleep -Milliseconds 250; $app.Refresh() } while (-not $app.HasExited -and $app.MainWindowTitle -ne "Lumina Ready" -and (Get-Date) -lt $deadline)
if ($app.HasExited -or $app.MainWindowHandle -eq 0 -or -not $app.Responding) { throw "Aplicativo portátil não ficou pronto." }
$result = [pscustomobject]@{ ManifestEntries=$manifest.Count; ExifToolFiles=$toolFiles.Count; ExifToolVersion=$exifVersion; FrontendReady=$true; Responding=$app.Responding; Sha256=(Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant() }
$app.Kill(); $app.WaitForExit(); $result
