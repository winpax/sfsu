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

    Write-Output "$ hyperfine --warmup 5 $($commands | ForEach-Object { "'$($_)'" })" > $FileName
    hyperfine --warmup 5 $Commands >> $FileName
}

# Set current branch to develop
scoop config scoop_branch develop
pwsh -NoProfile -C 'scoop update'

# Downloading
Write-Output 'Benchmarking downloads'
BenchCommands @('cargo r -r download sfsu', 'scoop download sfsu')

Write-Output 'Benchmarking downloads with versions'
BenchCommands @('cargo r -r download sfsu@1.10.2', 'scoop download sfsu@1.10.2') -OutFile 'download_versions'

# Listing
Write-Output 'Benchmarking listing commands'
BenchCommands @('sfsu list', 'hok list', 'scoop list')

# Searching
## Without sqlite cache
Write-Output 'Benchmarking search without cache'
scoop config use_sqlite_cache false
BenchCommands @('sfsu search google', 'hok search google', 'scoop-search google', 'scoop search google')

## With sqlite cache
scoop config use_sqlite_cache true
Write-Output 'Benchmarking search with cache'
BenchCommands @('sfsu search google', 'scoop search google') -OutFile 'search_cached'

# Info
Write-Output 'Benchmarking info'
BenchCommands @('sfsu info sfsu --disable-updated', 'hok info sfsu') -OutFile 'info_hok'
BenchCommands @('sfsu info sfsu', 'scoop info sfsu')
