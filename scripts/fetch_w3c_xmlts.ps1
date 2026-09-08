# Fetch official W3C XML Conformance Test Suite (XML TS 20130923)
# URL: https://www.w3.org/XML/Test/

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$targetDir = Join-Path $rootDir "crates\xml\tests"
$zipPath = Join-Path $targetDir "xmlts20130923.zip"
$xmlconfDir = Join-Path $targetDir "xmlconf"
$url = "https://www.w3.org/XML/Test/xmlts20130923.zip"

Write-Host "=== W3C XML Conformance Test Suite Downloader ===" -ForegroundColor Cyan

if (Test-Path (Join-Path $xmlconfDir "xmlconf.xml")) {
    Write-Host "[OK] W3C XML test suite already present at: $xmlconfDir" -ForegroundColor Green
    exit 0
}

Write-Host "Downloading $url ..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

Write-Host "Extracting to $targetDir ..." -ForegroundColor Yellow
Expand-Archive -Path $zipPath -DestinationPath $targetDir -Force

# Clean up zip
if (Test-Path $zipPath) {
    Remove-Item $zipPath -Force
}

if (Test-Path (Join-Path $xmlconfDir "xmlconf.xml")) {
    Write-Host "[SUCCESS] W3C XML Conformance Test Suite successfully installed at: $xmlconfDir" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Archive extracted, but xmlconf.xml was not found at expected location: $xmlconfDir" -ForegroundColor Red
}
