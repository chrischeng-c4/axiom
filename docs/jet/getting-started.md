# Getting Started

Jet is a fast JavaScript/TypeScript bundler and package manager built in Rust. It replaces Vite, Webpack, pnpm, and npm with a single tool.

## Prerequisites

- Node.js 18+ (for running JavaScript)
- Rust toolchain (for building from source)

## Installation

```bash
cargo install cclab
```

Once installed, all jet commands are available via `cclab jet`.

## Quick Start

### Create a new project

```bash
cclab jet init
```

### Install dependencies

```bash
cclab jet install
```

This reads `package.json`, resolves all dependencies, and creates a `jet-lock.yaml` lockfile.

### Start dev server

```bash
cclab jet dev
```

Opens a dev server at `http://localhost:3000` with hot module reloading (HMR). Edit your files and see changes instantly.

### Build for production

```bash
cclab jet build
```

Outputs optimized, minified, tree-shaken bundles to `dist/`.

## What's Next

- [Package Manager](./package-manager) — Install, add, remove, and manage dependencies
- [Bundler](./bundler) — Production builds with code splitting and minification
- [Dev Server](./dev-server) — HMR, proxy, and CSS/Tailwind pipeline
- [Task Runner](./task-runner) — Run scripts, execute commands, pipeline tasks
- [Configuration](./configuration) — `jet.config.yaml` and other config files
- [Workspaces](./workspaces) — Monorepo support with Nx and pnpm-workspace.yaml
