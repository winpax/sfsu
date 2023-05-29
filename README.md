# *S*tupid *F*ast *S*coop *U*tils

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jewlexx/sfsu/build.yml)
[![dependency status](https://deps.rs/repo/github/jewlexx/sfsu/status.svg)](https://deps.rs/repo/github/jewlexx/sfsu)
![GitHub all releases](https://img.shields.io/github/downloads/jewlexx/sfsu/total)
![GitHub](https://img.shields.io/github/license/jewlexx/sfsu)
![Scoop Version (extras bucket)](https://img.shields.io/scoop/v/sfsu?bucket=extras)

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

You can also optionally disable certain hooks via the `--disable <COMMAND>` flag

```powershell
Invoke-Expression (&sfsu hook --disable list)
```

It also supports Bash in WSL and MSYS2

Add the following to your .bashrc (or its derivatives, i.e zsh) configuration

```bash
source <(sfsu.exe hook --nix)
```

The above disable demonstration also works

## Benchmarks [^1]

On average, `sfsu search` is **~500** times faster than regular `scoop search` and **~6.5** times faster than [scoop-search](https://github.com/shilangyu/scoop-search).

That takes you from about 25 seconds on `scoop search` down to `50 milliseconds` with `sfsu search`

`sfsu list` is **~30** times faster than `scoop list`

Done on a *AMD Ryzen 7 2700X @ 4.3GHz* with *16GB* of RAM and 17 scoop buckets listed below

[^1]: These benchmarks are done after warmups. You will likely see far greater improvements when run on "cold" systems. Results will also differ depending on search request and the number of results, as well as installed buckets, and a few other factors

### Searching [^search-version]

```shell
$ hyperfine --warmup 3 'sfsu search google' 'scoop-search google' 'scoop search google'

Benchmark 1: sfsu search google
  Time (mean ± σ):      55.3 ms ±   5.3 ms    [User: 3.6 ms, System: 3.9 ms]
  Range (min … max):    47.8 ms …  73.0 ms    26 runs

Benchmark 2: scoop-search google
  Time (mean ± σ):     342.1 ms ±  24.6 ms    [User: 15.6 ms, System: 130.0 ms]
  Range (min … max):   316.8 ms … 384.0 ms    10 runs

Benchmark 3: scoop search google
  Time (mean ± σ):     25.794 s ±  2.048 s    [User: 5.261 s, System: 10.469 s]
  Range (min … max):   24.352 s … 31.234 s    10 runs

Summary
  'sfsu search google' ran
    6.19 ± 0.74 times faster than 'scoop-search google'
  466.67 ± 57.90 times faster than 'scoop search google'
```

[^search-version]: Run on version [v1.4.0][v1.4.0]

### Listing [^list-version]

```shell
$ hyperfine --warmup 3 'sfsu list' 'scoop list'

Benchmark 1: sfsu list
  Time (mean ± σ):      72.3 ms ±   8.5 ms    [User: 0.0 ms, System: 13.1 ms]
  Range (min … max):    64.3 ms … 110.3 ms    27 runs

Benchmark 2: scoop list
  Time (mean ± σ):      2.128 s ±  0.030 s    [User: 0.634 s, System: 0.301 s]
  Range (min … max):    2.090 s …  2.182 s    10 runs

Summary
  'sfsu list' ran
   29.43 ± 3.50 times faster than 'scoop list'
```

[^list-version]: Run on version [v1.4.0][v1.4.0]

[v1.4.0]: https://github.com/jewlexx/sfsu/releases/tag/v1.4.0

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

I have a couple of long term goals.

Firstly, I want to create a Rust library to help interacting with [Scoop](https://scoop.sh) from code. This library would allow for things like installing packages, running updates, etc.
It will likely start by providing a function to get the Scoop install path, but hopefully over time it will grow into a fully fledged library, which is used internally by sfsu to interact with Scoop.

My other long term goal is to create a Scoop replacement for those who want it, in a similar vein as [Shovel](https://github.com/Ash258/Scoop-Core). This is a fairly large undertaking and will definitely take me a lot of time, so this is a very long term goal, and may never happen. Despite this I never really plan to replace Scoop. It is a great package manager and if anything `sfsu` would just be a command you can run instead of Scoop, but would run on Scoop installations.

In the meantime I will continue working on this independently of Scoop as a collection of seperate tools that work in conjunction with Scoop.

<!-- markdownlint-disable-next-line MD036 -->
**Made with 💗 by Juliette Cordor**
