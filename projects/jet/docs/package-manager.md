# Package Manager

Jet includes a full-featured package manager compatible with the npm registry. Packages are stored in a global content-addressable store and hardlinked into your project.

## Install dependencies

```bash
cclab jet install
```

Reads `package.json` and installs all dependencies. Creates `jet-lock.yaml` if it doesn't exist.

| Flag | Description |
|------|-------------|
| `--frozen-lockfile` | Fail if lockfile is out of sync (auto-enabled in CI) |
| `--no-cache` | Skip metadata cache, always fetch from registry |
| `--no-install` | Resolve and update lockfile without downloading |
| `--nx` | Install all workspace packages in Nx mode |

## Add a dependency

```bash
# Production dependency
cclab jet add react

# Dev dependency
cclab jet add -D typescript

# Specific version
cclab jet add react@18.2.0
```

## Remove a dependency

```bash
cclab jet remove lodash
```

## Update dependencies

```bash
# Update within semver range
cclab jet update

# Update a specific package
cclab jet update react

# Ignore semver, update to latest
cclab jet update --latest
```

## Security audit

```bash
cclab jet audit
```

Checks installed packages against known vulnerabilities.

## Patching packages

Edit a dependency directly and generate a patch file:

```bash
# Create an editable copy
cclab jet patch react

# Make your changes in node_modules/react/...

# Generate a .patch file
cclab jet patch-commit react
```

Patches are saved in the project and applied automatically on `install`.

## Publishing

```bash
# Publish to npm registry
cclab jet publish

# Publish with a tag
cclab jet publish --tag beta

# Set access level
cclab jet publish --access public

# Create tarball without publishing
cclab jet pack
```

## Store management

Jet stores all downloaded packages in `~/.jet-store/`. To remove unreferenced packages:

```bash
cclab jet store prune
```

## Lockfile

Jet uses `jet-lock.yaml` as its lockfile. It contains the fully resolved dependency graph with exact versions, integrity hashes, and dependency relationships.

The lockfile is automatically created and updated. Commit it to version control for reproducible installs.
