<!-- HANDWRITE-BEGIN gap="missing-generator:doc:b7820e52" tracker="pending-tracker" reason="Document the library publish + private-registry workflow: `.npmrc` scoped registry + auth token (GitLab/Verdaccio/Nexus), `jet build --lib`, `jet publish --build --access --tag`, and the metadata fields jet validates." -->
# Publishing a library with jet

This guide covers publishing a JavaScript/TypeScript **library** with jet:
building a publishable artifact, validating that the package metadata points
at real files, and shipping it to a public or private npm registry.

> Scope: this is the *library publish* path (`jet build --lib` →
> `jet publish`). For app bundling see the build docs; this page is about
> producing a `dist/` an external consumer can `import`/`require`.

## TL;DR

```bash
# Build the library, then publish it (build + validate + pack + upload).
jet publish --build --access restricted --tag latest
```

`--build` runs `jet build --lib` first, auto-fills any missing
`main`/`module`/`types`, validates every metadata target exists, and only then
packs and uploads. Without `--build`, jet publishes the `dist/` you already
have on disk (and still validates the metadata — a dangling `main` is a hard
error, not a silent ship).

## 1. Build the library

A library build keeps `dependencies` and `peerDependencies` external (real
`import ... from "pkg"` / `require("pkg")` statements) and inlines your
internal relative modules. It emits ESM (`*.js`), optional CJS (`*.cjs`), and
`*.d.ts` declarations under `dist/`.

```bash
jet build --lib
```

Output for a single `.` entry:

```
dist/
  index.js      # ESM     → package.json "module"
  index.cjs     # CJS     → package.json "main"
  index.d.ts    # types   → package.json "types"
```

Multiple `exports` subpaths produce one output file per (entry × format),
e.g. `dist/client.js`, `dist/client.cjs`, `dist/client.d.ts`.

## 2. Metadata fields jet validates

Before packing/uploading, jet asserts that **every** declared metadata target
resolves to a file that exists under the project root (i.e. will be inside the
tarball). A missing target is a hard error naming the field, the declared
value, and the absolute path it checked — so a dangling entry never ships and
breaks every consumer with `MODULE_NOT_FOUND`.

| Field      | Meaning                          | Auto-filled by `--build` from |
|------------|----------------------------------|-------------------------------|
| `main`     | CommonJS entry (`require`)       | the `.` CJS output (`index.cjs`) |
| `module`   | ESM entry (`import`)             | the `.` ESM output (`index.js`)  |
| `types`    | TypeScript declarations          | the `.` `.d.ts` output (`index.d.ts`) |
| `exports`  | conditional/subpath export map   | not auto-filled — validated only |

Validation rules:

- `main`, `module`, `types`, and every concrete string leaf reachable from
  `exports` (conditional maps and subpath maps are walked recursively) must
  point at an existing file.
- `*` wildcard targets (`"./features/*": "./dist/features/*.js"`) are skipped —
  they need a filesystem glob and are out of scope.
- Absent fields are *not* required. With `--build`, jet auto-fills absent
  `main`/`module`/`types` from the build output before validating; an
  already-declared field is left untouched (your explicit choice wins).

Example `package.json` that validates cleanly after a build:

```json
{
  "name": "@acme/widget",
  "version": "1.0.0",
  "type": "module",
  "main": "./dist/index.cjs",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js",
      "require": "./dist/index.cjs"
    }
  },
  "files": ["dist"],
  "peerDependencies": { "react": "^18.0.0" }
}
```

## 3. Configure a private registry (`.npmrc`)

jet reads `.npmrc` with npm precedence (project `.npmrc` overrides
`~/.npmrc` overrides `/etc/npmrc`). Two keys matter for a private scope:

- `@scope:registry=<url>` — route a scoped package name to a registry.
- `//<host>/<path>/:_authToken=<token>` — the Bearer token jet sends as
  `Authorization: Bearer <token>` to that host.

The token host is matched against the registry URL host, so the `//host/...`
prefix of the `_authToken` key must line up with the `@scope:registry` URL.

### GitLab Package Registry

```ini
@acme:registry=https://gitlab.example.com/api/v4/projects/<project-id>/packages/npm/
//gitlab.example.com/api/v4/projects/<project-id>/packages/npm/:_authToken=${GITLAB_TOKEN}
```

### Verdaccio (self-hosted)

```ini
@acme:registry=https://verdaccio.internal.example.com/
//verdaccio.internal.example.com/:_authToken=${VERDACCIO_TOKEN}
```

### Sonatype Nexus

```ini
@acme:registry=https://nexus.example.com/repository/npm-private/
//nexus.example.com/repository/npm-private/:_authToken=${NEXUS_TOKEN}
```

> Never commit raw tokens. Use a CI secret and a `.npmrc` written at build
> time, or shell-expand from the environment. If jet finds no auth token for
> the resolved registry it fails loudly rather than uploading unauthenticated.

## 4. Publish

```bash
# Build first (recommended), choosing access + dist-tag.
jet publish --build --access restricted --tag latest

# Or publish an already-built dist/ (still validated, not built):
jet publish --access public --tag next
```

Flags:

| Flag             | Effect                                                              |
|------------------|--------------------------------------------------------------------|
| `--build`        | Run `jet build --lib` first; auto-fill missing `main`/`module`/`types`. |
| `--access`       | `public` (default) or `restricted` — passed through to the registry. |
| `--tag <tag>`    | Distribution tag (default `latest`, e.g. `next`, `beta`).          |

What `jet publish --build` does, in order:

1. Read and transform `package.json` (resolve `workspace:*` deps).
2. Build the library (`jet build --lib`) → `dist/`.
3. Auto-fill any absent `main`/`module`/`types` from the build output.
4. Validate that `main`/`module`/`types`/`exports` targets all exist.
5. Pack a tarball (skips `node_modules`, `.git`, `patches`, `.jet-cache`).
6. Resolve the scoped registry + Bearer auth token from `.npmrc`.
7. `PUT` the publish envelope (base64 tarball + `dist-tags`) to the registry.

## 5. Pack without publishing

`jet pack` runs the same build/validate path but writes the `.tgz` locally
instead of uploading — useful for inspecting the tarball or a CI dry run.

```bash
jet pack --build      # build + validate + write @acme-widget-1.0.0.tgz
tar -tzf @acme-widget-1.0.0.tgz   # inspect what would ship
```

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `cannot publish: package.json \`main\` points at \`...\`, but no file exists` | `main`/`module`/`types`/`exports` target missing from `dist/` | Run with `--build`, or fix the declared path. |
| `No auth token found for registry ...` | `.npmrc` has no `//host/...:_authToken` matching the scope's registry | Add the `_authToken` line whose host matches the `@scope:registry` URL. |
| `Publish failed (403 ...)` | Token lacks publish scope, or `--access` disallowed | Use a token with package-publish permission; check `restricted` vs `public`. |
| `workspace:* deps ... NOT resolved` warning | Malformed workspace config | Fix `jet-workspace.yaml` / `pnpm-workspace.yaml` / `package.json`. |
<!-- HANDWRITE-END -->
