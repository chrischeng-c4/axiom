#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(scriptDir, '../../../..')

async function resolveJetCommand() {
  if (process.env.CUE_ARTIFACT_STUDIO_JET) {
    return process.env.CUE_ARTIFACT_STUDIO_JET
  }

  const localJet = path.join(repoRoot, 'target/debug/jet')
  try {
    const metadata = await stat(localJet)
    if (metadata.isFile()) return localJet
  } catch {
    // Fall back to PATH when the checkout-local binary has not been built.
  }

  return 'jet'
}

const command = await resolveJetCommand()
const child = spawn(command, process.argv.slice(2), {
  cwd: process.cwd(),
  env: process.env,
  stdio: 'inherit',
})

child.on('error', (error) => {
  console.error(`failed to start ${command}: ${error.message}`)
  process.exit(127)
})

child.on('exit', (code, signal) => {
  if (signal) {
    console.error(`${command} exited with signal ${signal}`)
    process.exit(1)
  }
  process.exit(code ?? 1)
})
