# Fetch official Apache Parquet test suite (parquet-testing)
# URLs:
#   https://github.com/apache/parquet-testing.git (primary test files)
#   https://github.com/apache/parquet-format.git (format specification)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\parquet\tests"
$suiteDir = Join-Path $targetDir "parquet-testing"
$repoUrl = "https://github.com/apache/parquet-testing.git"
$zipUrl = "https://github.com/apache/parquet-testing/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "parquet-testing-master.zip"

Write-Host "=== Apache Parquet Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "data")) {
    Write-Host "[OK] parquet-testing already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "parquet-testing-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "parquet-testing"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "data")) {
    Write-Host "[SUCCESS] parquet-testing successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] data directory was not found at expected location: $suiteDir" -ForegroundColor Red
}
