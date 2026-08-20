$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$artifacts = Join-Path $workspace "artifacts"
$release = Join-Path $workspace "src-tauri\target-0.9.2-release\release"
$final = Join-Path $artifacts "Lumina-0.9.2-portable-windows-x64"
$zip = "$final.zip"
$staging = Join-Path $artifacts (".staging-Lumina-0.9.2-" + [guid]::NewGuid().ToString("N"))
if ((Test-Path $final) -or (Test-Path $zip)) { throw "A entrega 0.9.2 já existe; recuso substituição parcial." }
New-Item -ItemType Directory -Path $staging | Out-Null
try {
  Copy-Item (Join-Path $release "lumina.exe") (Join-Path $staging "Lumina.exe")
  Copy-Item (Join-Path $workspace "src-tauri\tools") $staging -Recurse
  foreach ($name in @("README.md", "SPEC-0.9.md", "SPEC-0.9.1.md", "TESTING.md", "TEST-REPORT-0.9.md", "TEST-REPORT-0.9.1.md", "TRACEABILITY-0.9.md", "THIRD-PARTY-NOTICES.md")) { Copy-Item (Join-Path $workspace $name) $staging }
  $toolFiles = @(Get-ChildItem (Join-Path $staging "tools\exiftool_files") -Recurse | Where-Object { -not $_.PSIsContainer })
  if ($toolFiles.Count -lt 500) { throw "ExifTool incompleto: apenas $($toolFiles.Count) arquivos auxiliares." }
  $exif = Join-Path $staging "tools\exiftool.exe"
  $version = & $exif -ver
  if ($LASTEXITCODE -ne 0 -or -not $version) { throw "ExifTool empacotado não inicia." }
  $manifest = foreach ($file in Get-ChildItem $staging -Recurse | Where-Object { -not $_.PSIsContainer } | Sort-Object FullName) {
    [pscustomobject]@{ path = $file.FullName.Substring($staging.Length + 1).Replace("\", "/"); bytes = $file.Length; sha256 = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() }
  }
  $manifest | ConvertTo-Json -Depth 3 | Set-Content (Join-Path $staging "MANIFEST.json") -Encoding utf8
  Rename-Item -LiteralPath $staging -NewName (Split-Path $final -Leaf)
  Compress-Archive (Join-Path $final "*") $zip -CompressionLevel Optimal
  Get-Item $zip
} catch { throw }
