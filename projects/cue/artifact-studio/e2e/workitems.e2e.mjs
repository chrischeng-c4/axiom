import { spawn } from 'node:child_process'
import { mkdir, mkdtemp, rm, stat, writeFile } from 'node:fs/promises'
import http from 'node:http'
import { tmpdir } from 'node:os'
import path from 'node:path'
import net from 'node:net'

const chromePath =
  process.env.CUE_ARTIFACT_STUDIO_CHROME ??
  '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
const e2ePort = process.env.CUE_ARTIFACT_STUDIO_E2E_PORT ?? '3214'
const serverTimeoutMs = Number(process.env.CUE_ARTIFACT_STUDIO_SERVER_TIMEOUT_MS ?? '300000')
const baseUrl = process.env.CUE_ARTIFACT_STUDIO_BASE_URL ?? `http://127.0.0.1:${e2ePort}`
const jetScript = path.resolve(import.meta.dirname, '../scripts/jet.mjs')
let appUrl = baseUrl

const tests = [
  {
    name: 'keeps WorkItems summary owner-facing before expanding details',
    run: async (page) => {
      await page.goto(appUrl)
      const body = await page.text()
      assertContains(body, 'Artifact Studio')
      assertContains(body, 'Team Request Tracker')
      assertContains(body, 'Request tracker intake')
      assertContains(body, 'Policy review follow-up')
      assertContains(body, 'Owner handoff')
      assertNotContains(body, 'Archived import cleanup')
      assertContains(body, 'Project workstream')
      assertContains(body, 'WorkItems 1/2')
      assertNotContains(body, 'Create request tracker PRD')
      assertNotContains(body, 'Hidden GitLab project')
      assertNotContains(body, 'Runtime tenant')
    },
  },
  {
    name: 'opens the WorkItems pane with route target status and progress',
    run: async (page) => {
      await page.goto(appUrl)
      await page.clickButton('WorkItems 1/2')
      await page.waitForText('Hide WorkItems')
      const body = await page.text()
      assertContains(body, 'Hide WorkItems')
      assertContains(body, '1 of 2 ready')
      assertContains(body, 'Create request tracker PRD')
      assertContains(body, 'prompt-to-PRD to PRD')
      assertContains(body, 'Accepted')
      assertContains(body, 'QC pass')
      assertContains(body, '100%')
      assertContains(body, 'Confirm data retention')
      assertContains(body, 'Needs info')
      assertContains(body, 'QC needs input')
      assertContains(body, 'Answer retention question')
      assertContains(body, '62%')
    },
  },
  {
    name: 'updates WorkItems summary when the owner switches project',
    run: async (page) => {
      await page.goto(appUrl)
      await page.clickButton('Weekly Ops Report')
      await page.waitForText('WorkItems 0/1')
      await page.clickButton('WorkItems 0/1')
      await page.waitForText('Collect report basics')
      const body = await page.text()
      assertContains(body, 'Weekly Ops Report')
      assertContains(body, 'Complete WorkItem')
      assertContains(body, '0 of 1 ready')
      assertContains(body, 'Collect report basics')
      assertContains(body, 'prompt-to-WorkItem to WorkItem')
      assertContains(body, 'Add recipients and data source')
    },
  },
  {
    name: 'redirects general chat without creating a WorkItem',
    run: async (page) => {
      await page.goto(appUrl)
      await page.fillPrompt('你好，今天天氣如何？')
      await page.clickButton('Send')
      await page.waitForText('一般聊天')
      const body = await page.text()
      assertContains(body, '不會建立 WorkItem')
      assertNotContains(body, 'Generate website workflow')
    },
  },
  {
    name: 'turns a website prompt into a WorkItem workflow plan',
    run: async (page) => {
      await page.goto(appUrl)
      await page.fillPrompt('幫我產生一個 marketing 網站，收集 lead 並追蹤成效。')
      await page.clickButton('Send')
      await page.waitForText('Generate website workflow')
      const body = await page.text()
      assertContains(body, 'Workflow graph')
      assertContains(body, '3 nodes')
      assertContains(body, 'PRD')
      assertContains(body, 'TD')
      assertContains(body, 'Website')
      assertContains(body, 'PM agent')
      assertContains(body, 'Architect agent')
      assertContains(body, 'Designer agent')
      assertContains(body, 'Create PRD')
      assertContains(body, 'Quality gate')
      assertContains(body, 'QC pass')
    },
  },
  {
    name: 'creates a PRD artifact from an accepted WorkItem',
    run: async (page) => {
      await page.goto(appUrl)
      await page.fillPrompt('幫我產生一個 website artifact，需要 PRD TD 和網站。')
      await page.clickButton('Send')
      await page.waitForText('Generate website workflow')
      await page.waitForText('Create PRD')
      await page.clickButton('Create PRD')
      await page.waitForText('Generate website workflow PRD')
      const body = await page.text()
      assertContains(body, 'Generate website workflow PRD')
      assertContains(body, 'Done')
      assertContains(body, 'QC pass')
      assertContains(body, 'PRD artifact is complete.')
    },
  },
  {
    name: 'creates a todo app project and completes full delivery through UI',
    run: async (page) => {
      await page.goto(appUrl)
      await page.clickButton('New Project')
      await page.waitForText('Todo App Project')
      await page.fillPrompt('Create a todo app for Operations with title owner status and due date.')
      await page.clickButton('Send')
      await page.waitForText('Create todo app')
      await page.waitForText('Create PRD')
      await page.waitForText('PM agent')
      await page.waitForText('Architect agent')
      await page.waitForText('Dev agent')
      await page.waitForText('QA/policy agent')
      await page.waitForText('Release agent')
      await page.waitForText('Data agent')
      assertEqual(await page.evaluate(`document.querySelectorAll('[data-workflow-node]').length`), 6, 'workflow DAG node count')
      assertEqual(await page.evaluate(`document.querySelectorAll('[data-workflow-edge]').length`), 5, 'workflow DAG edge count')
      const desktopLayout = await page.evaluate(`(() => {
        const conversation = document.querySelector('[aria-label="Project conversation"]');
        const context = document.querySelector('[aria-label="Project status"]');
        return {
          conversationWidth: Math.round(conversation?.getBoundingClientRect().width ?? 0),
          contextWidth: Math.round(context?.getBoundingClientRect().width ?? 0),
          viewportWidth: window.innerWidth,
        };
      })()`)
      assertGreater(desktopLayout.contextWidth, desktopLayout.conversationWidth, 'right context pane width')

      const expectedArtifacts = [
        ['Create PRD', 'Create todo app PRD'],
        ['Create TD', 'Create todo app TD'],
        ['Create CODEBASE', 'Create todo app CODEBASE'],
        ['Create TEST', 'Create todo app TEST'],
        ['Create DEPLOYMENT', 'Create todo app DEPLOYMENT'],
        ['Create OPERATION', 'Create todo app OPERATION'],
      ]
      for (const [button, artifactText] of expectedArtifacts) {
        await page.clickButton(button)
        await page.waitForText(artifactText)
      }

      await page.waitForText('Operations dashboard')
      await page.waitForText('sandbox-v1')
      await page.waitForText('OPEN BLOCKERS')
      const body = await page.text()
      assertContains(body, 'Todo app sandbox is ready')
      assertContains(body, 'HEALTH')
      assertContains(body, 'ok')
      assertContains(body, 'TESTS')
      assertContains(body, 'passed')
      assertContains(body, 'ENVIRONMENT')
      assertContains(body, 'sandbox')
      assertNotContains(body, 'GitLab')
      assertNotContains(body, 'GCP')
      assertNotContains(body, 'Terraform')
      assertNotContains(body, 'Kustomize')
      assertNotContains(body, 'branch')
      assertNotContains(body, 'commit')

      const evidence = await page.evaluate(`
        fetch('/api/debug/projects/todo-app-project/evidence')
          .then((response) => response.json())
      `)
      assertEqual(new Set(evidence.agents.map((agent) => agent.agent_role)).size, 6, 'distinct workflow node agents')
      assertContains(evidence.hidden_repo_path, 'todo-app-project')
      assertContains(evidence.files.join('\\n'), 'infra/terraform/main.tf')
      assertContains(evidence.files.join('\\n'), 'deploy/kustomize/base/deployment.yaml')
      assertContains(evidence.files.join('\\n'), '.gitlab-ci.yml')
    },
  },
]

async function main() {
  let appServer
  let chrome
  let browser
  let userDataDir
  let apiServer

  try {
    let apiBaseUrl = process.env.CUE_ARTIFACT_STUDIO_API_BASE_URL
    if (!apiBaseUrl) {
      const fixturePort = Number(process.env.CUE_ARTIFACT_STUDIO_FIXTURE_API_PORT ?? 43219)
      const fixture = await startFixtureApi(fixturePort)
      apiServer = fixture.server
      apiBaseUrl = fixture.url
    }

    if (!process.env.CUE_ARTIFACT_STUDIO_BASE_URL) {
      appServer = spawn(process.execPath, [jetScript, 'dev', '--host', '127.0.0.1', '--port', e2ePort], {
        cwd: path.resolve(import.meta.dirname, '..'),
        env: { ...process.env, CUE_ARTIFACT_STUDIO_API_BASE_URL: apiBaseUrl, CUE_ARTIFACT_STUDIO_E2E_PORT: e2ePort },
        stdio: ['ignore', 'pipe', 'pipe'],
      })
      appServer.stdout.on('data', (chunk) => process.stdout.write(`[jet] ${chunk}`))
      appServer.stderr.on('data', (chunk) => process.stderr.write(`[jet] ${chunk}`))
    }

    await waitForHttp(baseUrl, serverTimeoutMs)

    userDataDir = await mkdtemp(path.join(tmpdir(), 'cue-artifact-studio-e2e-'))
    const cdpPort = await findFreePort()
    chrome = spawn(chromePath, [
      `--remote-debugging-port=${cdpPort}`,
      `--user-data-dir=${userDataDir}`,
      '--headless=new',
      '--no-first-run',
      '--no-default-browser-check',
      '--disable-background-networking',
      '--disable-default-apps',
      '--disable-extensions',
      '--disable-sync',
      '--disable-translate',
      '--metrics-recording-only',
      '--mute-audio',
      '--no-sandbox',
    ])
    const chromeDiagnostics = captureChildDiagnostics(chrome, 'chrome')

    const wsUrl = await waitForChrome(cdpPort, 30_000, () => chromeDiagnostics.summary())
    browser = await connectCdp(wsUrl)
    const page = await browser.newPage()
    await page.setViewport(1440, 900)

    let passed = 0
    for (const testCase of tests) {
      page.clearErrors()
      try {
        await testCase.run(page)
        page.assertNoConsoleErrors()
        passed += 1
        console.log(`PASS ${testCase.name}`)
      } catch (error) {
        console.error(`FAIL ${testCase.name}`)
        throw error
      }
    }

    console.log(`E2E: ${passed}/${tests.length} passed`)
  } finally {
    await browser?.close()
    await stopProcess(chrome)
    await stopProcess(appServer)
    await closeHttpServer(apiServer)
    if (userDataDir) {
      await rm(userDataDir, { recursive: true, force: true }).catch(() => {})
    }
  }
}

async function connectCdp(wsUrl) {
  const socket = new WebSocket(wsUrl)
  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true })
    socket.addEventListener('error', reject, { once: true })
  })

  let nextId = 0
  const pending = new Map()
  const handlers = new Map()

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data)
    if (message.id !== undefined) {
      const request = pending.get(message.id)
      pending.delete(message.id)
      if (!request) return
      if (message.error) {
        request.reject(new Error(message.error.message))
      } else {
        request.resolve(message.result ?? {})
      }
      return
    }

    const key = eventKey(message.method, message.sessionId)
    for (const handler of handlers.get(key) ?? []) {
      handler(message.params ?? {})
    }
  })

  function send(method, params = {}, sessionId) {
    const id = ++nextId
    const payload = { id, method, params }
    if (sessionId) payload.sessionId = sessionId
    socket.send(JSON.stringify(payload))
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject })
    })
  }

  function on(method, sessionId, handler) {
    const key = eventKey(method, sessionId)
    const current = handlers.get(key) ?? []
    current.push(handler)
    handlers.set(key, current)
  }

  return {
    async newPage() {
      const target = await send('Target.createTarget', { url: 'about:blank' })
      const attached = await send('Target.attachToTarget', {
        targetId: target.targetId,
        flatten: true,
      })
      const sessionId = attached.sessionId
      const page = new CdpPage(send, on, sessionId)
      await page.init()
      return page
    },
    async close() {
      if (socket.readyState === WebSocket.CLOSED) return
      const closed = new Promise((resolve) => socket.addEventListener('close', resolve, { once: true }))
      socket.close()
      await Promise.race([closed, sleep(1_000)])
    },
  }
}

class CdpPage {
  #errors = []

  constructor(send, on, sessionId) {
    this.send = send
    this.on = on
    this.sessionId = sessionId
  }

  async init() {
    this.on('Runtime.consoleAPICalled', this.sessionId, (params) => {
      if (params.type !== 'error') return
      const text = params.args?.map((arg) => arg.value ?? arg.description ?? '').join(' ') ?? ''
      if (text.includes('Failed to load resource')) return
      this.#errors.push(text)
    })
    this.on('Runtime.exceptionThrown', this.sessionId, (params) => {
      const details = params.exceptionDetails
      const exception = details?.exception
      const frames = details?.stackTrace?.callFrames ?? []
      const frame = frames[0]
      const location = frame ? ` at ${frame.url}:${frame.lineNumber + 1}:${frame.columnNumber + 1}` : ''
      this.#errors.push(exception?.description ?? exception?.value ?? `${details?.text ?? 'Runtime exception'}${location}`)
    })
    await this.send('Runtime.enable', {}, this.sessionId)
    await this.send('Page.enable', {}, this.sessionId)
  }

  async setViewport(width, height) {
    await this.send(
      'Emulation.setDeviceMetricsOverride',
      {
        width,
        height,
        deviceScaleFactor: 1,
        mobile: false,
      },
      this.sessionId,
    )
  }

  clearErrors() {
    this.#errors = []
  }

  assertNoConsoleErrors() {
    if (this.#errors.length > 0) {
      throw new Error(`Unexpected browser errors:\n${this.#errors.join('\n')}`)
    }
  }

  errorSummary() {
    if (this.#errors.length === 0) return ''
    return `\nBrowser errors:\n${this.#errors.join('\n')}`
  }

  async goto(url) {
    const loaded = onceEvent(this.on, 'Page.loadEventFired', this.sessionId)
    await this.send('Page.navigate', { url }, this.sessionId)
    await withTimeout(loaded, 30_000, `Timed out loading ${url}`)
    await this.waitForText('Artifact Studio')
    await this.waitForText('Project workstream')
  }

  async text() {
    return this.evaluate('document.body ? document.body.innerText : ""')
  }

  async clickButton(text) {
    const clicked = await this.evaluate(
      `(function () {
        const candidates = Array.from(document.querySelectorAll('button, [role="button"]')).filter((candidate) =>
          candidate.textContent && !candidate.disabled
        );
        const button = candidates.find((candidate) => candidate.textContent.trim() === ${JSON.stringify(text)}) ??
          candidates.find((candidate) => candidate.textContent.includes(${JSON.stringify(text)}));
        if (!button) return false;
        button.click();
        return true;
      })()`,
    )
    if (!clicked) {
      throw new Error(`Button not found: ${text}`)
    }
  }

  async fillPrompt(text) {
    const filled = await this.evaluate(
      `(function () {
        const input = Array.from(document.querySelectorAll('input, textarea')).find((candidate) =>
          candidate.placeholder && candidate.placeholder.includes('Describe a request')
        );
        if (!input) return false;
        const setter = Object.getOwnPropertyDescriptor(Object.getPrototypeOf(input), 'value')?.set;
        if (!setter) return false;
        setter.call(input, ${JSON.stringify(text)});
        input.dispatchEvent(new Event('input', { bubbles: true }));
        return true;
      })()`,
    )
    if (!filled) {
      throw new Error('Prompt input not found')
    }
  }

  async waitForText(text) {
    await waitFor(async () => (await this.text()).includes(text), 5_000, () => `Timed out waiting for ${text}${this.errorSummary()}`)
  }

  async evaluate(expression) {
    const result = await this.send(
      'Runtime.evaluate',
      {
        expression,
        returnByValue: true,
        awaitPromise: true,
      },
      this.sessionId,
    )
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.text ?? 'Evaluation failed')
    }
    return result.result?.value
  }
}

function eventKey(method, sessionId) {
  return `${sessionId ?? 'browser'}:${method}`
}

function onceEvent(on, method, sessionId) {
  return new Promise((resolve) => {
    on(method, sessionId, resolve)
  })
}

async function waitForHttp(url, timeoutMs) {
  await waitFor(async () => {
    try {
      const response = await fetch(url)
      return response.ok
    } catch {
      return false
    }
  }, timeoutMs, `Timed out waiting for ${url}`)
}

async function waitForChrome(port, timeoutMs, diagnostics = () => '') {
  let wsUrl
  await waitFor(async () => {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/version`)
      const body = await response.json()
      wsUrl = body.webSocketDebuggerUrl
      return Boolean(wsUrl)
    } catch {
      return false
    }
  }, timeoutMs, () => `Timed out waiting for Chrome on ${port}${diagnostics()}`)
  return wsUrl
}

function captureChildDiagnostics(child, label) {
  const lines = []
  let exitMessage = ''

  child?.stderr?.on('data', (chunk) => {
    for (const line of chunk.toString().split(/\r?\n/)) {
      if (!line.trim()) continue
      lines.push(line)
    }
    while (lines.length > 20) lines.shift()
  })

  child?.on('error', (error) => {
    exitMessage = `${label} spawn error: ${error.message}`
  })

  child?.on('exit', (code, signal) => {
    exitMessage = `${label} exited before readiness: code=${code ?? 'null'} signal=${signal ?? 'null'}`
  })

  return {
    summary() {
      const parts = []
      if (exitMessage) parts.push(exitMessage)
      if (lines.length > 0) parts.push(`${label} stderr tail:\n${lines.join('\n')}`)
      return parts.length > 0 ? `\n${parts.join('\n')}` : ''
    },
  }
}

async function waitFor(check, timeoutMs, message) {
  const started = Date.now()
  while (Date.now() - started < timeoutMs) {
    if (await check()) return
    await sleep(100)
  }
  throw new Error(typeof message === 'function' ? message() : message)
}

function withTimeout(promise, timeoutMs, message) {
  return Promise.race([
    promise,
    sleep(timeoutMs).then(() => {
      throw new Error(message)
    }),
  ])
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

async function stopProcess(child) {
  if (!child || child.killed) return
  child.kill()
  await Promise.race([new Promise((resolve) => child.once('exit', resolve)), sleep(1_000)])
}

async function closeHttpServer(server) {
  if (!server) return
  await new Promise((resolve) => server.close(resolve))
}

async function startFixtureApi(port = undefined) {
  const workspace = await mkdtemp(path.join(tmpdir(), 'cue-artifact-studio-fixture-'))
  const state = fixtureState(workspace)
  const apiPort = port ?? await findFreePort()
  const server = http.createServer(async (request, response) => {
    response.setHeader('Access-Control-Allow-Origin', '*')
    response.setHeader('Access-Control-Allow-Headers', 'content-type')
    response.setHeader('Access-Control-Allow-Methods', 'GET,POST,OPTIONS')
    response.setHeader('Content-Type', 'application/json')
    if (request.method === 'OPTIONS') {
      response.writeHead(204)
      response.end()
      return
    }

    const url = new URL(request.url ?? '/', `http://${request.headers.host}`)
    const body = request.method === 'POST' ? await readJsonBody(request) : {}
    const result = await handleFixtureRequest(state, request.method ?? 'GET', url.pathname, body)
    response.writeHead(result.status ?? 200)
    response.end(JSON.stringify(result.body))
  })
  server.on('close', () => {
    void rm(workspace, { recursive: true, force: true })
  })
  await new Promise((resolve, reject) => {
    const onError = (error) => reject(error)
    server.once('error', onError)
    server.listen(apiPort, '127.0.0.1', () => {
      server.off('error', onError)
      resolve()
    })
  })
  const address = server.address()
  const boundPort = address && typeof address === 'object' ? address.port : apiPort
  return { server, url: `http://127.0.0.1:${boundPort}` }
}

async function handleFixtureRequest(state, method, pathname, body) {
  if (method === 'GET' && pathname === '/api/projects') {
    return { body: { projects: Object.values(state.projects) } }
  }

  if (method === 'POST' && pathname === '/api/projects') {
    return { body: { project: await createFixtureProject(state, body) } }
  }

  const messageMatch = pathname.match(/^\/api\/sessions\/([^/]+)\/messages$/)
  if (method === 'POST' && messageMatch) {
    return { body: postFixtureMessage(state, messageMatch[1], body.content ?? '') }
  }

  const contextMatch = pathname.match(/^\/api\/workitems\/([^/]+)\/context$/)
  if (method === 'GET' && contextMatch) {
    return { body: fixtureContext(state, contextMatch[1]) }
  }

  const debugMatch = pathname.match(/^\/api\/debug\/projects\/([^/]+)\/evidence$/)
  if (method === 'GET' && debugMatch) {
    return { body: await debugProjectEvidence(state, debugMatch[1]) }
  }

  const artifactRunMatch = pathname.match(/^\/api\/workitems\/([^/]+)\/artifact-runs$/)
  if (method === 'POST' && artifactRunMatch) {
    return { body: await runFixtureArtifact(state, artifactRunMatch[1], body.kind ?? 'prd') }
  }

  return { status: 404, body: { error: { code: 'not_found' } } }
}

async function debugProjectEvidence(state, projectId) {
  const project = state.projects[projectId]
  if (!project?.hidden_repo_path) return { hidden_repo_path: null, files: [] }
  const files = [
    'prd.md',
    'td.md',
    'app-spec.json',
    'src/todo-app.ts',
    'tests/todo-app.test.json',
    '.gitlab-ci.yml',
    'deploy/kustomize/base/deployment.yaml',
    'deploy/kustomize/overlays/sandbox/kustomization.yaml',
    'deploy/kustomize/overlays/production/kustomization.yaml',
    'infra/terraform/main.tf',
    'releases/sandbox.json',
    'operations/dashboard.json',
  ]
  const existing = []
  for (const file of files) {
    try {
      await stat(path.join(project.hidden_repo_path, file))
      existing.push(file)
    } catch {
      // Missing files are represented by absence from the evidence list.
    }
  }
  return {
    hidden_repo_path: project.hidden_repo_path,
    local_gitlab_simulation: true,
    local_gcp_simulation: true,
    files: existing,
    agents: (project.workitems[0]?.workflow_plan ?? []).map((step) => ({
      id: step.id,
      agent_role: step.agent_role,
      agent_label: step.agent_label,
    })),
  }
}

async function createFixtureProject(state, body) {
  const projectId = 'todo-app-project'
  if (state.projects[projectId]) return state.projects[projectId]
  const hiddenRepoPath = path.join(state.workspace, 'hidden-repos', projectId)
  await mkdir(hiddenRepoPath, { recursive: true })
  await writeJson(path.join(hiddenRepoPath, 'cue-project.json'), {
    id: projectId,
    name: 'Todo App Project',
    prompt: body.prompt ?? 'Create a todo app for the Operations team.',
    hidden_repo: 'local-folder',
  })
  const project = {
    id: projectId,
    name: body.name ?? 'Todo App Project',
    owner: 'Operations',
    status: 'in-progress',
    next_action: 'Create WorkItem',
    summary: 'Todo app project is ready for guided delivery. Sandbox and operations status will appear here.',
    active_session_id: 'session-todo-app-project',
    hidden_repo_path: hiddenRepoPath,
    admin_evidence: {
      hidden_repo_path: hiddenRepoPath,
      local_gitlab_simulation: true,
      local_gcp_simulation: true,
    },
    sessions: [
      {
        id: 'session-todo-app-project',
        project_id: projectId,
        title: 'Todo app intake',
        messages: [
          message('m1', 'cue', 'Todo App Project 已建立。請描述你要交付的工作。'),
        ],
      },
    ],
    stages: [
      projectStage('prd', 'PRD', 'not-started', 'Starts after WorkItem creation.'),
      projectStage('td', 'TD', 'not-started', 'Locked by PRD.'),
      projectStage('codebase', 'Codebase', 'not-started', 'App source and delivery assets are locked by TD.'),
      projectStage('test', 'Test', 'not-started', 'Locked by codebase.'),
      projectStage('deployment', 'Deployment', 'not-started', 'Locked by tests.'),
      projectStage('operation', 'Operation', 'not-started', 'Operations dashboard is locked by deployment.'),
    ],
    workitems: [],
    artifacts: [],
  }
  state.projects[projectId] = project
  return project
}

const workflowNodeAgents = {
  prd: {
    agent_role: 'pm',
    agent_label: 'PM agent',
    agent_task: 'Shape the owner goal, users, constraints, and success criteria.',
  },
  td: {
    agent_role: 'architect',
    agent_label: 'Architect agent',
    agent_task: 'Turn the approved PRD into system contracts and delivery design.',
  },
  website: {
    agent_role: 'designer',
    agent_label: 'Designer agent',
    agent_task: 'Produce the owner-facing website artifact after TD approval.',
  },
  codebase: {
    agent_role: 'dev',
    agent_label: 'Dev agent',
    agent_task: 'Implement source, schemas, delivery assets, and generated artifacts.',
  },
  test: {
    agent_role: 'qa_policy',
    agent_label: 'QA/policy agent',
    agent_task: 'Run quality, policy, and workflow acceptance checks.',
  },
  deployment: {
    agent_role: 'release',
    agent_label: 'Release agent',
    agent_task: 'Prepare sandbox release evidence and promotion gates.',
  },
  operation: {
    agent_role: 'data',
    agent_label: 'Data agent',
    agent_task: 'Maintain runtime metrics and operations dashboard evidence.',
  },
}

function workflowStep(id, label, state, depends_on = []) {
  return { id, label, state, depends_on, ...(workflowNodeAgents[id] ?? {}) }
}

function projectStage(id, label, state, detail) {
  return { id, label, state, detail, ...(workflowNodeAgents[id] ?? {}) }
}

function websiteWorkflowPlan(firstState = 'ready') {
  return [
    workflowStep('prd', 'PRD', firstState, []),
    workflowStep('td', 'TD', 'not-started', ['prd']),
    workflowStep('website', 'Website', 'not-started', ['td']),
  ]
}

function postFixtureMessage(state, sessionId, content) {
  const project = projectForSession(state, sessionId)
  const session = project.sessions.find((candidate) => candidate.id === sessionId)
  session.messages.push(message(`m${session.messages.length + 1}`, 'owner', content))
  if (content.includes('天氣') || content.toLowerCase().includes('hello')) {
    const cue = message(
      `m${session.messages.length + 1}`,
      'cue',
      '這看起來是一般聊天，不會建立 WorkItem。請描述一個專案目標、工作流程或要產生的 artifact。',
    )
    session.messages.push(cue)
    return {
      classification: 'general_chat_redirect',
      message: cue,
      project,
      session,
      context: { type: 'project_overview', project_id: project.id, next_action: project.next_action },
    }
  }

  const isTodo = project.id === 'todo-app-project' || content.toLowerCase().includes('todo')
  const id = isTodo ? 'todo-app-workitem' : `${project.id}-generate-website-workflow`
  let workitem = project.workitems.find((candidate) => candidate.id === id)
  if (!workitem) {
    workitem = {
      id,
      project_id: project.id,
      title: isTodo ? 'Create todo app' : 'Generate website workflow',
      route: 'prompt-to-WorkItem',
      target: isTodo ? 'Todo App' : 'Website',
      state: 'accepted',
      progress: isTodo ? 0 : 55,
      next_action: 'Create PRD artifact',
      blockers: [],
      workflow_plan: isTodo ? todoWorkflowPlan() : websiteWorkflowPlan('ready'),
      qc_status: 'pass',
      qc_checks: [
        { id: 'intent', label: 'Intent is project work', status: 'pass', summary: isTodo ? 'Prompt requests a governed todo app delivery workflow.' : 'Prompt requests a governed website artifact workflow.' },
        { id: 'artifact_graph', label: 'Artifact graph', status: 'pass', summary: isTodo ? 'PRD, TD, Codebase, Test, Deployment, and Operation dependencies are present.' : 'PRD, TD, and Website dependencies are present.' },
      ],
    }
    project.workitems.push(workitem)
  }
  project.summary = isTodo ? 'WorkItem accepted; PRD -> TD -> Codebase -> Test -> Deployment -> Operation workflow is ready.' : 'WorkItem accepted; PRD -> TD -> Website workflow is ready.'
  project.next_action = 'Create PRD artifact'
  const cue = message(
    `m${session.messages.length + 1}`,
    'cue',
    isTodo
      ? '我已把這段 prompt 收成 WorkItem：Create todo app。目前 workflow 是 PRD -> TD -> Codebase -> Test -> Deployment -> Operation，下一步是 Create PRD artifact。'
      : '我已把這段 prompt 收成 WorkItem：Generate website workflow。目前 workflow 是 PRD -> TD -> Website，下一步是 Create PRD artifact。',
    'Open WorkItem',
  )
  session.messages.push(cue)
  return {
    classification: 'project_work',
    message: cue,
    project,
    session,
    workitem,
    context: fixtureContext(state, workitem.id),
  }
}

async function runFixtureArtifact(state, workitemId, kind) {
  const project = projectForWorkItem(state, workitemId)
  const workitem = project.workitems.find((candidate) => candidate.id === workitemId)
  if (workitem.state !== 'accepted' && workitem.state !== 'done') {
    return {
      status: 'rejected',
      reason: 'workitem_not_accepted',
      message: 'WorkItem must be accepted before creating artifacts.',
      project,
      context: fixtureContext(state, workitemId),
    }
  }
  const nextArtifactKind = nextWorkflowArtifactKind(workitem)
  if (kind !== nextArtifactKind) {
    return {
      status: 'rejected',
      reason: 'workflow_dependency_not_done',
      message: nextArtifactKind
        ? `${kind.toUpperCase()} is locked until ${nextArtifactKind.toUpperCase()} is done.`
        : 'Workflow is already complete.',
      project,
      context: fixtureContext(state, workitemId),
    }
  }
  const artifactId = `${workitemId}-${kind}-v1`
  const step = workitem.workflow_plan.find((candidate) => candidate.id === kind)
  const agentLabel = step?.agent_label ?? 'Cue agent'
  if (!project.artifacts.some((artifact) => artifact.id === artifactId)) {
    await writeArtifactEvidence(project, workitem, kind)
    project.artifacts.push({
      id: artifactId,
      workitem_id: workitemId,
      label: `${workitem.title} ${kind.toUpperCase()}`,
      kind,
      status: 'Done',
      summary: `${kind.toUpperCase()} artifact is complete. ${agentLabel} handled this node.`,
      agent_role: step?.agent_role,
      agent_label: step?.agent_label,
      qc_status: 'pass',
      qc_checks: [
        {
          id: `${kind}_review`,
          label: `${kind.toUpperCase()} owner review`,
          status: 'pass',
          summary: `${kind.toUpperCase()} artifact passed the workflow gate.`,
        },
      ],
      versions: [{ id: artifactId, version: 1, status: 'current' }],
    })
  }
  workitem.workflow_plan = workitem.workflow_plan.map((step) => {
    if (step.id === kind) return { ...step, state: 'done' }
    return step
  })
  const afterKind = nextWorkflowArtifactKind(workitem)
  workitem.next_action = afterKind ? `Create ${afterKind.toUpperCase()} artifact` : 'Workflow complete'
  workitem.state = afterKind ? 'accepted' : 'done'
  workitem.progress = workflowProgress(workitem)
  for (const stage of project.stages) {
    if (stage.id === kind) {
      stage.state = 'done'
      stage.detail = `${kind.toUpperCase()} artifact is complete. ${agentLabel} handled this node.`
    } else if (stage.id === afterKind) {
      stage.state = 'ready'
      stage.detail = `${afterKind.toUpperCase()} can start now that dependencies are done.`
    }
  }
  project.next_action = workitem.next_action
  project.status = afterKind ? 'in-progress' : 'ready'
  if (!afterKind && project.id === 'todo-app-project') {
    project.summary = 'Todo app sandbox is ready. Health is ok, tests passed, and operations dashboard has no blockers.'
    project.operations_dashboard = {
      health: 'ok',
      latest_release: 'sandbox-v1',
      ci_status: 'passed',
      deployment_environment: 'sandbox',
      open_blockers: [],
      runtime_metrics: { todo_count: 3, completed_count: 1, error_rate: 0 },
    }
  }
  return { status: 'created', project, context: fixtureContext(state, workitemId) }
}

function todoWorkflowPlan() {
  return [
    workflowStep('prd', 'PRD', 'ready', []),
    workflowStep('td', 'TD', 'not-started', ['prd']),
    workflowStep('codebase', 'Codebase', 'not-started', ['td']),
    workflowStep('test', 'Test', 'not-started', ['codebase']),
    workflowStep('deployment', 'Deployment', 'not-started', ['test']),
    workflowStep('operation', 'Operation', 'not-started', ['deployment']),
  ]
}

async function writeArtifactEvidence(project, workitem, kind) {
  if (!project.hidden_repo_path) return
  const writers = {
    prd: () => writeText(project.hidden_repo_path, 'prd.md', '# Todo App PRD\n\nCreate a governed todo app for Operations.\n'),
    td: () => writeText(project.hidden_repo_path, 'td.md', '# Todo App TD\n\nTodoItem fields: title, owner, status, due_date.\n'),
    codebase: async () => {
      await writeJson(path.join(project.hidden_repo_path, 'app-spec.json'), { schema_version: 'cue.app-spec.v0', app_id: 'todo-app', fields: ['title', 'owner', 'status', 'due_date'] })
      await writeText(project.hidden_repo_path, 'src/todo-app.ts', 'export const todoApp = { name: "todo-app" }\n')
      await writeText(project.hidden_repo_path, '.gitlab-ci.yml', 'stages:\n  - test\n  - deploy\n')
      await writeText(project.hidden_repo_path, 'deploy/kustomize/base/deployment.yaml', 'kind: Deployment\nmetadata:\n  name: todo-app\n')
      await writeText(project.hidden_repo_path, 'deploy/kustomize/overlays/sandbox/kustomization.yaml', 'resources:\n  - ../../base\n')
      await writeText(project.hidden_repo_path, 'deploy/kustomize/overlays/production/kustomization.yaml', 'resources:\n  - ../../base\n')
      await writeText(project.hidden_repo_path, 'infra/terraform/main.tf', 'resource "cue_runtime_app" "todo_app" {}\n')
    },
    test: () => writeJson(path.join(project.hidden_repo_path, 'tests/todo-app.test.json'), { status: 'passed', checks: ['schema', 'policy', 'workflow'] }),
    deployment: () => writeJson(path.join(project.hidden_repo_path, 'releases/sandbox.json'), { release: 'sandbox-v1', environment: 'sandbox', status: 'deployed' }),
    operation: () => writeJson(path.join(project.hidden_repo_path, 'operations/dashboard.json'), { health: 'ok', latest_release: 'sandbox-v1', ci_status: 'passed', deployment_environment: 'sandbox', open_blockers: [], runtime_metrics: { todo_count: 3, completed_count: 1, error_rate: 0 } }),
  }
  await writers[kind]?.()
}

async function writeText(root, relativePath, content) {
  const target = path.join(root, relativePath)
  await mkdir(path.dirname(target), { recursive: true })
  await writeFile(target, content, 'utf8')
}

async function writeJson(target, value) {
  await mkdir(path.dirname(target), { recursive: true })
  await writeFile(target, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

function fixtureContext(state, workitemId) {
  const project = projectForWorkItem(state, workitemId)
  const workitem = project.workitems.find((candidate) => candidate.id === workitemId)
  const artifacts = project.artifacts.filter((artifact) => artifact.workitem_id === workitemId)
  const nextArtifactKind = nextWorkflowArtifactKind(workitem)
  return {
    type: artifacts.length > 0 ? 'artifact' : workitem.blockers.length > 0 ? 'blockers' : 'workflow_plan',
    project_id: project.id,
    workitem,
    workflow_plan: workitem.workflow_plan,
    artifacts,
    blockers: workitem.blockers,
    qc_status: workitem.qc_status,
    qc_checks: workitem.qc_checks,
    next_action: workitem.next_action,
    next_artifact_kind: nextArtifactKind,
  }
}

function nextWorkflowArtifactKind(workitem) {
  for (const step of workitem.workflow_plan) {
    if (step.state === 'done') continue
    const dependenciesDone = (step.depends_on ?? []).every((dependency) =>
      workitem.workflow_plan.some((candidate) => candidate.id === dependency && candidate.state === 'done'),
    )
    return dependenciesDone ? step.id : null
  }
  return null
}

function workflowProgress(workitem) {
  const steps = workitem.workflow_plan.length
  if (steps === 0) return workitem.progress
  const done = workitem.workflow_plan.filter((step) => step.state === 'done').length
  return Math.round((done / steps) * 100)
}

async function readJsonBody(request) {
  const chunks = []
  for await (const chunk of request) chunks.push(chunk)
  const raw = Buffer.concat(chunks).toString('utf8')
  return raw ? JSON.parse(raw) : {}
}

function projectForSession(state, sessionId) {
  for (const project of Object.values(state.projects)) {
    if (project.sessions.some((session) => session.id === sessionId)) return project
  }
  throw new Error(`Unknown session: ${sessionId}`)
}

function projectForWorkItem(state, workitemId) {
  for (const project of Object.values(state.projects)) {
    if (project.workitems.some((workitem) => workitem.id === workitemId)) return project
  }
  throw new Error(`Unknown WorkItem: ${workitemId}`)
}

function message(id, speaker, body, action) {
  return { id, speaker, body, action }
}

function fixtureState(workspace = tmpdir()) {
  return {
    workspace,
    projects: {
      'team-request-tracker': {
        id: 'team-request-tracker',
        name: 'Team Request Tracker',
        owner: 'Operations',
        status: 'needs-review',
        next_action: 'Review PRD',
        summary: 'WorkItem accepted; PRD is waiting for owner review.',
        active_session_id: 'session-request-tracker',
        sessions: [
          {
            id: 'session-request-tracker-archive',
            project_id: 'team-request-tracker',
            title: 'Archived import cleanup',
            messages: [
              message('m1', 'owner', '舊匯入流程可以先封存，避免影響 request tracker。'),
              message('m2', 'cue', '我會保留這個 session 作為歷史脈絡，不列入最近三個。'),
            ],
          },
          {
            id: 'session-request-tracker-owner-handoff',
            project_id: 'team-request-tracker',
            title: 'Owner handoff',
            messages: [
              message('m1', 'owner', 'Operations lead 下週交接，請把 owner review 的待辦留下。'),
              message('m2', 'cue', '已標記 owner handoff，PRD review gate 仍由 Operations 確認。'),
            ],
          },
          {
            id: 'session-request-tracker-policy',
            project_id: 'team-request-tracker',
            title: 'Policy review follow-up',
            messages: [
              message('m1', 'owner', '請確認 request tracker 會不會碰到 PII 欄位。'),
              message('m2', 'cue', '已把 PII 欄位檢查列入 PRD review 前的 policy evidence。'),
            ],
          },
          {
            id: 'session-request-tracker',
            project_id: 'team-request-tracker',
            title: 'Request tracker intake',
            messages: [
              message('m1', 'owner', '我們需要一個內部請求追蹤流程，讓同事送單，Operations 可以分派和追狀態。'),
              message('m2', 'cue', '我先把這個 prompt 收成 WorkItem，判斷路由是 prompt-to-PRD。WorkItem 已建立；接著整理 PRD，請確認目標和使用者。', 'Open PRD'),
              message('m3', 'cue', '目前還需要確認 production owner 和資料保存期限。PRD 通過後 TD 才能開始。'),
            ],
          },
        ],
        stages: [
          { id: 'workitem', label: 'WorkItem', state: 'done', detail: 'Prompt accepted as prompt-to-PRD.' },
          projectStage('prd', 'PRD', 'in-progress', 'Goal, users, fields, and success metric drafted.'),
          projectStage('td', 'TD', 'not-started', 'Starts after PRD approval.'),
          projectStage('website', 'Website', 'not-started', 'Runtime artifact is locked by TD.'),
        ],
        workitems: [
          {
            id: 'request-tracker-prd',
            project_id: 'team-request-tracker',
            title: 'Create request tracker PRD',
            route: 'prompt-to-PRD',
            target: 'PRD',
            state: 'accepted',
            progress: 100,
            next_action: 'Review PRD',
            blockers: [],
            workflow_plan: websiteWorkflowPlan('in-progress'),
            qc_status: 'pass',
            qc_checks: [
              { id: 'intent', label: 'Intent is project work', status: 'pass', summary: 'Prompt maps to a governed PRD artifact route.' },
            ],
          },
          {
            id: 'request-tracker-retention',
            project_id: 'team-request-tracker',
            title: 'Confirm data retention',
            route: 'prompt-to-PRD',
            target: 'PRD',
            state: 'collecting',
            progress: 62,
            next_action: 'Answer retention question',
            blockers: ['Confirm production owner'],
            workflow_plan: [],
            qc_status: 'needs_input',
            qc_checks: [
              { id: 'owner', label: 'Production owner', status: 'needs_input', summary: 'Owner must confirm production ownership.' },
            ],
          },
        ],
        artifacts: [
          {
            id: 'request-tracker-prd-v1',
            workitem_id: 'request-tracker-prd',
            label: 'Project PRD',
            kind: 'prd',
            status: 'Needs review',
            qc_status: 'pending',
            qc_checks: [{ id: 'prd_review', label: 'PRD owner review', status: 'pending', summary: 'Owner review must complete before TD starts.' }],
          },
        ],
      },
      'weekly-ops-report': {
        id: 'weekly-ops-report',
        name: 'Weekly Ops Report',
        owner: 'Revenue Ops',
        status: 'in-progress',
        next_action: 'Complete WorkItem',
        summary: 'WorkItem is collecting the minimum details before PRD generation.',
        active_session_id: 'session-weekly-ops-report',
        sessions: [
          {
            id: 'session-weekly-ops-source',
            project_id: 'weekly-ops-report',
            title: 'Source mapping',
            messages: [
              message('m1', 'owner', 'Pipeline 來源先用 CRM export。'),
              message('m2', 'cue', '資料來源已記錄，但還缺收件人和寄送時間。'),
            ],
          },
          {
            id: 'session-weekly-ops-recipient',
            project_id: 'weekly-ops-report',
            title: 'Recipient review',
            messages: [
              message('m1', 'owner', '收件人可能是主管群組。'),
              message('m2', 'cue', 'WorkItem 還需要正式的收件群組名稱。'),
            ],
          },
          {
            id: 'session-weekly-ops-report',
            project_id: 'weekly-ops-report',
            title: 'Weekly report intake',
            messages: [
              message('m1', 'owner', '每週幫我整理 pipeline 和逾期 account，寄給主管。'),
              message('m2', 'cue', '我先建立 WorkItem，但還缺收件人、資料來源和寄送時間。確認後才會產生 PRD artifact。'),
            ],
          },
        ],
        stages: [
          { id: 'workitem', label: 'WorkItem', state: 'in-progress', detail: 'Missing owner, recipients, and data source.' },
          projectStage('prd', 'PRD', 'not-started', 'Starts after WorkItem acceptance.'),
          projectStage('td', 'TD', 'not-started', 'Waiting on PRD approval.'),
          projectStage('website', 'Website', 'not-started', 'Report artifact is locked by TD.'),
        ],
        workitems: [
          {
            id: 'weekly-report-intake',
            project_id: 'weekly-ops-report',
            title: 'Collect report basics',
            route: 'prompt-to-WorkItem',
            target: 'WorkItem',
            state: 'collecting',
            progress: 48,
            next_action: 'Add recipients and data source',
            blockers: ['Add recipients', 'Select data source'],
            workflow_plan: websiteWorkflowPlan('blocked'),
            qc_status: 'needs_input',
            qc_checks: [
              { id: 'recipients', label: 'Recipients', status: 'needs_input', summary: 'Report recipients are missing.' },
            ],
          },
        ],
        artifacts: [],
      },
    },
  }
}

function assertContains(text, expected) {
  if (!text.includes(expected)) {
    throw new Error(`Expected page to contain: ${expected}\nActual: ${text}`)
  }
}

function assertNotContains(text, unexpected) {
  if (text.includes(unexpected)) {
    throw new Error(`Expected page not to contain: ${unexpected}`)
  }
}

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`)
  }
}

function assertGreater(actual, expected, label) {
  if (actual <= expected) {
    throw new Error(`${label}: expected ${JSON.stringify(actual)} to be greater than ${JSON.stringify(expected)}`)
  }
}

async function findFreePort() {
  await mkdir(tmpdir(), { recursive: true })
  return new Promise((resolve, reject) => {
    const server = net.createServer()
    server.listen(0, '127.0.0.1', () => {
      const address = server.address()
      server.close(() => {
        if (address && typeof address === 'object') {
          resolve(address.port)
        } else {
          reject(new Error('Could not allocate a free port'))
        }
      })
    })
    server.on('error', reject)
  })
}

if (process.argv.includes('--fixture-api')) {
  const port = Number(process.env.CUE_ARTIFACT_STUDIO_FIXTURE_API_PORT ?? 43219)
  const fixture = await startFixtureApi(port)
  console.log(`Fixture API: ${fixture.url}`)
  setInterval(() => {}, 1 << 30)
} else {
  const keepAlive = setInterval(() => {}, 1 << 30)
  try {
    await main()
    process.exit(0)
  } catch (error) {
    console.error(error)
    process.exit(1)
  } finally {
    clearInterval(keepAlive)
  }
}
