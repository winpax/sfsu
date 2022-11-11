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
Invoke-Expression (&sfss --hook)
```

## Benchmarks

On average, `sfss` is ~7 times faster than [scoop-search](https://github.com/shilangyu/scoop-search) and **~1200** times faster than regular `scoop search`

Done on a _AMD Ryzen 7 2700X @ 4.3GHz_ with _16GB_ of RAM and 17 scoop buckets listed below

### Benchmark Results

```powershell
❯  hyperfine --warmup 1 'sfss google' 'scoop-search google' 'scoop search google'
Benchmark 1: sfss google
  Time (mean ± σ):      30.8 ms ±   2.8 ms    [User: 4.0 ms, System: 4.2 ms]
  Range (min … max):    26.6 ms …  40.8 ms    70 runs

Benchmark 2: scoop-search google
  Time (mean ± σ):     232.8 ms ±   9.6 ms    [User: 11.7 ms, System: 72.9 ms]
  Range (min … max):   218.5 ms … 251.7 ms    12 runs

Benchmark 3: scoop search google
  Time (mean ± σ):     38.186 s ±  0.673 s    [User: 5.330 s, System: 14.492 s]
  Range (min … max):   37.182 s … 39.419 s    10 runs

Summary
  'sfss google' ran
    7.56 ± 0.75 times faster than 'scoop-search google'
 1239.47 ± 114.54 times faster than 'scoop search google'
```

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

