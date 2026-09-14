# Fetch official cbor/test-vectors (CBOR RFC 7049 Appendix A Conformance Suite)
# URL: https://github.com/cbor/test-vectors

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\cbor\tests"
$suiteDir = Join-Path $targetDir "test-vectors"
$repoUrl = "https://github.com/cbor/test-vectors.git"
$zipUrl = "https://github.com/cbor/test-vectors/archive/refs/heads/master.zip"
$zipPath = Join-Path $targetDir "test-vectors-master.zip"

Write-Host "=== cbor/test-vectors Conformance Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $suiteDir "appendix_a.json")) {
    Write-Host "[OK] test-vectors already present at: $suiteDir" -ForegroundColor Green
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

    $extractedFolder = Join-Path $targetDir "test-vectors-master"
    if (Test-Path $extractedFolder) {
        Rename-Item -Path $extractedFolder -NewName "test-vectors"
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path (Join-Path $suiteDir "appendix_a.json")) {
    Write-Host "[SUCCESS] test-vectors successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] appendix_a.json was not found at expected location: $suiteDir" -ForegroundColor Red
}
