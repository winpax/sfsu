# *S*tupid *F*ast *S*coop *U*tils

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jewlexx/sfsu/build.yml)
[![dependency status](https://deps.rs/repo/github/jewlexx/sfsu/status.svg)](https://deps.rs/repo/github/jewlexx/sfsu)
![GitHub all releases](https://img.shields.io/github/downloads/jewlexx/sfsu/total)
![GitHub](https://img.shields.io/github/license/jewlexx/sfsu)
![Scoop Version (extras bucket)](https://img.shields.io/scoop/v/sfsu?bucket=extras)
[![wakatime](https://wakatime.com/badge/user/69c39493-dba9-4b9d-8ae6-1a6a17e60cb4/project/ba7eaa48-0f34-4b20-95e5-4ba2e6184d39.svg)](https://wakatime.com/badge/user/69c39493-dba9-4b9d-8ae6-1a6a17e60cb4/project/ba7eaa48-0f34-4b20-95e5-4ba2e6184d39)

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

```sh
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

## Benchmarks

Benchmarks have been moved to [the wiki](https://github.com/jewlexx/sfsu/wiki/Benchmarks)

## Building yourself

### Initial setup

- Read the [Contributing Guide](CONTRIBUTING.md)
- Read the [Code of Conduct](CODE_OF_CONDUCT.md)
- Install [just](https://github.com/casey/just) to run the build scripts
- Install [pre-commit](https://pre-commit.com/) to run the pre-commit hooks
  - After installing the tool, run `just pre-commit` to run the hooks automatically, or run `pre-commit install` and `pre-commit install --hook-type commit-msg` to install the relevant hooks to install them manually

The build instructions can be found [in the wiki](https://github.com/jewlexx/sfsu/wiki/Building)

## Long Term Goals

I have a couple of long term goals.

Firstly, I want to create a Rust library to help interacting with [Scoop](https://scoop.sh) from code. This library would allow for things like installing packages, running updates, etc.
It will likely start by providing a function to get the Scoop install path, but hopefully over time it will grow into a fully fledged library, which is used internally by sfsu to interact with Scoop.

My other long term goal is to create a Scoop replacement for those who want it, in a similar vein as [Shovel](https://github.com/Ash258/Scoop-Core). This is a fairly large undertaking and will definitely take me a lot of time, so this is a very long term goal, and may never happen. Despite this I never really plan to replace Scoop. It is a great package manager and if anything `sfsu` would just be a command you can run instead of Scoop, but would run on Scoop installations.

In the meantime I will continue working on this independently of Scoop as a collection of seperate tools that work in conjunction with Scoop.

**Made with 💗 by Juliette Cordor**
