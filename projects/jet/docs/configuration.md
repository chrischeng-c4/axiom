# Configuration

Jet uses several configuration files. Most projects need only `package.json` — everything else is optional.

## jet.config.yaml

Project-level configuration for the build pipeline, dev server, and task runner.

```yaml
# Dev server settings
dev:
  port: 3000
  proxy:
    /api: http://localhost:8080

# Module aliases (higher priority than tsconfig.json)
alias:
  "@/": "./src/"
  "@components/": "./src/components/"

# Build settings
build:
  outDir: dist

# Task pipeline (for monorepos)
pipeline:
  build:
    dependsOn: ["^build"]
    inputs: ["src/**"]
    outputs: ["dist/**"]
    cache: true
  test:
    dependsOn: ["build"]
  dev:
    persistent: true
    cache: false
```

## package.json

Standard npm `package.json` with Jet extensions:

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "scripts": {
    "dev": "cclab jet dev",
    "build": "cclab jet build"
  },
  "dependencies": {
    "react": "^18.2.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  },
  "overrides": {
    "lodash": "4.17.21"
  },
  "sideEffects": false
}
```

### Jet-specific fields

| Field | Description |
|-------|-------------|
| `overrides` | Force specific dependency versions |
| `sideEffects` | Tree shaking hint (`false` or array of globs) |
| `workspaces` | Workspace package globs |

## .npmrc

Registry and auth configuration. Jet reads `.npmrc` from project, user (`~/`), and global (`/etc/`) locations, with project taking highest priority.

```ini
registry=https://registry.npmjs.org/

# Scoped registry
@myorg:registry=https://npm.myorg.com/

# Auth token
//npm.myorg.com/:_authToken=your-token-here

# Proxy
proxy=http://proxy.example.com:8080
https-proxy=http://proxy.example.com:8080
```

## tsconfig.json

Jet reads `compilerOptions.paths` for module aliases:

```json
{
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

::: tip
If both `jet.config.yaml` `alias` and `tsconfig.json` `paths` define the same alias, `jet.config.yaml` takes priority.
:::

## Priority order

When the same setting is available in multiple places:

1. CLI flags (highest)
2. `jet.config.yaml`
3. `tsconfig.json`
4. `package.json`
5. `.npmrc`
6. Defaults (lowest)
