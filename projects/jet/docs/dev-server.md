# Dev Server

Jet includes a development server with hot module reloading (HMR) for instant feedback during development.

## Start the dev server

```bash
cclab jet dev
```

Opens at `http://localhost:3000` by default.

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --port <port>` | Server port | `3000` |
| `--host <host>` | Server host | `127.0.0.1` |

## Hot Module Reloading

HMR is automatic. When you save a file, only the changed module is replaced in the browser — no full page reload needed.

- **JS/TS changes** — Module hot-replaced via WebSocket
- **CSS changes** — Styles updated without page reload
- **New files** — Automatically picked up by the watcher

## Proxy

Route API requests through the dev server to avoid CORS issues. Configure in `jet.config.yaml`:

```yaml
dev:
  port: 3000
  proxy:
    /api: http://localhost:8080
    /auth: http://localhost:8081
```

Requests to `http://localhost:3000/api/users` are forwarded to `http://localhost:8080/api/users`.

## CSS and Tailwind

Jet automatically detects and processes CSS:

- **Tailwind CSS** — Scans source files for used classes, generates only the CSS you need
- **`@import`** — Resolves and inlines CSS imports
- **CSS Modules** — `.module.css` files get scoped class names
- **lightningcss** — Nesting, vendor prefixes, and minification

No configuration needed — Jet reads your `tailwind.config.js` automatically.

## Static files

Files in `public/` are served as-is at the root URL. For example, `public/favicon.ico` is available at `/favicon.ico`.
