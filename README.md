# Ogham

Ogham is a modern schema language for building data-oriented applications.

The repository is currently in active development mode, so breaking changes are expected.

## Quickstart

```bash
make help
make proto
make check
make test
make clippy
make fmt
```

## Repository Structure

- `crates/` - Rust workspace crates (`ogham-core`, `ogham-compiler`, `ogham-cli`, `ogham-lsp`, `ogham-proto`)
- `proto/` - protobuf sources and generation config
- `docs/` - architecture decisions and syntax references
- `editors/` - editor integrations

## References

- `docs/adr/syntax/` - syntax contracts and grammar
- `docs/adr/language.md` - language ADR
- `docs/adr/package.md` - package ADR
- `docs/adr/cmd.md` - CLI ADR
