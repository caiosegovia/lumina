$ErrorActionPreference = "Stop"
$workspace = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$zip = (Resolve-Path (Join-Path $workspace "artifacts\Lumina-0.2.0-portable-windows-x64.zip")).Path
$extract = Join-Path $env:TEMP ("Lumina-Portable-Smoke-" + [guid]::NewGuid().ToString("N"))

Expand-Archive -LiteralPath $zip -DestinationPath $extract
$root = Get-ChildItem -LiteralPath $extract -Directory | Select-Object -First 1
$manifest = Get-Content (Join-Path $root.FullName "MANIFEST.json") -Raw -Encoding UTF8 | ConvertFrom-Json
$bad = @()
foreach ($entry in $manifest) {
    $path = Join-Path $root.FullName ($entry.path.Replace("/", "\"))
    if (-not (Test-Path -LiteralPath $path)) { $bad += $entry.path; continue }
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $entry.sha256) { $bad += $entry.path }
}
if ($bad.Count) { throw "Manifesto inválido: $($bad -join ', ')" }

$data = Join-Path $extract "local-data"
New-Item -ItemType Directory -Path $data | Out-Null
$info = [System.Diagnostics.ProcessStartInfo]::new()
$info.FileName = Join-Path $root.FullName "Lumina.exe"
$info.UseShellExecute = $false
$info.CreateNoWindow = $true
$info.Environment["LOCALAPPDATA"] = $data
$info.Environment["Path"] = "$env:SystemRoot\System32;$env:SystemRoot"
$app = [System.Diagnostics.Process]::Start($info)
Start-Sleep -Seconds 5
if ($app.HasExited) { throw "Portátil encerrou com código $($app.ExitCode)" }
$app.Kill()
$app.WaitForExit()

$result = [pscustomobject]@{
    ManifestEntries = $manifest.Count
    PortableStayedRunning = $true
    ToolsBundled = (Test-Path (Join-Path $root.FullName "tools\ffmpeg.exe")) -and (Test-Path (Join-Path $root.FullName "tools\exiftool.exe"))
    ZipSha256 = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLowerInvariant()
}
$resolved = (Resolve-Path $extract).Path
$tempRoot = (Resolve-Path $env:TEMP).Path
if (-not $resolved.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Extração fora do TEMP; limpeza recusada"
}
Remove-Item -LiteralPath $resolved -Recurse -Force
$result
