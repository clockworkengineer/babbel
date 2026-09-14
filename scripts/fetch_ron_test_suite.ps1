& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "starfederation/ron" `
    -RepoUrl "https://github.com/starfederation/ron.git" `
    -ZipUrl "https://github.com/starfederation/ron/archive/refs/heads/main.zip" `
    -TargetRelativeDir "crates/ron/tests" `
    -SuiteDirName "ron-upstream" `
    -MarkerFile "testdata/conformance/manifest.json"
