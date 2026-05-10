# noyafmt + noyavalidate recipes

Common workflows. Each recipe is a self-contained snippet you can
drop into your project or CI.

## CI: format gate

Block PRs that introduce unformatted YAML.

### GitHub Actions

```yaml
# .github/workflows/yaml-fmt.yml
name: YAML format
on: [pull_request]
jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install noya-cli --locked
      - run: noyafmt --check $(git ls-files '*.yaml' '*.yml')
```

### GitLab CI

```yaml
yaml-fmt:
  image: rust:1.85-bookworm
  before_script:
    - cargo install noya-cli --locked
  script:
    - noyafmt --check $(git ls-files '*.yaml' '*.yml')
```

### Pre-commit hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: noyafmt
        name: noyafmt --check
        entry: noyafmt --check
        language: system
        files: \.(yaml|yml)$
```

## CI: schema gate

Block PRs that violate a JSON Schema contract.

### GitHub Actions

```yaml
name: YAML schema
on: [pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install noya-cli --locked
      - name: Validate Kubernetes manifests
        run: |
          for f in k8s/**/*.yaml; do
            noyavalidate --schema schemas/k8s.schema.yaml "$f" || exit 1
          done
```

## Editor integration

### VS Code on save

Pair noyafmt with VS Code's `editor.formatOnSave`. The recommended
path is the dedicated [noyalib-lsp](../../noyalib-lsp/README.md)
extension, which provides format-on-save, diagnostics, and hover.
For lighter-weight integration:

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [{
    "label": "noyafmt: write",
    "type": "shell",
    "command": "noyafmt",
    "args": ["--write", "${file}"],
    "problemMatcher": []
  }]
}
```

### Neovim

```lua
-- ~/.config/nvim/lua/yaml-fmt.lua
vim.api.nvim_create_autocmd("BufWritePost", {
  pattern = { "*.yaml", "*.yml" },
  callback = function()
    vim.fn.system("noyafmt --write " .. vim.fn.expand("%"))
    vim.cmd("e!") -- reload buffer
  end,
})
```

For the richer experience (diagnostics, hover, code actions) use
[noyalib-lsp](../../noyalib-lsp/README.md) via `nvim-lspconfig`.

## Bulk operations

### Format every YAML in a tree

```sh
find . -name '*.yaml' -o -name '*.yml' | xargs noyafmt --write
```

### Validate every Kubernetes manifest

```sh
find k8s -name '*.yaml' -exec \
  noyavalidate --schema schemas/k8s.schema.yaml --quiet {} +
```

### Generate canonical diff after a refactor

When mass-renaming keys across many YAML files, format both before
and after to reduce the diff to the actual semantic change:

```sh
git stash                                            # save WIP
noyafmt --write **/*.yaml && git commit -m 'fmt'     # format baseline
git stash pop                                        # apply WIP
noyafmt --write **/*.yaml                            # format after
git diff                                             # only semantic changes
```

## Pipeline integration

### `kubectl apply` only when format-clean

```sh
noyafmt --check k8s.yaml && kubectl apply -f k8s.yaml
```

### Helm chart linting

```sh
helm template ./chart \
  | noyavalidate --schema schemas/k8s.schema.yaml -
```

### Render-then-validate Argo CD application

```sh
argocd app manifests my-app \
  | noyavalidate --schema schemas/argo-app.schema.yaml -
```

## Troubleshooting

### "permission denied" rewriting a file

`--write` opens FILE for writing. If the file is read-only or
owned by another user, the write fails with exit 3 (I/O error).
Use `chmod +w FILE` or `sudo`.

### Schema validation passes locally, fails in CI

The schema file path is resolved relative to the working
directory of the `noyavalidate` invocation, not relative to the
input file. In GitHub Actions, `${{ github.workspace }}` is the
repo root; ensure your `--schema` path is relative to that.

### `--fix` changed more than I expected

The CST formatter normalises whitespace, quoting, and indentation
to a canonical form. It does *not* change document semantics:
keys, values, comments, anchors, and tags survive byte-for-byte.
If a `--fix` produced unexpected output, file an issue with the
input — the formatter is documented as preserving everything that
isn't trivia.

### Stdin closes the process before output

When piping `--stdin` you may need to terminate the input with
Ctrl-D (`^D`) on a new line:

```sh
noyafmt --stdin
key: value
^D
```

For programmatic use (CI scripts), redirect a file or use `<<<`
heredoc syntax.
