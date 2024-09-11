# *S*tupid *F*ast *S*coop *U*tils

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/winpax/sfsu/build.yml)](https://github.com/winpax/sfsu/actions)
[![dependency status](https://deps.rs/repo/github/winpax/sfsu/status.svg)](https://deps.rs/repo/github/winpax/sfsu)
[![GitHub all releases](https://img.shields.io/github/downloads/winpax/sfsu/total)](https://github.com/winpax/sfsu/releases)
[![GitHub](https://img.shields.io/github/license/winpax/sfsu)](LICENSE-APACHE)
[![Scoop Version (winpax bucket)](https://img.shields.io/scoop/v/sfsu?bucket=https%3A%2F%2Fgithub.com%2Fwinpax%2Fbucket)](https://github.com/winpax/bucket)
![wakatime](https://wakatime.com/badge/user/69c39493-dba9-4b9d-8ae6-1a6a17e60cb4/project/ba7eaa48-0f34-4b20-95e5-4ba2e6184d39.svg)

> [!NOTE]
> This is still under development. It currently provides faster alternatives to most, but not all, Scoop commands.
> All breaking changes will only occur in v2.0 and with proper deprecation warnings.

Super fast replacements and additions to scoop commands written in Rust

Are looking for our underlying library that makes all of this possible (or do you want to make your own implementation of Scoop?), check out [sprinkles](https://github.com/winpax/sprinkles)

## Reporting Issues

Please, please, please check the [FAQs](https://github.com/winpax/FAQs), before you report an issue.

If you have a question, please ask it on [the discussions page](https://github.com/winpax/sfsu/discussions).

If you have a bug report, feature request, or other issue, then [open an issue](https://github.com/winpax/sfsu/issues/new/choose).

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

Benchmarks have been moved to [the wiki](https://github.com/winpax/sfsu/wiki/Benchmarks)

## Building yourself

### Initial setup

Before you get started make sure you

- Read the [Contributing Guide](CONTRIBUTING.md)
- Read the [Code of Conduct](CODE_OF_CONDUCT.md)

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) as per [rust-toolchain.toml](rust-toolchain.toml)
  - Prerequisites for the msvc toolchain are also required. This generally means you will need to install Visual Studio.
- [Just](https://github.com/casey/just)
- [Powershell](https://github.com/PowerShell/PowerShell)
- Install [pre-commit](https://pre-commit.com/) to run the pre-commit hooks

#### Build instructions

- Run `just setup`
- Run `cargo build` to build the project

## Long Term Goals

I have a couple of long term goals.

Firstly, I want to create a Rust library to help interacting with [Scoop](https://scoop.sh) from code. This library would allow for things like installing packages, running updates, etc.
It will likely start by providing a function to get the Scoop install path, but hopefully over time it will grow into a fully fledged library, which is used internally by sfsu to interact with Scoop.

My other long term goal is to create a Scoop replacement for those who want it, in a similar vein as [Shovel](https://github.com/Ash258/Scoop-Core). This is a fairly large undertaking and will definitely take me a lot of time, so this is a very long term goal, and may never happen. Despite this I never really plan to replace Scoop. It is a great package manager and if anything `sfsu` would just be a command you can run instead of Scoop, but would run on Scoop installations.

In the meantime I will continue working on this independently of Scoop as a collection of seperate tools that work in conjunction with Scoop.

**Made with 💗 by Juliette Cordor**
