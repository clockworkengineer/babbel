& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "apache/parquet-testing" `
    -RepoUrl "https://github.com/apache/parquet-testing.git" `
    -ZipUrl "https://github.com/apache/parquet-testing/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/parquet/tests" `
    -SuiteDirName "parquet-testing" `
    -MarkerFile "data"
