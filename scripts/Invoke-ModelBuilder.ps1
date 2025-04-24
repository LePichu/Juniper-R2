param(
    [Parameter(Mandatory=$true)]
    [string]$Name,

    [Parameter(Mandatory=$true)]
    [string]$Model
)

$TEMPL = Get-Content ../include/Template.modelfile
$RES = ""

Get-ChildItem ../include/data | % {
    $TXT = Get-Content $_ -Raw
    $RES += @"

$TXT
---

"@
}

Set-Content -Path "../include/$Name.build.modelfile" -Value (@"
$TEMPL

SYSTEM """
$RES 
"""
"@ -Replace "%%MODEL%%", "$Model")
