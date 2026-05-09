# kotonoha-core

**Open-source core implementation of the Semantic Lineage System (SLS)** for the Kotonoha ecosystem. This repository hosts source code, developer-oriented documentation, and implementation guidance that align with the public specifications.

**Japanese:** [README_ja.md](README_ja.md)

## Scope

- Core libraries and runtime components for SLS
- Developer documentation, build instructions, and contribution notes (as the codebase grows)
- Implementation notes that belong with the code, distinct from normative public specs

Normative, review-facing specifications live in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Non-public design exploration and internal project material remain in [`kotonoha-project`](https://github.com/zyx-corporation/kotonoha-project).

## Related repositories

| Repository | Role |
| --- | --- |
| [`kotonoha-project`](https://github.com/zyx-corporation/kotonoha-project) | Non-public project documents and design exploration |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | Canonical public specifications |
| **kotonoha-core (this repository)** | OSS core implementation of SLS |

Whenever possible, changes here should track the specifications in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). If a design point is still unresolved, work it through `kotonoha-project` and the spec before locking behavior in code.

## Language policy

**By default, documents under `kotonoha-core` are written in English.** Provide Japanese alongside the English source when helpful. Keep English primary and use the `*_ja.md` suffix for Japanese files (for example, `README.md` / `README_ja.md`).

## License

Unless otherwise stated in a specific file, repository content is licensed under the [Apache License 2.0](LICENSE).

## Links

- Repository: https://github.com/zyx-corporation/kotonoha-core
