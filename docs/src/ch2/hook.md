# Hook

Generate shell hooks

Prints the hook for the supplied shell, allowing one to use `sfsu` instead of `scoop` for its supported commands

## Arguments

- `-D/--disable`

  A comma seperated list of commands to disable.
    Could be, for example, one could supply `search`, and the `search` command would not work with the hooks and would thus just call the regular `scoop search` command

- `-s/--shell`

  The shell the supply hooks for. Defaults to `powershell`.

  Possible values are: `powershell`, `bash`, `zsh`, `nu`
