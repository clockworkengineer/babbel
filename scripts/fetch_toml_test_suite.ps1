# Fetch official toml-test (TOML v1.0.0 Conformance Test Suite)
# URL: https://github.com/toml-lang/toml-test

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\toml\tests"
$suiteDir = Join-Path $targetDir "toml-test"
$repoUrl = "https://github.com/toml-lang/toml-test.git"
$zipUrl = "https://github.com/toml-lang/toml-test/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "toml-test-master.zip"

Write-Host "=== toml-lang/toml-test Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "tests")) {
    Write-Host "[OK] toml-test already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "toml-test-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "toml-test"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "tests")) {
    Write-Host "[SUCCESS] toml-test successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] tests directory was not found at expected location: $suiteDir" -ForegroundColor Red
}
