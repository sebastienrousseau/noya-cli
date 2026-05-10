# CLI flag reference

The full surface for both binaries — `noyafmt` and `noyavalidate`
— with worked examples for every flag. The same content is
available via `--help` and the man pages installed under
`/usr/share/man/man1/` (or `complete/{noyafmt,noyavalidate}.1`
when installed from the tarball).

The clap command tree is regenerated at build time via the same
`noya_cli::{noyafmt_command, noyavalidate_command}` builders that
drive the runtime parser, so this file, the manpages, and the
shell completions cannot drift from each other.

## `noyafmt`

```
noyafmt [OPTIONS] [FILE]...
```

Auto-format YAML via the noyalib CST. Reads YAML from FILE
arguments (or stdin via `--stdin`) and rewrites them through
noyalib's lossless CST formatter. Comments, anchor positions, and
document structure are preserved byte-for-byte; only whitespace
and quoting are normalised.

### Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--check` | bool | off | Verify each FILE is formatted; print the list of files that need formatting and exit 1 if any do. Non-destructive. Suitable as a pre-commit / CI gate. Conflicts with `--write`. |
| `--write` | bool | off | Rewrite each FILE in place. Default is to print the formatted source to stdout. Conflicts with `--check`. |
| `--stdin` | bool | off | Read from stdin, write to stdout. Mutually exclusive with FILE arguments. |
| `--indent N` | unsigned int | `2` | Indentation width in spaces. |
| `-h`, `--help` | — | — | Print help (long form on `--help`). |
| `-V`, `--version` | — | — | Print the version. |

### Positional arguments

| Argument | Repeatable | Description |
|---|---|---|
| `FILE` | yes | YAML files to format. Pass `--stdin` to read from stdin instead. |

### Examples

```sh
# Default: read FILE, print formatted YAML to stdout.
noyafmt config.yaml

# Rewrite in place.
noyafmt --write config.yaml

# CI gate — exit 1 if any file needs formatting.
noyafmt --check ci/*.yaml

# Editor integration: pipe through stdin.
cat config.yaml | noyafmt --stdin

# Custom indent width.
noyafmt --write --indent 4 config.yaml

# Multiple files at once.
noyafmt --write services/*.yaml deployments/*.yaml
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | All files are formatted (or were rewritten when `--write`) |
| 1 | At least one file is not formatted (`--check`) or a parse error occurred |
| 2 | Usage error (conflicting flags, missing argument) |
| 3 | I/O error (file not found, permission denied) |

## `noyavalidate`

```
noyavalidate [OPTIONS] [FILE]
```

Check YAML syntax (and optional JSON Schema). Reads one or more
YAML documents from FILE (or stdin), reports syntax errors via
the miette fancy renderer, and — when `--schema PATH` is given —
validates each parsed document against a JSON Schema 2020-12
contract (the schema may itself be written in YAML or JSON).

`--fix` rewrites the input in-place through the lossless CST
formatter, normalising whitespace and quoting without changing
semantics. When the input is stdin, the formatted output is
written to stdout instead.

### Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `-s PATH`, `--schema PATH` | path | — | Validate each document against the JSON Schema 2020-12 at PATH (the schema may itself be YAML or JSON). |
| `--fix` | bool | off | Rewrite FILE in place via the CST formatter (lossless: byte-faithful for everything except normalised whitespace and line endings). With stdin input, the formatted bytes go to stdout. |
| `-q`, `--quiet` | bool | off | Suppress success output. |
| `-h`, `--help` | — | — | Print help (long form on `--help`). |
| `-V`, `--version` | — | — | Print the version. |

### Positional arguments

| Argument | Required | Description |
|---|---|---|
| `FILE` | optional | YAML file to validate. Use `-` or omit for stdin. |

### Examples

```sh
# Syntax-only check, errors via miette fancy renderer.
noyavalidate config.yaml

# Schema validation against a YAML-flavoured JSON Schema.
noyavalidate --schema schemas/config.schema.yaml config.yaml

# Schema validation against a JSON-flavoured schema.
noyavalidate --schema schemas/config.schema.json config.yaml

# Read from stdin.
cat config.yaml | noyavalidate

# Format-fix in place.
noyavalidate --fix config.yaml

# Format-fix from stdin to stdout.
cat config.yaml | noyavalidate --fix > clean.yaml

# Combined: schema-validate then format-fix.
noyavalidate --schema schema.yaml --fix config.yaml

# Quiet mode for CI: only errors are printed.
noyavalidate --quiet --schema schema.yaml ci/*.yaml
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | All documents valid (and fixed if `--fix`) |
| 1 | Parse error or schema violation |
| 2 | Usage error |
| 3 | I/O error |

## Shell completions

Pre-generated completion scripts ship in the release tarball under
`complete/`:

| Shell | File |
|---|---|
| bash | `complete/noyafmt.bash`, `complete/noyavalidate.bash` |
| fish | `complete/noyafmt.fish`, `complete/noyavalidate.fish` |
| zsh | `complete/_noyafmt`, `complete/_noyavalidate` |
| PowerShell | `complete/_noyafmt.ps1`, `complete/_noyavalidate.ps1` |

System packages install them to the right location automatically.
For local install (`cargo install noya-cli`), regenerate via
`cargo xtask completions`.

## Man pages

Roff-format man pages ship as `doc/noyafmt.1` and
`doc/noyavalidate.1`. System packages install to
`/usr/share/man/man1/`; for local install, regenerate via
`cargo xtask manpages`.

## Related

- [Recipes](./recipes.md) — common workflows (CI gates, pre-commit
  hooks, schema validation patterns)
- [Crate README](../README.md) — installation, package channels,
  build-from-source
