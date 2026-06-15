# Tickeys Windows Release Packaging Script
# Usage: .\scripts\package-release.ps1

$ErrorActionPreference = "Stop"

$version = "1.0.2"
$releaseDir = "target\release"
$packageName = "tickeys-windows-v$version"
$packageDir = "target\$packageName"
$zipFile = "target\$packageName.zip"

Write-Host "Packaging Tickeys Windows v$version..." -ForegroundColor Cyan

# Clean previous package
if (Test-Path $packageDir) {
    Remove-Item -Recurse -Force $packageDir
}
if (Test-Path $zipFile) {
    Remove-Item -Force $zipFile
}

# Create package directory
New-Item -ItemType Directory -Path $packageDir -Force | Out-Null

# Copy files
Write-Host "Copying files..." -ForegroundColor Yellow

# Copy exe
Copy-Item "$releaseDir\tickeys-windows.exe" $packageDir
Write-Host "  + tickeys-windows.exe"

# Copy OpenAL32.dll
Copy-Item "resource\dll\OpenAL32.dll" $packageDir
Write-Host "  + OpenAL32.dll"

# Copy icon
Copy-Item "resource\icon.ico" $packageDir
Write-Host "  + icon.ico"

# Copy data directory (schemes and audio files)
Copy-Item -Recurse "resource\data" "$packageDir\data"
Write-Host "  + data/"

# Create zip archive
Write-Host "Creating zip archive..." -ForegroundColor Yellow
Compress-Archive -Path $packageDir -DestinationPath $zipFile

# Get file sizes
$zipSize = (Get-Item $zipFile).Length / 1MB
$exeSize = (Get-Item "$releaseDir\tickeys-windows.exe").Length / 1MB

Write-Host "`nPackaging complete!" -ForegroundColor Green
Write-Host "  Package: $zipFile" -ForegroundColor Cyan
Write-Host "  Zip size: $([math]::Round($zipSize, 2)) MB" -ForegroundColor Cyan
Write-Host "  Exe size: $([math]::Round($exeSize, 2)) MB" -ForegroundColor Cyan

# List package contents
Write-Host "`nPackage contents:" -ForegroundColor Yellow
Get-ChildItem -Recurse $packageDir | ForEach-Object {
    $relativePath = $_.FullName.Replace((Resolve-Path $packageDir).Path, "")
    if ($_.PSIsContainer) {
        Write-Host "  [DIR] $relativePath/"
    } else {
        $size = $_.Length / 1KB
        Write-Host "  [FILE] $relativePath ($([math]::Round($size, 1)) KB)"
    }
}
