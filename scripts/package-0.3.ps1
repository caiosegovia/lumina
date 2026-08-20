$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$artifacts = Join-Path $workspace "artifacts"
$portable = Join-Path $artifacts "Lumina-0.3.0-portable-windows-x64"
$zip = Join-Path $artifacts "Lumina-0.3.0-portable-windows-x64.zip"
$release = Join-Path $workspace "src-tauri\target\release"
New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
if(Test-Path -LiteralPath $portable){$resolved=(Resolve-Path -LiteralPath $portable).Path;if(-not $resolved.StartsWith($artifacts,[StringComparison]::OrdinalIgnoreCase)){throw "Destino fora de artifacts"};Remove-Item -LiteralPath $resolved -Recurse -Force}
New-Item -ItemType Directory -Path $portable | Out-Null
Copy-Item -LiteralPath (Join-Path $release "lumina.exe") -Destination (Join-Path $portable "Lumina.exe")
Copy-Item -LiteralPath (Join-Path $workspace "src-tauri\tools") -Destination $portable -Recurse
foreach($name in @("README.md","SPEC-0.3.md","TESTING.md","TEST-REPORT-0.3.md","TRACEABILITY-0.3.md","THIRD-PARTY-NOTICES.md")){Copy-Item -LiteralPath (Join-Path $workspace $name) -Destination $portable}
$manifest=foreach($file in Get-ChildItem -LiteralPath $portable -Recurse -File|Sort-Object FullName){[pscustomobject]@{path=$file.FullName.Substring($portable.Length+1).Replace("\","/");bytes=$file.Length;sha256=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}}
$manifest|ConvertTo-Json -Depth 3|Set-Content -LiteralPath (Join-Path $portable "MANIFEST.json") -Encoding utf8
if(Test-Path -LiteralPath $zip){Remove-Item -LiteralPath $zip -Force};Compress-Archive -LiteralPath $portable -DestinationPath $zip -CompressionLevel Optimal
Get-Item -LiteralPath $zip
