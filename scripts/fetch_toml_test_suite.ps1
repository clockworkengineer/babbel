& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "skystrife/toml-test" `
    -RepoUrl "https://github.com/skystrife/toml-test.git" `
    -ZipUrl "https://github.com/skystrife/toml-test/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/toml/tests" `
    -SuiteDirName "toml-test" `
    -MarkerFile "tests"
