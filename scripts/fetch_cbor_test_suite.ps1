& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "cbor/test-vectors" `
    -RepoUrl "https://github.com/cbor/test-vectors.git" `
    -ZipUrl "https://github.com/cbor/test-vectors/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/cbor/tests" `
    -SuiteDirName "test-vectors" `
    -MarkerFile "appendix_a.json"
