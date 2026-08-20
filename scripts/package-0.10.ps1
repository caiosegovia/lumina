$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$artifacts = Join-Path $workspace "artifacts"
$release = Join-Path $workspace "src-tauri\target-0.10-release\release"
$final = Join-Path $artifacts "Lumina-0.10.0-portable-windows-x64"
$zip = "$final.zip"
$staging = Join-Path $artifacts (".staging-Lumina-0.10-" + [guid]::NewGuid().ToString("N"))
if ((Test-Path $final) -or (Test-Path $zip)) { throw "A entrega 0.10.0 já existe; recuso substituição parcial." }
New-Item -ItemType Directory -Path $staging | Out-Null
try {
  Copy-Item (Join-Path $release "lumina.exe") (Join-Path $staging "Lumina.exe")
  Copy-Item (Join-Path $workspace "src-tauri\tools") $staging -Recurse
  foreach ($name in @("README.md", "SPEC-0.10.md", "TESTING.md", "TEST-REPORT-0.10.md", "ARCHITECTURE-STABILIZATION.md", "THIRD-PARTY-NOTICES.md")) { Copy-Item (Join-Path $workspace $name) $staging }
  $toolFiles = @(Get-ChildItem (Join-Path $staging "tools\exiftool_files") -Recurse -File)
  if ($toolFiles.Count -lt 500) { throw "ExifTool incompleto: apenas $($toolFiles.Count) arquivos auxiliares." }
  $version = & (Join-Path $staging "tools\exiftool.exe") -ver
  if ($LASTEXITCODE -ne 0 -or -not $version) { throw "ExifTool empacotado não inicia." }
  $manifest = foreach ($file in Get-ChildItem $staging -Recurse -File | Sort-Object FullName) {
    [pscustomobject]@{ path = $file.FullName.Substring($staging.Length + 1).Replace("\", "/"); bytes = $file.Length; sha256 = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
  }
  $manifest | ConvertTo-Json -Depth 3 | Set-Content (Join-Path $staging "MANIFEST.json") -Encoding utf8
  New-Item -ItemType Directory -Path $final | Out-Null
  Copy-Item (Join-Path $staging "*") $final -Recurse
  $copiedManifest = Get-Content (Join-Path $final "MANIFEST.json") -Raw | ConvertFrom-Json
  foreach ($entry in $copiedManifest) {
    $copied = Join-Path $final $entry.path.Replace("/", "\")
    if ((Get-FileHash $copied -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.sha256) { throw "Falha na promoção: $($entry.path)" }
  }
  Compress-Archive (Join-Path $final "*") $zip -CompressionLevel Optimal
  Get-Item $zip
} catch { throw }
