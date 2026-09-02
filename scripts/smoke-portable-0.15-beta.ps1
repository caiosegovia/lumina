$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$zip = (Resolve-Path (Join-Path $workspace "artifacts\Lumina-0.15.0-beta.2-portable-windows-x64.zip")).Path
$extract = Join-Path $env:TEMP ("Lumina-015b2-Smoke-" + [guid]::NewGuid().ToString("N"))
Expand-Archive -LiteralPath $zip -DestinationPath $extract
$manifest = Get-Content (Join-Path $extract "MANIFEST.json") -Raw | ConvertFrom-Json
foreach ($entry in $manifest) {
  $path = Join-Path $extract $entry.path.Replace("/", "\")
  if (-not (Test-Path -LiteralPath $path)) { throw "Ausente no portátil: $($entry.path)" }
  $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($actual -ne $entry.sha256) { throw "Hash inválido: $($entry.path)" }
}
$data = Join-Path $extract "local-data"
New-Item -ItemType Directory -Path $data | Out-Null
$info = [Diagnostics.ProcessStartInfo]::new()
$info.FileName = Join-Path $extract "Lumina.exe"
$info.WorkingDirectory = $extract
$info.UseShellExecute = $false
$info.Environment["LOCALAPPDATA"] = $data
$app = [Diagnostics.Process]::Start($info)
$deadline = (Get-Date).AddSeconds(30)
do { Start-Sleep -Milliseconds 250; $app.Refresh() } while (-not $app.HasExited -and $app.MainWindowTitle -ne "Lumina Ready" -and (Get-Date) -lt $deadline)
if ($app.HasExited) { throw "Aplicativo encerrou no smoke: $($app.ExitCode)" }
if ($app.MainWindowHandle -eq 0 -or $app.MainWindowTitle -ne "Lumina Ready" -or -not $app.Responding) { throw "Frontend não ficou pronto" }
$workingSet = $app.WorkingSet64
$null = $app.CloseMainWindow()
if (-not $app.WaitForExit(10000)) { $app.Kill(); throw "Aplicativo não encerrou normalmente" }
$marker = Join-Path $data "Lumina\diagnostics\running.session"
if (Test-Path -LiteralPath $marker) { throw "Marcador de sessão não foi limpo" }
$result = [pscustomobject]@{
  ManifestEntries = $manifest.Count
  FrontendReady = $true
  CleanExit = $true
  WorkingSetBytes = $workingSet
  ZipSha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
}
$resolved = (Resolve-Path $extract).Path
$temp = (Resolve-Path $env:TEMP).Path
if (-not $resolved.StartsWith($temp, [StringComparison]::OrdinalIgnoreCase) -or $resolved -eq $temp) { throw "Limpeza recusada" }
for ($attempt = 1; $attempt -le 10; $attempt++) {
  try { Remove-Item -LiteralPath $resolved -Recurse -Force; break }
  catch { if ($attempt -eq 10) { throw }; Start-Sleep -Seconds 1 }
}
$result
