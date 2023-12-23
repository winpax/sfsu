# Describe

Provides information about a particular `scoop` package, installed or not

Acts pretty similar to `scoop info`, but lists every matching package in every bucket, in a more compact, non-powershell format, and slightly less verbose

## Arguments

- `<PACKAGE>`

The package to describe

Note that unlike `sfsu search`, this does not take a regex, and rather matches the exact package name

- `-b/--bucket <NAME>`

  The name of the bucket to exclusively search in
