# CLI Commands

Reference for `ogham` CLI commands.

## Package Manager

```bash
ogham get github.com/org/database             # add dependency to ogham.mod.yaml, fetch and build (if plugin)
ogham get github.com/org/database@2.1.0       # add a specific version
ogham install                                  # install/build all dependencies for the current project
ogham update                                   # update dependency versions
ogham vendor                                   # copy dependencies into vendor/
```

## Generate

```bash
ogham generate                                 # run all plugins from ogham.gen.yaml
ogham generate --plugin=database               # run single plugin
```

## Proto Export

```bash
ogham proto export ./proto/                    # export .proto files for external toolchains (protoc, buf, etc.)
```

## Breaking Change Detection

```bash
ogham check breaking --against git:main                          # compare against git ref
ogham check breaking --against git:v1.0.0                        # compare against git tag
ogham check breaking --against ./previous-schemas/               # compare against local directory
ogham check breaking --against github.com/org/schemas@v1.0.0    # compare against published version

ogham check breaking --against git:main --allow                  # only ERROR blocks, WARNING logged
ogham check breaking --against git:main --force                  # nothing blocks, everything logged
```

## Plugin Development

```bash
ogham init --plugin <name>                     # create a plugin scaffold in the current directory
ogham serve --plugin <name> --address :50051   # serve plugin as gRPC
```
