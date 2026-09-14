# Parameterized Conformance Test Suite Downloader Engine (PowerShell)
param(
    [Parameter(Mandatory=$true)][string]$Name,
    [Parameter(Mandatory=$false)][string]$RepoUrl = "",
    [Parameter(Mandatory=$true)][string]$ZipUrl,
    [Parameter(Mandatory=$true)][string]$TargetRelativeDir,
    [Parameter(Mandatory=$true)][string]$SuiteDirName,
    [Parameter(Mandatory=$true)][string]$MarkerFile
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir $TargetRelativeDir
$suiteDir = Join-Path $targetDir $SuiteDirName
$zipPath = Join-Path $targetDir "$SuiteDirName-temp.zip"

Write-Host "=== $Name Conformance Suite Downloader ===" -ForegroundColor Cyan

$markerPath = Join-Path $suiteDir $MarkerFile
if (Test-Path $markerPath) {
    Write-Host "[OK] $Name already present at: $suiteDir" -ForegroundColor Green
    exit 0
}

# Ensure parent target directory exists
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$gitAvailable = $false
if (-not [string]::IsNullOrWhiteSpace($RepoUrl)) {
    try {
        $null = git --version
        $gitAvailable = $true
    } catch {
        $gitAvailable = $false
    }
}

if ($gitAvailable) {
    Write-Host "Cloning $RepoUrl via git..." -ForegroundColor Yellow
    git clone --depth 1 $RepoUrl $suiteDir
} else {
    Write-Host "Downloading archive from $ZipUrl ..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri $ZipUrl -OutFile $zipPath -UseBasicParsing

    Write-Host "Extracting to $targetDir ..." -ForegroundColor Yellow
    Expand-Archive -Path $zipPath -DestinationPath $targetDir -Force

    # If the zip extracted into a subfolder like SuiteDirName-master or SuiteDirName-main, rename it
    $candidates = @(
        Join-Path $targetDir "$SuiteDirName-master",
        Join-Path $targetDir "$SuiteDirName-main",
        Join-Path $targetDir (Split-Path -Leaf $RepoUrl).Replace(".git", "-master"),
        Join-Path $targetDir (Split-Path -Leaf $RepoUrl).Replace(".git", "-main")
    )
    foreach ($cand in $candidates) {
        if ((Test-Path $cand) -and ($cand -ne $suiteDir)) {
            Rename-Item -Path $cand -NewName $SuiteDirName
            break
        }
    }

    if (Test-Path $zipPath) {
        Remove-Item $zipPath -Force
    }
}

if (Test-Path $markerPath) {
    Write-Host "[SUCCESS] $Name successfully installed at: $suiteDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Marker '$MarkerFile' was not found at expected location: $suiteDir" -ForegroundColor Red
}
