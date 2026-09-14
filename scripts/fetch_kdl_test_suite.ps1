# Fetch official kdl-org/kdl-test (KDL Conformance Test Suite)
# URL: https://github.com/kdl-org/kdl-test

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\kdl\tests"
$suiteDir = Join-Path $targetDir "kdl-test"
$repoUrl = "https://github.com/kdl-org/kdl-test.git"
$zipUrl = "https://github.com/kdl-org/kdl-test/archive/refs/heads/main.zip"
$zipPath = Join-Path $targetDir "kdl-test-main.zip"

Write-Host "=== kdl-org/kdl-test Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "test_cases")) {
    Write-Host "[OK] kdl-org/kdl-test already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "kdl-test-main"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "kdl-test"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "test_cases")) {
    Write-Host "[SUCCESS] kdl-org/kdl-test successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] test_cases was not found at expected location: $suiteDir" -ForegroundColor Red
}
