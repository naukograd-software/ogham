# CLI Commands

Reference for `ogham` CLI commands.

## Package Manager

```bash
ogham get github.com/org/database             # add dependency to ogham.toml, fetch and build (if plugin)
ogham get github.com/org/database@2.1.0       # add a specific version
ogham install                                  # install/build all dependencies for the current project
ogham install github.com/org/tool@latest      # fetch and build a binary into OGHAM_BIN (global)
ogham update                                   # update versions in ogham.lock
ogham vendor                                   # copy dependencies into vendor/
ogham verify                                   # verify checksums
```

## Compile

```bash
ogham compile                         # parse schemas/, generate .proto, run all plugins
ogham compile --plugin=database       # run only one plugin
ogham compile --target=go             # run only for a specific target
ogham compile --mode=native           # override: use native mode instead of proto
```

## Plugin Development

```bash
ogham init --plugin <name>            # create a plugin scaffold in the current directory
ogham serve --plugin <name> --address :50051
```
