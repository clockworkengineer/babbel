& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "kmoneil/hcl-test-suite" `
    -RepoUrl "https://github.com/kmoneil/hcl-test-suite.git" `
    -ZipUrl "https://github.com/kmoneil/hcl-test-suite/archive/refs/heads/main.zip" `
    -TargetRelativeDir "crates/hcl/tests" `
    -SuiteDirName "hcl-test-suite" `
    -MarkerFile "tests"
