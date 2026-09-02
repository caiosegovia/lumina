$ErrorActionPreference = "Stop"
$workspace=(Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$release=Join-Path $workspace "src-tauri\target-0.15-beta\release"
$artifacts=Join-Path $workspace "artifacts"
$final=Join-Path $artifacts "Lumina-0.15.0-beta.1-portable-windows-x64"
$zip="$final.zip"
if(Test-Path $final){Remove-Item -LiteralPath $final -Recurse -Force}
if(Test-Path $zip){Remove-Item -LiteralPath $zip -Force}
New-Item -ItemType Directory -Path $final -Force|Out-Null
Copy-Item (Join-Path $release "lumina.exe") (Join-Path $final "Lumina.exe")
Copy-Item (Join-Path $workspace "src-tauri\tools") $final -Recurse
foreach($name in @("README.md","SPEC-0.15-BETA.md","ARCHITECTURE-0.14.md","DESIGN-0.15-GALLERY.md","RELEASE-0.15-BETA.md","ROADMAP.md")){Copy-Item (Join-Path $workspace $name) $final}
New-Item -ItemType Directory -Path (Join-Path $final "docs\design") -Force|Out-Null
Copy-Item (Join-Path $workspace "docs\design\*.svg") (Join-Path $final "docs\design")
$manifest=foreach($file in Get-ChildItem $final -Recurse -File|Sort-Object FullName){[pscustomobject]@{path=$file.FullName.Substring($final.Length+1).Replace("\","/");bytes=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}}
$manifest|ConvertTo-Json -Depth 3|Set-Content (Join-Path $final "MANIFEST.json") -Encoding utf8
Compress-Archive (Join-Path $final "*") $zip -CompressionLevel Optimal
Get-Item (Join-Path $final "Lumina.exe"),$zip
