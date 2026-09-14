# Fetch official starfederation/ron (RON Conformance Test Suite)
# URL: https://github.com/starfederation/ron

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\ron\tests"
$suiteDir = Join-Path $targetDir "ron-upstream"
$repoUrl = "https://github.com/starfederation/ron.git"
$zipUrl = "https://github.com/starfederation/ron/archive/refs/heads/main.zip"
$zipPath = Join-Path $targetDir "ron-upstream-main.zip"

Write-Host "=== starfederation/ron Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "testdata\conformance\manifest.json")) {
    Write-Host "[OK] starfederation/ron already present at: $suiteDir" -ForegroundColor Green
    exit 0
}

# Ensure parent tests directory exists
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$gitAvailable = $false
try {
    $null = git --version
    $gitAvailable = $true
} catch {
    $gitAvailable = $false
}

if ($gitAvailable) {
    Write-Host "Cloning $repoUrl via git..." -ForegroundColor Yellow
    git clone --depth 1 $repoUrl $suiteDir
} else {
    Write-Host "git not found. Downloading archive from $zipUrl ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -UseBasicParsing

    Write-Host "Extracting to $targetDir ..." -ForegroundColor Yellow
    Expand-Archive -Path $zipPath -DestinationPath $targetDir -Force

    $extractedFolder = Join-Path $targetDir "ron-main"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "ron-upstream"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "testdata\conformance\manifest.json")) {
    Write-Host "[SUCCESS] starfederation/ron successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] manifest.json was not found at expected location: $suiteDir" -ForegroundColor Red
}
