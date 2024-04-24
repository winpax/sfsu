function BenchCommands
{
    param(
        [parameter(Mandatory = $true)]
        [String[]]$Commands,
        [parameter(Mandatory = $false)]
        [String]$OutDir,
        [parameter(Mandatory = $false)]
        [String]$OutFile
    )

    if (-Not $PSBoundParameters.ContainsKey('OutDir'))
    {
        $OutDir = "$PSScriptRoot/outputs"
        mkdir $OutDir -ErrorAction SilentlyContinue
    }

    if (-Not $PSBoundParameters.ContainsKey('OutFile'))
    {
        $OutFile = "$(($Commands[0] -split ' ')[1])"
    }
    $FileName = "$OutDir/$OutFile.txt"

    Write-Output "$ hyperfine --warmup 5 $commands" > $FileName
    hyperfine --warmup 5 $Commands >> $FileName
}

# Set current branch to develop
scoop config scoop_branch develop
pwsh -NoProfile -C 'scoop update'

# Listing
BenchCommands @('sfsu list', 'hok list', 'scoop list')

# Searching
## Without sqlite cache
scoop config use_sqlite_cache false
BenchCommands @('sfsu search google', 'hok search google', 'scoop-search google', 'scoop search google')

## With sqlite cache
scoop config use_sqlite_cache true
BenchCommands @('sfsu search google', 'scoop search google') -OutFile 'search_cached'

# Info
BenchCommands @('sfsu info sfsu --disable-updated', 'hok info sfsu')
BenchCommands @('sfsu info sfsu', 'scoop info sfsu')
