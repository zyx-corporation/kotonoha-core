# kotonoha-core

**Open-source core implementation of the Semantic Lineage System (SLS)** for the Kotonoha ecosystem. This repository hosts source code, developer-oriented documentation, and implementation guidance that align with the public specifications.

**Japanese:** [README_ja.md](README_ja.md)

## Scope

- Core libraries and runtime components for SLS
- Developer documentation, build instructions, and contribution notes (as the codebase grows)
- Implementation notes that belong with the code, distinct from normative public specs

Normative, review-facing specifications live in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). This repository focuses on code and developer-facing artifacts that implement those specifications.

## Related repositories

Public cross-references only.

| Repository | Role |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | Canonical public specifications |
| **kotonoha-core (this repository)** | OSS core implementation of SLS |
| [`kotonoha-cli`](https://github.com/zyx-corporation/kotonoha-cli) | Official `kotonoha` CLI ([definition](https://github.com/zyx-corporation/kotonoha-cli/blob/main/docs/cli-definition.md)) |
| [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs) | Non-specification public docs (manuals, tutorials, guides) |

Whenever possible, changes here should track the specifications in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Resolve open design questions through the public specification before locking behavior in code.

## Language policy

**By default, documents under `kotonoha-core` are written in English.** Provide Japanese alongside the English source when helpful. Keep English primary and use the `*_ja.md` suffix for Japanese files (for example, `README.md` / `README_ja.md`).

## License

Unless otherwise stated in a specific file, repository content is licensed under the [Apache License 2.0](LICENSE).

## Links

- Repository: https://github.com/zyx-corporation/kotonoha-core
