# Workspaces

Jet supports monorepo workspaces with automatic detection. It works with Nx, pnpm-workspace.yaml, and package.json workspaces.

## Auto-detection

Jet automatically detects your workspace type in this order:

1. **Nx** — `nx.json` exists
2. **pnpm-workspace.yaml** — pnpm workspace config exists
3. **package.json `workspaces`** — workspaces field in root package.json
4. **Single package** — No workspace markers found

## workspace:\* protocol

Reference local packages in your monorepo using the `workspace:` protocol:

```json
{
  "dependencies": {
    "@myorg/ui": "workspace:*",
    "@myorg/utils": "workspace:^1.0.0"
  }
}
```

| Protocol | Resolves to |
|----------|-------------|
| `workspace:*` | Exact local version |
| `workspace:^` | Compatible local version |
| `workspace:~` | Patch-level local version |

Local packages are symlinked into `node_modules/` instead of fetched from the registry.

## pnpm-workspace.yaml

Define which directories contain workspace packages:

```yaml
packages:
  - "packages/*"
  - "apps/*"
  - "libs/*"
```

## Nx integration

When `nx.json` is present, Jet operates in Nx mode:

```bash
# Install all workspace packages
cclab jet install --nx

# Build all projects in topological order
cclab jet build --nx

# Build a single project (with its dependencies)
cclab jet build --project my-app
```

Nx mode ensures:

- Packages are built in dependency order
- Library packages are treated as externals
- Project graph is derived from `project.json` files

## Installing in a workspace

```bash
# Install all packages across the workspace
cclab jet install

# Add a dependency to a specific package
cd packages/my-lib
cclab jet add lodash
```

Jet resolves workspace dependencies first, then falls back to the npm registry for external packages.
