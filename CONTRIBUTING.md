# Contributing to kotonoha-core

Follow [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) for normative behaviour. Update [`docs/spec-traceability.md`](docs/spec-traceability.md) when you add or change public API surface tied to the specification.

## Build

```bash
cargo test
cargo fmt --all --check
```

## Workflow

1. Open an **Issue** for design questions that might affect the public specification (resolve in `kotonoha-spec` first when normative).
2. **Pull requests** should include tests and traceability updates.

## License

[Apache License 2.0](LICENSE).
