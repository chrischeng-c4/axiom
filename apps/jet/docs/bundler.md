# Bundler

Jet bundles JavaScript and TypeScript for production with tree shaking, code splitting, minification, and source maps.

## Basic build

```bash
cclab jet build
```

Outputs to `dist/` by default.

## Options

| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output <dir>` | Output directory | `dist` |
| `--minify` | Enable minification | on |
| `--no-minify` | Disable minification | |
| `--sourcemap <mode>` | Source map mode | `external` |
| `--splitting` | Enable code splitting | off |
| `--define KEY=VALUE` | Compile-time constant (repeatable) | |
| `--drop <type>` | Drop statements (repeatable) | |
| `-w, --watch` | Watch mode | off |
| `--nx` | Build all Nx projects in topological order | |
| `-p, --project <name>` | Build a single Nx project | |

## Source maps

```bash
# External .map files (default)
cclab jet build --sourcemap external

# Inline in bundle
cclab jet build --sourcemap inline

# External file, no comment in bundle
cclab jet build --sourcemap hidden

# No source maps
cclab jet build --sourcemap none
```

## Code splitting

Split your bundle at dynamic `import()` boundaries:

```bash
cclab jet build --splitting
```

```js
// This creates a separate chunk
const module = await import('./heavy-module.js')
```

Shared dependencies used by multiple chunks are automatically extracted into a shared chunk.

## Define replacement

Replace expressions at build time:

```bash
cclab jet build --define NODE_ENV=production --define API_URL=https://api.example.com
```

In your code:

```js
if (process.env.NODE_ENV === 'production') {
  // This block is kept; the else branch is removed by dead code elimination
}
```

`process.env.NODE_ENV` and `import.meta.env.*` are replaced automatically in production builds.

## Drop statements

Remove `console.log` and `debugger` from production builds:

```bash
cclab jet build --drop console --drop debugger
```

## Tree shaking

Tree shaking is always enabled. Unused exports are automatically removed from the bundle.

Jet respects the `sideEffects` field in `package.json`:

```json
{
  "sideEffects": false
}
```

## Output structure

```
dist/
  index.html
  assets/
    main.[hash].js
    vendor.[hash].js     (shared chunk)
    main.[hash].css
    logo.[hash].svg
  main.[hash].js.map     (source map)
```

Output filenames include a content hash for cache busting.

## Output formats

Jet supports three output formats:

- **ESM** (default) — ES modules
- **CJS** — CommonJS
- **IIFE** — Self-executing function (for `<script>` tags)
