# *S*tupid *F*ast *S*coop *U*tils

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jewlexx/sfsu/build.yml)
[![dependency status](https://deps.rs/repo/github/jewlexx/sfsu/status.svg)](https://deps.rs/repo/github/jewlexx/sfsu)
![GitHub all releases](https://img.shields.io/github/downloads/jewlexx/sfsu/total)
![GitHub](https://img.shields.io/github/license/jewlexx/sfsu)
![Scoop Version (extras bucket)](https://img.shields.io/scoop/v/sfsu?bucket=extras)

> [!WARNING]
> This is still under development. It currently does not replace even close to all the scoop commands, and is missing a lot of functionality.
> There is unlikely to be any breaking changes, so there isn't much harm using it as is.

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

Add the following to your .bashrc (or its equivalents, i.e .zshrc) file

```bash
source <(sfsu.exe hook --shell bash)
```

Nushell is also supported. Run the following command save it to a file.

```sh
sfsu hook --shell nu | save -f path/to/some/file.nu
```

Then source it in your `config.nu` (situated in path `$nu.config-path`).

```sh
source path/to/the/file.nu
```

The above disable demonstration also works

## Benchmarks [^1]

On average, `sfsu search` is **~500** times faster than regular `scoop search` and **~6.5** times faster than [scoop-search](https://github.com/shilangyu/scoop-search).

That takes you from about 25 seconds on `scoop search` down to `50 milliseconds` with `sfsu search`

`sfsu list` is **~30** times faster than `scoop list`

`sfsu` and `hok` are pretty comperable in all benchmarks. `sfsu` wins the search and info benchmarks but `hok` wins the list benchmark, all by an insignificant margin.

Done on a *AMD Ryzen 7 2700X @ 4.3GHz* with *16GB* of RAM and 11 scoop buckets listed below

### Searching [^search-version]

```shell
$ hyperfine --warmup 3 'sfsu search google' 'hok search google' 'scoop-search google' 'scoop search google'

Benchmark 1: sfsu search google
  Time (mean ± σ):      64.0 ms ±   5.8 ms    [User: 4.6 ms, System: 11.4 ms]
  Range (min … max):    56.1 ms …  85.1 ms    36 runs

Benchmark 2: hok search google
  Time (mean ± σ):      81.9 ms ±  12.9 ms    [User: 12.1 ms, System: 11.6 ms]
  Range (min … max):    72.5 ms … 115.9 ms    34 runs

Benchmark 3: scoop-search google
  Time (mean ± σ):     315.1 ms ±   9.7 ms    [User: 16.7 ms, System: 75.9 ms]
  Range (min … max):   303.6 ms … 330.5 ms    10 runs

Benchmark 4: scoop search google
  Time (mean ± σ):     21.872 s ±  1.206 s    [User: 5.002 s, System: 8.509 s]
  Range (min … max):   20.383 s … 24.293 s    10 runs

Summary
  sfsu search google ran
    1.28 ± 0.23 times faster than hok search google
    4.92 ± 0.47 times faster than scoop-search google
  341.59 ± 36.14 times faster than scoop search google
```

### Listing [^list-version]

```shell
$ hyperfine --warmup 3 'sfsu list' 'hok list' 'scoop list'

Benchmark 1: sfsu list
  Time (mean ± σ):      86.8 ms ±   2.7 ms    [User: 1.6 ms, System: 16.5 ms]
  Range (min … max):    82.8 ms …  93.6 ms    28 runs

Benchmark 2: hok list
  Time (mean ± σ):      85.1 ms ±   6.6 ms    [User: 13.8 ms, System: 71.5 ms]
  Range (min … max):    77.8 ms …  99.7 ms    32 runs

Benchmark 3: scoop list
  Time (mean ± σ):      2.512 s ±  0.092 s    [User: 1.013 s, System: 0.475 s]
  Range (min … max):    2.423 s …  2.728 s    10 runs

  Warning: Statistical outliers were detected. Consider re-running this benchmark on a quiet system without any interferences from other programs. It might help to use the '--warmup' or '--prepare' options.

Summary
  hok list ran
    1.02 ± 0.09 times faster than sfsu list
   29.53 ± 2.52 times faster than scoop list
```

### Info [^info-version]

```shell
$ hyperfine --warmup 3 'sfsu describe sfsu' 'hok info sfsu' 'scoop info sfsu'
Benchmark 1: sfsu describe sfsu
  Time (mean ± σ):      54.8 ms ±   2.3 ms    [User: 0.7 ms, System: 3.6 ms]
  Range (min … max):    51.2 ms …  60.5 ms    44 runs

Benchmark 2: hok info sfsu
  Time (mean ± σ):      74.3 ms ±   2.3 ms    [User: 5.4 ms, System: 5.4 ms]
  Range (min … max):    69.7 ms …  79.7 ms    35 runs

Benchmark 3: scoop info sfsu
  Time (mean ± σ):      1.420 s ±  0.024 s    [User: 0.405 s, System: 0.119 s]
  Range (min … max):    1.389 s …  1.457 s    10 runs

Summary
  sfsu describe sfsu ran
    1.36 ± 0.07 times faster than hok info sfsu
   25.94 ± 1.18 times faster than scoop info sfsu
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
nirsoft      https://github.com/kodybrown/scoop-nirsoft
personal     https://github.com/jewlexx/personal-scoop.git
spotify      https://github.com/TheRandomLabs/Scoop-Spotify.git
sysinternals https://github.com/niheaven/scoop-sysinternals.git
versions     https://github.com/ScoopInstaller/Versions
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

[^1]: These benchmarks are done after warmups. You will likely see far greater improvements when run on "cold" systems. Results will also differ depending on search request and the number of results, as well as installed buckets, and a few other factors

[^search-version]: Run on sfsu version [v1.4.10][v1.4.10], Hok version [v0.1.0-beta.3][hokv0.1.0-beta.3], scoop-search version [1.3.1](https://github.com/shilangyu/scoop-search/releases/tag/v1.3.1)
[^info-version]: Run on sfsu version [v1.4.10][v1.4.10] and Hok version [v0.1.0-beta.3][hokv0.1.0-beta.3]
[^list-version]: Run on sfsu version [v1.4.10][v1.4.10] and Hok version [v0.1.0-beta.3][hokv0.1.0-beta.3]

[v1.4.10]: https://github.com/jewlexx/sfsu/releases/tag/v1.4.10
[hokv0.1.0-beta.3]: https://github.com/chawyehsu/hok/releases/tag/v0.1.0-beta.3
