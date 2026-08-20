$ErrorActionPreference = "Stop"

$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$artifacts = Join-Path $workspace "artifacts"
$portable = Join-Path $artifacts "Lumina-0.2.0-portable-windows-x64"
$zip = Join-Path $artifacts "Lumina-0.2.0-portable-windows-x64.zip"
$release = Join-Path $workspace "src-tauri\target\release"

New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
if (Test-Path -LiteralPath $portable) {
    $resolved = (Resolve-Path -LiteralPath $portable).Path
    if (-not $resolved.StartsWith($artifacts, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Destino portátil fora da pasta de artefatos"
    }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
New-Item -ItemType Directory -Path $portable -Force | Out-Null

Copy-Item -LiteralPath (Join-Path $release "lumina.exe") -Destination (Join-Path $portable "Lumina.exe")
Copy-Item -LiteralPath (Join-Path $workspace "src-tauri\tools") -Destination $portable -Recurse
Copy-Item -LiteralPath (Join-Path $workspace "README.md") -Destination $portable
Copy-Item -LiteralPath (Join-Path $workspace "TESTING.md") -Destination $portable
Copy-Item -LiteralPath (Join-Path $workspace "TEST-REPORT-0.2.md") -Destination $portable
Copy-Item -LiteralPath (Join-Path $workspace "TRACEABILITY-0.2.md") -Destination $portable
Copy-Item -LiteralPath (Join-Path $workspace "THIRD-PARTY-NOTICES.md") -Destination $portable

$manifest = foreach ($file in Get-ChildItem -LiteralPath $portable -Recurse -File | Sort-Object FullName) {
    [pscustomobject]@{
        path = $file.FullName.Substring($portable.Length + 1).Replace("\", "/")
        bytes = $file.Length
        sha256 = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}
$manifest | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath (Join-Path $portable "MANIFEST.json") -Encoding utf8

if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
Compress-Archive -LiteralPath $portable -DestinationPath $zip -CompressionLevel Optimal

$releaseFiles = @(
    $zip,
    (Join-Path $release "bundle\msi\Lumina_0.2.0_x64_en-US.msi"),
    (Join-Path $release "bundle\nsis\Lumina_0.2.0_x64-setup.exe")
)
$releaseFiles | ForEach-Object {
    $item = Get-Item -LiteralPath $_
    "{0}  {1}" -f (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $item.Name
} | Set-Content -LiteralPath (Join-Path $artifacts "SHA256SUMS.txt") -Encoding utf8

Get-Item -LiteralPath $releaseFiles | Select-Object Name,Length,FullName
