# Fetch official msgpack-test-suite (MessagePack Conformance Test Suite)
# URL: https://github.com/kawanet/msgpack-test-suite

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\msgpack\tests"
$suiteDir = Join-Path $targetDir "msgpack-test-suite"
$repoUrl = "https://github.com/kawanet/msgpack-test-suite.git"
$zipUrl = "https://github.com/kawanet/msgpack-test-suite/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "msgpack-test-suite-master.zip"

Write-Host "=== kawanet/msgpack-test-suite Conformance Suite Downloader ===" -ForegroundColor Cyan

if ((Test-Path (Join-Path $suiteDir "dist\msgpack-test-suite.json")) -or (Test-Path (Join-Path $suiteDir "src"))) {
    Write-Host "[OK] msgpack-test-suite already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "msgpack-test-suite-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "msgpack-test-suite"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if ((Test-Path (Join-Path $suiteDir "dist\msgpack-test-suite.json")) -or (Test-Path (Join-Path $suiteDir "src"))) {
    Write-Host "[SUCCESS] msgpack-test-suite successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] test files were not found at expected location: $suiteDir" -ForegroundColor Red
}
