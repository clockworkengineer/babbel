& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "kdl-org/kdl-test" `
    -RepoUrl "https://github.com/kdl-org/kdl-test.git" `
    -ZipUrl "https://github.com/kdl-org/kdl-test/archive/refs/heads/main.zip" `
    -TargetRelativeDir "crates/kdl/tests" `
    -SuiteDirName "kdl-test" `
    -MarkerFile "test_cases"
