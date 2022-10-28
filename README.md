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

**Made with 💗 by Juliette Cordor**
