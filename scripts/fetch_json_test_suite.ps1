& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "nst/JSONTestSuite" `
    -RepoUrl "https://github.com/nst/JSONTestSuite.git" `
    -ZipUrl "https://github.com/nst/JSONTestSuite/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/json/tests" `
    -SuiteDirName "JSONTestSuite" `
    -MarkerFile "test_parsing"
