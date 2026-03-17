# CLI Commands

Reference for `ogham` CLI commands.

## Compile & Generate

```bash
ogham check                                    # validate schemas — compile without running plugins
ogham check --dir ./myproject                  # validate specific project
ogham generate                                 # run all plugins from ogham.gen.yaml
ogham generate --plugin=proto                  # run single plugin by short name
ogham generate --plugin=ogham-gen-proto        # run single plugin by binary name
ogham generate --skip-breaking                 # skip breaking check even if configured
```

## Debug

```bash
ogham dump                                     # dump compiled IR as JSON to stdout
ogham dump -o ir.json                          # dump IR to file
ogham dump --dir examples/store                # dump specific project
```

## Package Manager

```bash
ogham get github.com/org/database             # add dependency (auto-detects git source for github.com/*)
ogham get github.com/org/database@v2.1.0      # add with specific version/tag
ogham install                                  # resolve + fetch all deps, write ogham.lock.yaml, verify integrity
ogham update                                   # re-fetch latest versions within ranges
ogham vendor                                   # copy deps into vendor/ (without .git)
```

`ogham install` writes `ogham.lock.yaml` for reproducible git dependency resolution. Dependency sources: path (`path:`), git (`git:` + tag/branch/rev), version range (`^1.0.0` — requires proxy or git source). See [package.md](package.md) for full details.

## Breaking Change Detection

```bash
ogham breaking --against git:main                          # compare against git ref
ogham breaking --against git:v1.0.0                        # compare against git tag
ogham breaking --against ./previous-schemas/               # compare against local directory

ogham breaking --against git:main --allow                  # only ERROR blocks, WARNING logged
ogham breaking --against git:main --force                  # nothing blocks, everything logged
```

Breaking checks can also run automatically during `ogham generate` — see [compatibility.md](compatibility.md) for `ogham.mod.yaml` configuration.

## Plugins

Proto export and all code generation is done via plugins. Plugins are standalone binaries that receive IR via stdin and return generated files via stdout.

```bash
# Run proto export plugin:
ogham generate --plugin=proto

# Configure plugins in ogham.gen.yaml:
#   generate:
#     plugins:
#       - name: ogham-gen-proto
#         out: proto/
#       - name: github.com/org/ogham-gen-go
#         out: gen/go/
#       - name: github.com/org/ogham-gen-db
#         grpc: localhost:50051
#         out: gen/db/
```

## Project Scaffolding

Project scaffolding is done via template repositories, not a built-in command. See the oghamlang GitHub org for starter templates.
