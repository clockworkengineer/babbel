& "$PSScriptRoot/fetch_suite_core.ps1" `
    -Name "W3C XML Conformance Test Suite (XML TS 20130923)" `
    -RepoUrl "" `
    -ZipUrl "https://www.w3.org/XML/Test/xmlts20130923.zip" `
    -TargetRelativeDir "crates/xml/tests" `
    -SuiteDirName "xmlconf" `
    -MarkerFile "xmlconf.xml"
