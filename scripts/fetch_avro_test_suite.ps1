& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "drnice/AvroTest" `
    -RepoUrl "https://github.com/drnice/AvroTest.git" `
    -ZipUrl "https://github.com/drnice/AvroTest/archive/refs/heads/master.zip" `
    -TargetRelativeDir "crates/avro/tests" `
    -SuiteDirName "avro-test-suite" `
    -MarkerFile "users2.avro"
