# Master Conformance Test Suite Downloader Dispatcher (PowerShell)
param(
    [Parameter(Mandatory=$false)][string]$Format = "all"
)

$ErrorActionPreference = "Stop"

$suites = @(
    @{
        Format = "json"
        Script = "fetch_json_test_suite.ps1"
    },
    @{
        Format = "toml"
        Script = "fetch_toml_test_suite.ps1"
    },
    @{
        Format = "xml"
        Script = "fetch_w3c_xmlts.ps1"
    },
    @{
        Format = "cbor"
        Script = "fetch_cbor_test_suite.ps1"
    },
    @{
        Format = "bson"
        Script = "fetch_bson_test_suite.ps1"
    },
    @{
        Format = "msgpack"
        Script = "fetch_msgpack_test_suite.ps1"
    },
    @{
        Format = "ron"
        Script = "fetch_ron_test_suite.ps1"
    },
    @{
        Format = "kdl"
        Script = "fetch_kdl_test_suite.ps1"
    },
    @{
        Format = "parquet"
        Script = "fetch_parquet_test_suite.ps1"
    },
    @{
        Format = "hcl"
        Script = "fetch_hcl_test_suite.ps1"
    }
)

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " Babbel Conformance Test Suite Master Downloader" -ForegroundColor Cyan
Write-Host " Target Filter: $Format" -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$selected = $suites | Where-Object { $Format -eq "all" -or $_.Format -eq $Format }

if ($selected.Count -eq 0) {
    Write-Host "No test suites match format '$Format'. Available: json, toml, xml, cbor, bson, msgpack, ron, kdl, parquet, hcl, all" -ForegroundColor Red
    exit 1
}

foreach ($s in $selected) {
    $scriptPath = Join-Path $PSScriptRoot $s.Script
    if (Test-Path $scriptPath) {
        & $scriptPath
        Write-Host ""
    } else {
        Write-Host "Missing script: $scriptPath" -ForegroundColor Red
    }
}

Write-Host "All requested conformance test suites checked/downloaded." -ForegroundColor Green
