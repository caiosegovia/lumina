param(
    [switch]$Execute,
    [string]$Workspace = (Split-Path -Parent $PSScriptRoot)
)

# Default: preview only. Run with -Execute to remove regenerable build caches.
# Keeps release executables, installers, tools, runtime data and diagnostics.
$ErrorActionPreference = 'Stop'
$workspacePath = (Resolve-Path -LiteralPath $Workspace).Path.TrimEnd('\')
if (-not (Test-Path -LiteralPath (Join-Path $workspacePath 'src-tauri\Cargo.toml'))) {
    throw 'This is not the Lumina workspace.'
}
$prefix = $workspacePath + '\'
$candidates = [System.Collections.Generic.List[string]]::new()

function Assert-SafeDirectory([string]$Path) {
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    if (-not $resolved.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Path outside workspace: $resolved"
    }
    # Check every ancestor so a junction cannot redirect a lexical child path.
    $ancestor = Get-Item -LiteralPath $resolved -Force
    while ($null -ne $ancestor) {
        if ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw "Link or junction is not allowed: $($ancestor.FullName)"
        }
        $ancestor = $ancestor.Parent
    }
    return $resolved
}

function Add-CacheDirectory([string]$Path) {
    if (Test-Path -LiteralPath $Path -PathType Container) {
        $resolved = Assert-SafeDirectory $Path
        $candidates.Add($resolved)
    }
}

$buildRoots = @(Get-ChildItem -LiteralPath $workspacePath -Directory -Force |
    Where-Object Name -Like '.build-*')
foreach ($relative in @('src-tauri', 'src-tauri\src-tauri')) {
    $parent = Join-Path $workspacePath $relative
    if (Test-Path -LiteralPath $parent) {
        $null = Assert-SafeDirectory $parent
        $buildRoots += @(Get-ChildItem -LiteralPath $parent -Directory -Force |
            Where-Object { $_.Name -eq 'target' -or $_.Name -like 'target-*' })
    }
}
foreach ($root in $buildRoots) {
    $null = Assert-SafeDirectory $root.FullName
    Add-CacheDirectory (Join-Path $root.FullName 'debug')
    foreach ($name in @('deps', 'build', 'incremental', '.fingerprint', 'examples')) {
        Add-CacheDirectory (Join-Path $root.FullName "release\$name")
    }
}
foreach ($relative in @('dist', 'node_modules\.vite', 'node_modules\.vite-temp', 'node_modules\.cache')) {
    Add-CacheDirectory (Join-Path $workspacePath $relative)
}
$artifactsPath = Join-Path $workspacePath 'artifacts'
if (Test-Path -LiteralPath $artifactsPath) {
    $null = Assert-SafeDirectory $artifactsPath
    Get-ChildItem -LiteralPath $artifactsPath -Directory -Force |
        Where-Object { $_.Name -like '.staging-Lumina-*' -or $_.Name -like 'Lumina-*-portable-windows-x64-staging' } |
        ForEach-Object { Add-CacheDirectory $_.FullName }
}

$rows = foreach ($path in ($candidates | Sort-Object -Unique)) {
    $items = @(Get-ChildItem -LiteralPath $path -Recurse -Force)
    $protected = @($items | Where-Object {
        ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        $_.Name -eq '.lumina' -or $_.Name -eq 'library.json' -or
        $_.Name -like '*.sqlite*' -or $_.Name -like 'lumina-diagnostics-*'
    })
    if ($protected.Count -gt 0) {
        Write-Warning "Skipped: protected data or link in $path"
        continue
    }
    $bytes = ($items | Where-Object { -not $_.PSIsContainer } | Measure-Object Length -Sum).Sum
    [pscustomobject]@{Path=$path; Bytes=[long]$bytes; GiB=[math]::Round($bytes/1GB,2)}
}
$rows | Select-Object Path,GiB | Format-Table -AutoSize
$total = ($rows | Measure-Object Bytes -Sum).Sum
Write-Host ('Regenerable files: {0:N2} GiB' -f ($total/1GB))
if (-not $Execute) {
    Write-Host 'Preview only. Run with -Execute to delete these directories.'
    return
}
if (Get-Process -Name cargo,rustc,clippy-driver,lumina,makensis,light,candle -ErrorAction SilentlyContinue) {
    throw 'Close Lumina and wait for builds to finish before cleaning.'
}
$drive = [IO.DriveInfo]::new([IO.Path]::GetPathRoot($workspacePath))
$freeBefore = $drive.AvailableFreeSpace
foreach ($row in $rows) {
    $resolved = Assert-SafeDirectory $row.Path
    Write-Host "Removing $resolved"
    Remove-Item -LiteralPath $resolved -Recurse -Force -ErrorAction Stop
}
$freeAfter = ([IO.DriveInfo]::new($drive.Name)).AvailableFreeSpace
Write-Host ('Recovered: {0:N2} GiB; free: {1:N2} GiB' -f (($freeAfter-$freeBefore)/1GB),($freeAfter/1GB))
