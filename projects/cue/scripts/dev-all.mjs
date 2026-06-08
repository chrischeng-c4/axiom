#!/usr/bin/env node
import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const cueRoot = path.resolve(scriptDir, '..')
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm'

const backendHost = process.env.CUE_BACKEND_HOST ?? '127.0.0.1'
const backendPort = process.env.CUE_BACKEND_PORT ?? '43219'
const artifactStudioPort = process.env.CUE_ARTIFACT_STUDIO_PORT ?? '3212'
const adminPort = process.env.CUE_ADMIN_PORT ?? '3216'
const backendMode = process.env.CUE_BACKEND_MODE ?? 'mamba'
const backendUrl = `http://${backendHost}:${backendPort}`

if (!['mamba', 'bridge'].includes(backendMode)) {
  console.error(`Unsupported CUE_BACKEND_MODE=${backendMode}; expected "mamba" or "bridge".`)
  process.exit(2)
}

const backendEntry = backendMode === 'mamba'
  ? {
      name: 'backend:mamba',
      cwd: path.join(cueRoot, 'backend'),
      command: 'mamba',
      args: ['run', '--config', 'mamba.toml'],
      env: {
        CUE_BACKEND_HOST: backendHost,
        CUE_BACKEND_PORT: backendPort,
      },
    }
  : {
      name: 'backend:bridge',
      cwd: path.join(cueRoot, 'backend'),
      command: npmCommand,
      args: ['run', 'dev:bridge'],
      env: {
        CUE_BACKEND_HOST: backendHost,
        CUE_BACKEND_PORT: backendPort,
      },
    }

const entries = [
  backendEntry,
  {
    name: 'artifact-studio',
    cwd: path.join(cueRoot, 'artifact-studio'),
    command: process.execPath,
    args: ['scripts/jet.mjs', 'dev', '--host', '127.0.0.1', '--port', artifactStudioPort],
    env: {
      CUE_ARTIFACT_STUDIO_API_BASE_URL: backendUrl,
    },
  },
  {
    name: 'admin',
    cwd: path.join(cueRoot, 'admin'),
    command: process.execPath,
    args: ['../artifact-studio/scripts/jet.mjs', 'dev', '--host', '127.0.0.1', '--port', adminPort],
    env: {
      CUE_BACKEND_BASE_URL: backendUrl,
      CUE_ADMIN_PORT: adminPort,
    },
  },
]

console.log('Cue dev servers:')
console.log(`  backend         ${backendUrl} (${backendMode})`)
console.log(`  artifact studio http://127.0.0.1:${artifactStudioPort}`)
console.log(`  admin           http://127.0.0.1:${adminPort}`)
if (backendMode === 'mamba') {
  console.log('  bridge fallback CUE_BACKEND_MODE=bridge npm run dev')
}

const children = []
let shuttingDown = false

for (const entry of entries) {
  const child = spawn(entry.command, entry.args, {
    cwd: entry.cwd,
    env: { ...process.env, ...entry.env },
    detached: process.platform !== 'win32',
    stdio: ['ignore', 'pipe', 'pipe'],
  })
  children.push({ entry, child })
  pipe(child.stdout, entry.name, process.stdout)
  pipe(child.stderr, entry.name, process.stderr)

  child.on('error', (error) => {
    console.error(`[${entry.name}] failed to start: ${error.message}`)
    shutdown(1)
  })

  child.on('exit', (code, signal) => {
    if (shuttingDown) return
    const status = signal ? `signal ${signal}` : `code ${code ?? 1}`
    console.error(`[${entry.name}] exited with ${status}`)
    if (entry.name === 'backend:mamba' && code === 0) {
      console.error(
        '[backend:mamba] Mamba exited without holding the API server open. ' +
          'Use CUE_BACKEND_MODE=bridge only as an explicit temporary fallback while mamba HTTP dispatch catches up.',
      )
    }
    shutdown(code === 0 ? 1 : code ?? 1)
  })
}

process.on('SIGINT', () => shutdown(0))
process.on('SIGTERM', () => shutdown(0))

function pipe(stream, label, target) {
  let buffer = ''
  stream.on('data', (chunk) => {
    buffer += chunk.toString()
    const lines = buffer.split(/\r?\n/)
    buffer = lines.pop() ?? ''
    for (const line of lines) {
      if (line.length > 0) target.write(`[${label}] ${line}\n`)
    }
  })
}

function shutdown(code) {
  if (shuttingDown) return
  shuttingDown = true
  process.exitCode = code
  for (const { child } of children) {
    if (child.exitCode !== null) continue
    try {
      if (process.platform === 'win32') {
        child.kill('SIGTERM')
      } else {
        process.kill(-child.pid, 'SIGTERM')
      }
    } catch {
      // Process may already have exited.
    }
  }
  setTimeout(() => process.exit(code), 500).unref()
}
