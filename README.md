# *S*tupid *F*ast *S*coop *S*earch

Super fast `scoop search` replacement written in Rust

## Installation

```powershell
scoop bucket add extras

scoop install sfss
```

## Hook

You may set up the hook to use the `scoop search` command normally and have it use `sfss` instead

Add the following to your Powershell profile

```powershell
Invoke-Expression (&scoop-search --hook)
```

## Benchmarks

Done on a _AMD Ryzen 7 2700X @ 4.3GHz_ with _16GB_ of RAM and 17 scoop buckets listed below

### Benchmark Results

### Scoop Buckets

```powershell
dorado      https://github.com/chawyehsu/dorado
extras      https://github.com/ScoopInstaller/Extras
games       ~\scoop\buckets\games
java        https://github.com/ScoopInstaller/Java
lemon       https://github.com/hoilc/scoop-lemon
main        https://github.com/ScoopInstaller/Main
nerd-fonts  https://github.com/matthewjberger/scoop-nerd-fonts
nirsoft     https://github.com/kodybrown/scoop-nirsoft
nonportable https://github.com/ScoopInstaller/Nonportable
personal    https://github.com/jewlexx/personal-scoop.git
php         https://github.com/ScoopInstaller/PHP
python      https://github.com/TheRandomLabs/Scoop-Python.git
random      https://github.com/TheRandomLabs/Scoop-Bucket.git
scoopet     https://github.com/ivaquero/scoopet
spotify     https://github.com/TheRandomLabs/Scoop-Spotify.git
versions    https://github.com/ScoopInstaller/Versions
wsl         https://github.com/KNOXDEV/wsl
```

**Made with 💗 by Juliette Cordor**

```

```
