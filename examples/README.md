<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# `noya-cli` examples

End-to-end demos for the `noyafmt` and `noyavalidate` binaries.
Each example is a self-contained shell script — run it directly,
no project setup needed (assumes the binaries are on `$PATH`,
which `cargo install noya-cli` arranges).

| Script | What it shows |
|---|---|
| [`format-precommit.sh`](format-precommit.sh) | Drop-in `git pre-commit` hook gating commits on `noyafmt --check`. |
| [`fix-quoted-numbers.sh`](fix-quoted-numbers.sh) | Walkthrough of the `--fix` autofix flow: quoted scalar → schema-typed integer, with the surrounding comment preserved. |
| [`ci-pipeline.sh`](ci-pipeline.sh) | Combined `noyafmt --check` + `noyavalidate --schema` gate for `.github/workflows/*.yml` (or any other CI runner). |

### Ecosystem-specific schema gates

Each script fetches its respective schema from
[schemastore.org](https://www.schemastore.org/) (cached
locally on first run) and validates every matching file under
the working directory:

| Script | Targets | Schema source |
|---|---|---|
| [`validate-k8s.sh`](validate-k8s.sh) | Kubernetes manifests | per-manifest schema (kustomize / kubeconform style) |
| [`validate-github-actions.sh`](validate-github-actions.sh) | `.github/workflows/*.yml` | schemastore: `github-workflow.json` |
| [`validate-helm.sh`](validate-helm.sh) | Helm charts (`charts/*/values*.yaml`) | per-chart `values.schema.json` |
| [`validate-compose.sh`](validate-compose.sh) | `docker-compose.yml` / `compose.yaml` | upstream `compose-spec.json` |
| [`validate-pyproject.sh`](validate-pyproject.sh) | Python project YAML configs (mkdocs, pre-commit, dependabot, readthedocs, gitlab-ci, circleci) | schemastore — one schema per file pattern |

```bash
# All examples are independent — pick one and run it.
chmod +x crates/noya-cli/examples/*.sh
crates/noya-cli/examples/fix-quoted-numbers.sh
```

## License

Dual-licensed under Apache 2.0 or MIT, at your option.
