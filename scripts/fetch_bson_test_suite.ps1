& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "mpaland/bsonfy" `
    -RepoUrl "https://github.com/mpaland/bsonfy.git" `
    -ZipUrl "https://github.com/mpaland/bsonfy/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/bson/tests" `
    -SuiteDirName "bsonfy" `
    -MarkerFile "test/spec/bson_test.ts"
