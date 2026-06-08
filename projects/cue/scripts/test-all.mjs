import { spawn } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const cueRoot = path.resolve(__dirname, '..')
const repoRoot = path.resolve(cueRoot, '../..')

const steps = [
  {
    name: 'backend contract tests',
    cwd: repoRoot,
    command: 'uv',
    args: ['run', '--with', 'pytest', 'python', '-m', 'pytest', 'projects/cue/backend/tests/test_workstream_api.py', '-q'],
  },
  {
    name: 'Artifact Studio typecheck',
    cwd: path.join(cueRoot, 'artifact-studio'),
    command: 'npm',
    args: ['run', 'typecheck'],
  },
  {
    name: 'Artifact Studio browser e2e',
    cwd: path.join(cueRoot, 'artifact-studio'),
    command: 'npm',
    args: ['run', 'test:e2e'],
  },
  {
    name: 'Cue full product e2e',
    cwd: cueRoot,
    command: 'npm',
    args: ['run', 'test:e2e:full'],
  },
]

for (const step of steps) {
  console.log(`\n==> ${step.name}`)
  await run(step)
}

console.log('\nCue test suite completed.')

function run(step) {
  return new Promise((resolve, reject) => {
    const child = spawn(step.command, step.args, {
      cwd: step.cwd,
      env: process.env,
      stdio: 'inherit',
    })
    child.on('error', reject)
    child.on('exit', (code, signal) => {
      if (code === 0) {
        resolve()
        return
      }
      reject(new Error(`${step.name} failed with ${signal || code}`))
    })
  })
}
