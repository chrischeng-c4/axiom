# Task Runner

Jet can run scripts, execute binaries, and orchestrate task pipelines.

## Run a script

```bash
# Run a package.json script
cclab jet run build
cclab jet run test

# List available scripts
cclab jet run
```

Jet runs scripts defined in `package.json` with `node_modules/.bin` on the PATH. Pre/post hooks (`prebuild`, `postbuild`) are executed automatically.

## Execute a command

Run any command with `node_modules/.bin` on the PATH:

```bash
cclab jet exec eslint src/
cclab jet exec tsc --noEmit
```

## Download and execute

Like `npx` — download a package and run it:

```bash
cclab jet jtx create-react-app my-app
cclab jet jtx @biomejs/biome check .
```

## Watch mode

Re-run a script when files change:

```bash
cclab jet run test --watch
```

## Pipeline tasks

Define task pipelines in `jet.config.yaml` for monorepo orchestration:

```yaml
pipeline:
  build:
    dependsOn: ["^build"]
    inputs: ["src/**"]
    outputs: ["dist/**"]
    cache: true

  test:
    dependsOn: ["build"]
    cache: true

  lint:
    cache: true

  dev:
    persistent: true
    cache: false
```

### Task options

| Field | Description |
|-------|-------------|
| `dependsOn` | Tasks that must run first. `^build` means the `build` task in dependencies. |
| `inputs` | Glob patterns for input files (affects cache key) |
| `outputs` | Glob patterns for output files (cached) |
| `cache` | Enable caching (default: `true`) |
| `persistent` | Long-running task, never cached (e.g., dev server) |
| `env` | Environment variables that affect the cache key |
| `command` | Override the command to run |

### Run with filters

```bash
# Run in specific packages
cclab jet run build --filter my-package

# Dry run — show what would execute
cclab jet run build --dry
```

## Caching

Jet caches task outputs based on:

- Input file content hashes
- Dependent task outputs
- Specified environment variables

If inputs haven't changed, the cached output is restored instantly.
