<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# CLI ecosystem comparison

This comparison describes delivery and workflow characteristics. It is not a
claim that every tool has the same scope.

| Project | Comment-preserving format | JSON Schema | Native packages | Generated manpages |
| :--- | :---: | :---: | :---: | :---: |
| `noya-cli` | Yes | 2020-12 | deb and rpm | Yes |
| `yq` | Operation-dependent | No | Project-dependent | Project-dependent |
| `yamllint` | Lint only | No | Project-dependent | Project-dependent |
| Prettier YAML | Reprints documents | No | npm | No |

The meaningful distinction is the editing model. `noyafmt` and
`noyavalidate --fix` use noyalib's concrete syntax tree, so an edit need not
reprint unrelated comments or whitespace. Use each upstream project's current
documentation before making a tool-selection decision.

For the complete command surface, see [CLI reference](cli-reference.md).
