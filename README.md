# *S*tupid *F*ast *S*coop *U*tils

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jewlexx/sfsu/build.yml)
[![dependency status](https://deps.rs/repo/github/jewlexx/sfsu/status.svg)](https://deps.rs/repo/github/jewlexx/sfsu)
![GitHub all releases](https://img.shields.io/github/downloads/jewlexx/sfsu/total)
![GitHub](https://img.shields.io/github/license/jewlexx/sfsu)
![Scoop Version (extras bucket)](https://img.shields.io/scoop/v/sfsu?bucket=extras)

> [!NOTE]
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

> [!NOTE]
> These benchmarks may not accurately represent the speeds on your system.
> While they do provide a relative measurement, your environment and results **will** be different.

On average, `sfsu search` is **~400** times faster than regular `scoop search` and **~5** times faster than [scoop-search](https://github.com/shilangyu/scoop-search).

`sfsu list` is **~35** times faster than `scoop list`

<!--TODO: A more detailed comparison of sfsu and hok-->

`sfsu` and `hok` are pretty comperable in all benchmarks. `sfsu` wins some benchmarks and `hok` wins others.

Done on a _AMD Ryzen 9 7900X @ 5.5GHz_ with _32GB_ of RAM at 4800MHz and 11 scoop buckets listed below

### Searching [^search-version]

```shell
$ hyperfine --warmup 5 'sfsu search google' 'hok search google' 'scoop-search google' 'scoop search google'

Benchmark 1: sfsu search google
  Time (mean ± σ):      32.0 ms ±   2.1 ms    [User: 0.4 ms, System: 2.4 ms]
  Range (min … max):    29.5 ms …  44.5 ms    78 runs

Benchmark 2: hok search google
  Time (mean ± σ):      37.8 ms ±   1.6 ms    [User: 2.2 ms, System: 2.2 ms]
  Range (min … max):    34.5 ms …  44.2 ms    65 runs

Benchmark 3: scoop-search google
  Time (mean ± σ):     352.6 ms ±  29.9 ms    [User: 14.1 ms, System: 78.1 ms]
  Range (min … max):   325.2 ms … 415.2 ms    10 runs

Benchmark 4: scoop search google
  Time (mean ± σ):     12.031 s ±  0.192 s    [User: 3.400 s, System: 6.475 s]
  Range (min … max):   11.756 s … 12.464 s    10 runs

Summary
  sfsu search google ran
    1.18 ± 0.09 times faster than hok search google
   11.03 ± 1.17 times faster than scoop-search google
  376.34 ± 24.89 times faster than scoop search google
```

### Listing [^list-version]

```shell
$ hyperfine --warmup 5 'sfsu list' 'hok list' 'scoop list'

Benchmark 1: sfsu list
  Time (mean ± σ):      41.3 ms ±   1.6 ms    [User: 3.1 ms, System: 11.2 ms]
  Range (min … max):    39.2 ms …  45.1 ms    63 runs

Benchmark 2: hok list
  Time (mean ± σ):      32.4 ms ±   0.9 ms    [User: 3.9 ms, System: 13.7 ms]
  Range (min … max):    30.9 ms …  35.6 ms    84 runs

Benchmark 3: scoop list
  Time (mean ± σ):      1.147 s ±  0.019 s    [User: 0.531 s, System: 0.251 s]
  Range (min … max):    1.121 s …  1.183 s    10 runs

Summary
  hok list ran
    1.27 ± 0.06 times faster than sfsu list
   35.34 ± 1.14 times faster than scoop list
```

### Info [^info-version]

```shell
$ hyperfine --warmup 5 'sfsu describe sfsu' 'hok info sfsu' 'scoop info sfsu'

Benchmark 1: sfsu describe sfsu
  Time (mean ± σ):      20.3 ms ±   1.9 ms    [User: 1.4 ms, System: 5.2 ms]
  Range (min … max):    18.0 ms …  30.5 ms    134 runs

Benchmark 2: hok info sfsu
  Time (mean ± σ):      31.6 ms ±   1.2 ms    [User: 4.1 ms, System: 6.4 ms]
  Range (min … max):    29.4 ms …  35.2 ms    84 runs

Benchmark 3: scoop info sfsu
  Time (mean ± σ):     475.1 ms ±   3.9 ms    [User: 146.9 ms, System: 59.7 ms]
  Range (min … max):   469.9 ms … 481.2 ms    10 runs

Summary
  sfsu describe sfsu ran
    1.55 ± 0.16 times faster than hok info sfsu
   23.35 ± 2.22 times faster than scoop info sfsu
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
[^search-version]: Run on sfsu version [v1.8.3][v1.8.3], Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4], scoop-search version [1.4.0](https://github.com/shilangyu/scoop-search/releases/tag/v1.4.0)
[^info-version]: Run on sfsu version [v1.6.3][v1.6.3] and Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4]
[^list-version]: Run on sfsu version [v1.6.3][v1.6.3] and Hok version [v0.1.0-beta.4][hokv0.1.0-beta.4]

[v1.8.3]: https://github.com/jewlexx/sfsu/releases/tag/v1.8.3
[v1.6.3]: https://github.com/jewlexx/sfsu/releases/tag/v1.6.3
[hokv0.1.0-beta.4]: https://github.com/chawyehsu/hok/releases/tag/v0.1.0-beta.4
