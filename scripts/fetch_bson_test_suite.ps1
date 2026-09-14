# Fetch official mpaland/bsonfy (BSON Conformance Test Suite)
# URL: https://github.com/mpaland/bsonfy

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\bson\tests"
$suiteDir = Join-Path $targetDir "bsonfy"
$repoUrl = "https://github.com/mpaland/bsonfy.git"
$zipUrl = "https://github.com/mpaland/bsonfy/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "bsonfy-master.zip"

Write-Host "=== mpaland/bsonfy Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "test\spec\bson_test.ts")) {
    Write-Host "[OK] bsonfy already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "bsonfy-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "bsonfy"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "test\spec\bson_test.ts")) {
    Write-Host "[SUCCESS] bsonfy successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] bson_test.ts was not found at expected location: $suiteDir" -ForegroundColor Red
}
