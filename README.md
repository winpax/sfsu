# *S*tupid *F*ast *S*coop *U*tils

<!-- TODO: Add version numbers for the benchmarks run, and re-run them on latest versions -->

Super fast replacements and additions to scoop commands written in Rust

## Installation

```powershell
scoop bucket add extras

scoop install sfsu
```

## Hook

You may set up the hooks to use the scoop commands normally

Add the following to your Powershell profile

```powershell
Invoke-Expression (&sfsu hook)
```

You can also optionally disable certain hooks via the `--no-<hook>` flag

```powershell
Invoke-Expression (&sfsu hook --no-list)
```

## Benchmarks

On average, `sfsu search` is **~1200** times faster than regular `scoop search` and ~7 times faster than [scoop-search](https://github.com/shilangyu/scoop-search)

`sfsu list` is ~4 times faster than `scoop list`

Done on a *AMD Ryzen 7 2700X @ 4.3GHz* with *16GB* of RAM and 17 scoop buckets listed below

### Searching

```shell
$ hyperfine --warmup 1 'sfsu search google' 'scoop-search google' 'scoop search google'

Benchmark 1: sfsu search google
  Time (mean ± σ):      30.8 ms ±   2.8 ms    [User: 4.0 ms, System: 4.2 ms]
  Range (min … max):    26.6 ms …  40.8 ms    70 runs

Benchmark 2: scoop-search google
  Time (mean ± σ):     232.8 ms ±   9.6 ms    [User: 11.7 ms, System: 72.9 ms]
  Range (min … max):   218.5 ms … 251.7 ms    12 runs

Benchmark 3: scoop search google
  Time (mean ± σ):     38.186 s ±  0.673 s    [User: 5.330 s, System: 14.492 s]
  Range (min … max):   37.182 s … 39.419 s    10 runs

Summary
  'sfsu search google' ran
    7.56 ± 0.75 times faster than 'scoop-search google'
 1239.47 ± 114.54 times faster than 'scoop search google'
```

### Listing

```shell
$ hyperfine --warmup 1 'sfsu list' 'scoop list'

Benchmark 1: sfsu list
  Time (mean ± σ):     396.3 ms ±  26.3 ms    [User: 21.9 ms, System: 45.3 ms]
  Range (min … max):   359.6 ms … 435.1 ms    10 runs

Benchmark 2: scoop list
  Time (mean ± σ):      1.541 s ±  0.015 s    [User: 0.473 s, System: 0.253 s]
  Range (min … max):    1.518 s …  1.569 s    10 runs

Summary
  'sfsu list' ran
    3.89 ± 0.26 times faster than 'scoop list'
```

### Scoop Buckets

<!-- markdownlint-disable-next-line MD040 -->
```
dorado       https://github.com/chawyehsu/dorado
emulators    https://github.com/borger/scoop-emulators.git
extras       https://github.com/ScoopInstaller/Extras
games        https://github.com/Calinou/scoop-games
java         https://github.com/ScoopInstaller/Java
lemon        https://github.com/hoilc/scoop-lemon
main         https://github.com/ScoopInstaller/Main
nerd-fonts   https://github.com/matthewjberger/scoop-nerd-fonts
nirsoft      https://github.com/kodybrown/scoop-nirsoft
nonportable  https://github.com/ScoopInstaller/Nonportable
personal     https://github.com/jewlexx/personal-scoop.git
php          https://github.com/ScoopInstaller/PHP
python       https://github.com/TheRandomLabs/Scoop-Python.git
random       https://github.com/TheRandomLabs/Scoop-Bucket.git
scoopet      https://github.com/ivaquero/scoopet
spotify      https://github.com/TheRandomLabs/Scoop-Spotify.git
sysinternals https://github.com/niheaven/scoop-sysinternals.git
versions     https://github.com/ScoopInstaller/Versions
wsl          https://github.com/KNOXDEV/wsl
```

## Building yourself

The build instructions can be found [in the wiki](https://github.com/jewlexx/sfsu/wiki/Building)

## Long Term Goals

Currently I am considering creating an entire Scoop alternative that has 100% interoperability with existing Scoop buckets, but way way way faster than Scoop.

In the meantime I will continue working on this independently of Scoop as "seperate" tools that work without an entire package manager.

<!-- markdownlint-disable-next-line MD036 -->
**Made with 💗 by Juliette Cordor**
