# *S*tupid *F*ast *S*coop *U*tils

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jewlexx/sfsu/build.yml)
[![dependency status](https://deps.rs/repo/github/jewlexx/sfsu/status.svg)](https://deps.rs/repo/github/jewlexx/sfsu)
![GitHub all releases](https://img.shields.io/github/downloads/jewlexx/sfsu/total)
![GitHub](https://img.shields.io/github/license/jewlexx/sfsu)
![Scoop Version (extras bucket)](https://img.shields.io/scoop/v/sfsu?bucket=extras)

> [!NOTE]
> This is still under development. It currently does not replace even close to all the scoop commands, and is missing a lot of functionality.
> There is unlikely to be any breaking changes, so there is likely no harm using it as is.

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

> [!NOTE]
> These benchmarks may not accurately represent the speeds on your system.
> While they do provide a relative measurement, your environment and results **will** be different.

On average, `sfsu search` is **~400** times faster than regular `scoop search` and **~5** times faster than [scoop-search](https://github.com/shilangyu/scoop-search).

`sfsu list` is **~35** times faster than `scoop list`

<!--TODO: A more detailed comparison of sfsu and hok-->

`sfsu` and `hok` are pretty comperable in all benchmarks. `sfsu` wins some benchmarks and `hok` wins others.

Done on a _AMD Ryzen 9 7900X @ 5.5GHz_ with _32GB_ of RAM at 5200MHz and 11 scoop buckets, as listed below

### Searching [^search-version]

```shell
$ hyperfine --warmup 5 'sfsu search google' 'hok search google' 'scoop-search google' 'scoop search google'
Benchmark 1: sfsu search google
  Time (mean ± σ):      59.1 ms ±   1.3 ms    [User: 0.0 ms, System: 2.6 ms]
  Range (min … max):    55.4 ms …  62.2 ms    36 runs

Benchmark 2: hok search google
  Time (mean ± σ):      67.7 ms ±   1.5 ms    [User: 3.3 ms, System: 1.9 ms]
  Range (min … max):    65.4 ms …  71.1 ms    33 runs

Benchmark 3: scoop-search google
  Time (mean ± σ):     292.5 ms ±   6.5 ms    [User: 14.1 ms, System: 79.7 ms]
  Range (min … max):   283.7 ms … 301.5 ms    10 runs

Benchmark 4: scoop search google
  Time (mean ± σ):      1.908 s ±  0.018 s    [User: 1.144 s, System: 0.419 s]
  Range (min … max):    1.894 s …  1.951 s    10 runs

Summary
  sfsu search google ran
    1.15 ± 0.04 times faster than hok search google
    4.95 ± 0.16 times faster than scoop-search google
   32.28 ± 0.79 times faster than scoop search google
```

#### With `Scoop` SQLite cache enabled, on the `develop` branch

```shell
$ hyperfine --warmup 5 'sfsu search google' 'scoop search google'
Benchmark 1: sfsu search google
  Time (mean ± σ):      59.9 ms ±   1.6 ms    [User: 1.3 ms, System: 0.9 ms]
  Range (min … max):    57.3 ms …  64.6 ms    36 runs

Benchmark 2: scoop search google
  Time (mean ± σ):     461.1 ms ±   2.8 ms    [User: 90.6 ms, System: 73.4 ms]
  Range (min … max):   455.5 ms … 465.9 ms    10 runs

Summary
  sfsu search google ran
    7.70 ± 0.21 times faster than scoop search google
```

### Listing [^list-version]

```shell
$ hyperfine --warmup 5 'sfsu list' 'hok list' 'scoop list'
Benchmark 1: sfsu list
  Time (mean ± σ):      69.7 ms ±   2.6 ms    [User: 1.5 ms, System: 6.3 ms]
  Range (min … max):    64.0 ms …  75.2 ms    32 runs

Benchmark 2: hok list
  Time (mean ± σ):      69.8 ms ±   1.6 ms    [User: 0.5 ms, System: 13.6 ms]
  Range (min … max):    66.1 ms …  73.1 ms    31 runs

Benchmark 3: scoop list
  Time (mean ± σ):      1.200 s ±  0.029 s    [User: 0.552 s, System: 0.334 s]
  Range (min … max):    1.167 s …  1.265 s    10 runs

Summary
  sfsu list ran
    1.00 ± 0.04 times faster than hok list
   17.21 ± 0.77 times faster than scoop list
```

### Info [^info-version]

Hok does not have the `Updated at` and `Updated by` fields.
As such, for the sake of fairness, I have split the benchmark in two.

The first benchmark compares `sfsu` without these fields to Hok,
and the second benchmark compares `sfsu` with these fields to Scoop.

```shell
$ hyperfine --warmup 5 'sfsu info sfsu' 'scoop info sfsu'
Benchmark 1: sfsu info sfsu
  Time (mean ± σ):     167.8 ms ±   2.3 ms    [User: 0.0 ms, System: 1.0 ms]
  Range (min … max):   163.8 ms … 171.7 ms    15 runs

Benchmark 2: scoop info sfsu
  Time (mean ± σ):     595.1 ms ±   5.3 ms    [User: 137.5 ms, System: 60.9 ms]
  Range (min … max):   587.6 ms … 603.8 ms    10 runs

Summary
  sfsu info sfsu ran
    3.55 ± 0.06 times faster than scoop info sfsu
```

```shell
$ hyperfine --warmup 5 'sfsu info sfsu --disable-updated' 'hok info sfsu'
Benchmark 1: sfsu info sfsu --disable-updated
  Time (mean ± σ):      56.6 ms ±   1.3 ms    [User: 0.4 ms, System: 0.0 ms]
  Range (min … max):    54.7 ms …  61.3 ms    37 runs

Benchmark 2: hok info sfsu
  Time (mean ± σ):      68.2 ms ±   1.5 ms    [User: 5.9 ms, System: 2.4 ms]
  Range (min … max):    65.6 ms …  70.8 ms    32 runs

Summary
  sfsu info sfsu --disable-updated ran
    1.20 ± 0.04 times faster than hok info sfsu
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
[^search-version]: Run on sfsu version [v1.10.3][v1.10.3], Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4], scoop-search version [1.4.1](https://github.com/shilangyu/scoop-search/releases/tag/v1.4.1)
[^info-version]: Run on sfsu version [v1.10.3][v1.10.3] and Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4]
[^list-version]: Run on sfsu version [v1.10.3][v1.10.3] and Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4]

[v1.10.3]: https://github.com/jewlexx/sfsu/releases/tag/v1.10.3
[hokv0.1.0-beta.4]: https://github.com/chawyehsu/hok/releases/tag/v0.1.0-beta.4
