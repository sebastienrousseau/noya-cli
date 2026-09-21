<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# Repository standard compliance grade

Assessment date: 2026-09-20. Source rubric:
`/Users/seb/Code/REPO-STANDARD.md`. The workspace auditor evaluates checked-in
signals conservatively and does not treat a workflow file as proof that a run
passed.

## Result

**17/24 signals (71%). README template: pass. Strict candidate tier: L1.**

| Category | Signals | Summary |
| :--- | :---: | :--- |
| Identity and README | 3/3 | Canonical structure, policy, badges, and toolchain evidence |
| Documentation | 2/3 | Manual and architecture present; complete L3 evidence remains open |
| Build and install UX | 2/3 | Native build, generated manpages and completions; static L2 detector does not recognize the GNUmakefile shape |
| Releases and binaries | 3/3 | Automated multi-platform release and provenance signals |
| Packaging | 1/3 | Packaging exists; Repology and reproducibility evidence remain incomplete |
| CI quality gates | 2/3 | Matrix and coverage signals present; full L3 powerset evidence remains open |
| Supply chain | 3/3 | Audit, signing, provenance, and policy signals present |
| Community | 1/3 | Foundation files present; template and docs-lint detection remain gaps |

The cumulative tier remains L1 because every L2 category must pass. Priority
work is explicit Repology tracking, reproducible-build verification, and making
the existing GNUmakefile, issue/PR templates, and markdown lint gates legible to
the portfolio audit.
