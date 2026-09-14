& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "kawanet/msgpack-test-suite" `
    -RepoUrl "https://github.com/kawanet/msgpack-test-suite.git" `
    -ZipUrl "https://github.com/kawanet/msgpack-test-suite/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/msgpack/tests" `
    -SuiteDirName "msgpack-test-suite" `
    -MarkerFile "dist/msgpack-test-suite.json"
