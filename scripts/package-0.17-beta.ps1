$ErrorActionPreference = "Stop"
$workspace=(Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$release=Join-Path $workspace "src-tauri\target-0.17-beta\release"
$artifacts=Join-Path $workspace "artifacts"
$final=Join-Path $artifacts "Lumina-0.17.0-beta.1-portable-windows-x64"
$zip="$final.zip"
if(Test-Path $final){$resolved=(Resolve-Path $final).Path;if(-not$resolved.StartsWith($artifacts,[StringComparison]::OrdinalIgnoreCase)){throw "Limpeza recusada"};Remove-Item -LiteralPath $resolved -Recurse -Force}
if(Test-Path $zip){$resolved=(Resolve-Path $zip).Path;if(-not$resolved.StartsWith($artifacts,[StringComparison]::OrdinalIgnoreCase)){throw "Limpeza recusada"};Remove-Item -LiteralPath $resolved -Force}
New-Item -ItemType Directory -Path $final -Force|Out-Null
Copy-Item (Join-Path $release "lumina.exe") (Join-Path $final "Lumina.exe")
Copy-Item (Join-Path $workspace "src-tauri\tools") $final -Recurse
foreach($name in @("README.md","SPEC-0.17-BETA.md","ARCHITECTURE-0.17.md","CLOSURE-0.16.md","RELEASE-0.17-BETA.md","ROADMAP.md")){Copy-Item (Join-Path $workspace $name) $final}
New-Item -ItemType Directory -Path (Join-Path $final "docs\design") -Force|Out-Null
Copy-Item (Join-Path $workspace "docs\design\*.svg") (Join-Path $final "docs\design")
$manifest=foreach($file in Get-ChildItem $final -Recurse -File|Sort-Object FullName){[pscustomobject]@{path=$file.FullName.Substring($final.Length+1).Replace("\","/");bytes=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}}
$manifest|ConvertTo-Json -Depth 3|Set-Content (Join-Path $final "MANIFEST.json") -Encoding utf8
Compress-Archive (Join-Path $final "*") $zip -CompressionLevel Optimal
Get-Item (Join-Path $final "Lumina.exe"),$zip
