# *S*tupid *F*ast *S*coop *U*tils

Super fast replacements for scoop commands written in Rust

## Installation

```powershell
scoop bucket add extras

scoop install sfss
```

## Hook

You may set up the hooks to use the scoop commands normally

Add the following to your Powershell profile

```powershell
Invoke-Expression (&sfst-hook)
```

You can also optionally disable certain hooks via the `--no-<hook>` flag

```powershell
Invoke-Expression (&sfst-hook --no-list)
```

## Benchmarks

On average, `sfss` is **~1200** times faster than regular `scoop search` and ~7 times faster than [scoop-search](https://github.com/shilangyu/scoop-search)

`sfsl` is ~4 times faster than `scoop list`

Done on a _AMD Ryzen 7 2700X @ 4.3GHz_ with _16GB_ of RAM and 17 scoop buckets listed below

### Benchmark Results

#### SFSS Benchmarks

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

#### SFSL Benchmarks

```powershell
❯ hyperfine --warmup 1 'sfsl' 'scoop list'
Benchmark 1: sfsl
  Time (mean ± σ):     396.3 ms ±  26.3 ms    [User: 21.9 ms, System: 45.3 ms]
  Range (min … max):   359.6 ms … 435.1 ms    10 runs

Benchmark 2: scoop list
  Time (mean ± σ):      1.541 s ±  0.015 s    [User: 0.473 s, System: 0.253 s]
  Range (min … max):    1.518 s …  1.569 s    10 runs

Summary
  'sfsl' ran
    3.89 ± 0.26 times faster than 'scoop list'
```

### Scoop Buckets

```powershell
dorado      https://github.com/chawyehsu/dorado
extras      https://github.com/ScoopInstaller/Extras
games       https://github.com/Calinou/scoop-games
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
