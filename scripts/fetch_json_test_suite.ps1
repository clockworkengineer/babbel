# Fetch official JSONTestSuite (RFC 8259 Conformance Test Suite)
# URL: https://github.com/nst/JSONTestSuite

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\json\tests"
$suiteDir = Join-Path $targetDir "JSONTestSuite"
$repoUrl = "https://github.com/nst/JSONTestSuite.git"
$zipUrl = "https://github.com/nst/JSONTestSuite/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "JSONTestSuite-master.zip"

Write-Host "=== nst/JSONTestSuite Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "test_parsing")) {
    Write-Host "[OK] JSONTestSuite already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "JSONTestSuite-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "JSONTestSuite"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "test_parsing")) {
    Write-Host "[SUCCESS] JSONTestSuite successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] test_parsing was not found at expected location: $suiteDir" -ForegroundColor Red
}
