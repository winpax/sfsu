# Search

`sfsu search`

Search for a package

Takes one argument, which is a regular expression to search for.

This can be just a package name, or a more complex expression

For example, the query `google` will return `googlechrome`,
but the query `g*gle` will return `googlechrome` and `megaglest`

You may have noticed that the queries are not exclusive, and the search `googlechrome` may return `fakegooglechrome` as well.
This can be fixed by putting a `^` at the start of the search, and a `$` at the end.

## Arguments

- `<QUERY>`

  The query to match. Currently takes a regular expression

- `-C/--case-sensitive`

  Whether or not the search should be case-sensitive

- `-b/--bucket <NAME>`

  The name of the bucket to exclusively search in

- `-I/--installed`

  Search only installed packages. Works similarly to running `sfsu list | grep <NAME>`

- `-m/--mode <MODE>`

  Determines what to match, whether that be the package name, its binaries, or both. Defaults to `name`

  Possible values: `name`, `binary`, `both`
