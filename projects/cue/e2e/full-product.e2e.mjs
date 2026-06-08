import { randomUUID } from 'node:crypto'
import { mkdtemp, rm, stat, writeFile, mkdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import path from 'node:path'

const forbiddenOwnerTerms = ['GitLab', 'GCP', 'Terraform', 'Kustomize', 'CI', 'branch', 'commit']

const stages = [
  { id: 'prd', label: 'PRD', artifact: 'prd.md', agent_role: 'pm', agent_label: 'PM agent' },
  { id: 'td', label: 'TD', artifact: 'td.md', agent_role: 'architect', agent_label: 'Architect agent' },
  { id: 'codebase', label: 'Codebase', artifact: 'app-spec.json', agent_role: 'dev', agent_label: 'Dev agent' },
  { id: 'test', label: 'Test', artifact: 'tests/todo-app.test.json', agent_role: 'qa_policy', agent_label: 'QA/policy agent' },
  { id: 'deployment', label: 'Deployment', artifact: 'releases/sandbox.json', agent_role: 'release', agent_label: 'Release agent' },
  { id: 'operation', label: 'Operation', artifact: 'operations/dashboard.json', agent_role: 'data', agent_label: 'Data agent' },
]

async function main() {
  const workspace = await mkdtemp(path.join(tmpdir(), 'cue-full-product-e2e-'))
  try {
    const project = await createProject(workspace)
    assertOwnerCopy(project.owner_visible_summary)
    await assertPath(project.hidden_repo.path)

    const workitem = createWorkItem(project, 'Create a todo app for the Operations team with task title, owner, status, and due date.')
    assertEqual(workitem.id, 'todo-app-workitem', 'workitem id')
    assertEqual(project.artifacts.length, 0, 'no downstream artifacts before WorkItem')

    await createPrd(project, workitem)
    await createTd(project, workitem)
    await createCodebase(project, workitem)
    await runLocalCi(project, workitem)
    await deploySandbox(project, workitem)
    await updateOperationsDashboard(project, workitem)

    await assertRepoFiles(project.hidden_repo.path, [
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
    ])
    assertStageOrder(workitem)
    assertNodeAgents(workitem)
    assertOperationsDashboard(project)
    assertOwnerCopy(project.owner_visible_summary)
    assertAdminEvidence(project)

    console.log(`Full product e2e passed: ${project.id}`)
    console.log(`Local hidden repo: ${project.hidden_repo.path}`)
  } finally {
    if (!process.env.CUE_KEEP_E2E_WORKSPACE) {
      await rm(workspace, { recursive: true, force: true })
    }
  }
}

async function createProject(workspace) {
  const repoPath = path.join(workspace, 'hidden-repos', 'todo-app-project')
  await mkdir(repoPath, { recursive: true })
  const project = {
    id: 'todo-app-project',
    name: 'Todo App Project',
    owner_visible_summary: 'Todo app project is ready for guided delivery. Sandbox and operations status will appear here.',
    hidden_repo: {
      path: repoPath,
      kind: 'local-hidden-repo',
    },
    workitems: [],
    artifacts: [],
    admin_evidence: {
      hidden_repo_path: repoPath,
      local_gitlab_simulation: true,
      local_gcp_simulation: true,
      events: [{ id: randomUUID(), type: 'project_created' }],
    },
  }

  await writeJson(path.join(repoPath, 'cue-project.json'), {
    id: project.id,
    name: project.name,
    owner_visible: false,
    registry_state: 'provisioned',
  })
  return project
}

function createWorkItem(project, prompt) {
  const workitem = {
    id: 'todo-app-workitem',
    goal: 'Create a governed todo app',
    prompt,
    status: 'in_progress',
    current_stage_id: 'prd',
    stages: stages.map((stage, index) => ({
      id: stage.id,
      label: stage.label,
      status: index === 0 ? 'ready' : 'not_started',
      artifact: stage.artifact,
      agent_role: stage.agent_role,
      agent_label: stage.agent_label,
    })),
    gates: [],
  }
  project.workitems.push(workitem)
  project.admin_evidence.events.push({ id: randomUUID(), type: 'workitem_created', workitem_id: workitem.id })
  return workitem
}

async function createPrd(project, workitem) {
  const content = [
    '# Todo App PRD',
    '',
    'Goal: create a governed todo app for Operations.',
    'Users: requester, operations owner.',
    'Fields: title, owner, status, due_date.',
    'Success: owner can create and complete todo items in sandbox.',
  ].join('\n')
  await writeText(project, 'prd.md', content)
  completeStage(project, workitem, 'prd', 'prd.md')
}

async function createTd(project, workitem) {
  const content = [
    '# Todo App TD',
    '',
    'Data model: TodoItem(id, title, owner, status, due_date).',
    'API: list todos, create todo, update status.',
    'UI: table, create form, status filter, operations summary.',
    'Deployment: sandbox first; production requires approval.',
  ].join('\n')
  await writeText(project, 'td.md', content)
  completeStage(project, workitem, 'td', 'td.md')
}

async function createCodebase(project, workitem) {
  await writeJson(filePath(project, 'app-spec.json'), {
    schema_version: 'cue.app-spec.v0',
    app_id: 'todo-app',
    display_name: 'Todo App',
    primitives: ['table', 'form', 'dashboard'],
    fields: [
      { id: 'title', type: 'string', required: true },
      { id: 'owner', type: 'string', required: true },
      { id: 'status', type: 'enum', values: ['todo', 'doing', 'done'] },
      { id: 'due_date', type: 'date' },
    ],
  })
  await writeText(project, 'src/todo-app.ts', [
    'export type TodoStatus = "todo" | "doing" | "done"',
    'export type TodoItem = { id: string; title: string; owner: string; status: TodoStatus; due_date?: string }',
    'export function completeTodo(item: TodoItem): TodoItem {',
    '  return { ...item, status: "done" }',
    '}',
  ].join('\n'))
  await writeText(project, '.gitlab-ci.yml', [
    'stages:',
    '  - test',
    '  - deploy',
    'test:',
    '  script:',
    '    - cue test todo-app',
    'deploy_sandbox:',
    '  script:',
    '    - cue deploy sandbox',
  ].join('\n'))
  await writeText(project, 'deploy/kustomize/base/deployment.yaml', [
    'apiVersion: apps/v1',
    'kind: Deployment',
    'metadata:',
    '  name: todo-app',
    'spec:',
    '  replicas: 1',
  ].join('\n'))
  await writeText(project, 'deploy/kustomize/overlays/sandbox/kustomization.yaml', [
    'resources:',
    '  - ../../base',
    'nameSuffix: -sandbox',
  ].join('\n'))
  await writeText(project, 'deploy/kustomize/overlays/production/kustomization.yaml', [
    'resources:',
    '  - ../../base',
    'nameSuffix: -production',
  ].join('\n'))
  await writeText(project, 'infra/terraform/main.tf', [
    'resource "cue_runtime_app" "todo_app" {',
    '  name        = "todo-app"',
    '  environment = "sandbox"',
    '}',
  ].join('\n'))
  completeStage(project, workitem, 'codebase', 'app-spec.json')
}

async function runLocalCi(project, workitem) {
  const result = {
    status: 'passed',
    checks: [
      { id: 'schema', status: 'passed' },
      { id: 'policy', status: 'passed' },
      { id: 'workflow', status: 'passed' },
    ],
  }
  await writeJson(filePath(project, 'tests/todo-app.test.json'), result)
  project.admin_evidence.latest_pipeline = {
    id: 'local-pipeline-1',
    status: 'passed',
    source: 'local-ci-simulation',
  }
  completeStage(project, workitem, 'test', 'tests/todo-app.test.json')
}

async function deploySandbox(project, workitem) {
  const release = {
    release: 'sandbox-v1',
    environment: 'sandbox',
    status: 'deployed',
    source_ref: 'local-hidden-repo/todo-app-project',
    pipeline: project.admin_evidence.latest_pipeline,
  }
  await writeJson(filePath(project, 'releases/sandbox.json'), release)
  project.admin_evidence.deployment = release
  completeStage(project, workitem, 'deployment', 'releases/sandbox.json')
}

async function updateOperationsDashboard(project, workitem) {
  const dashboard = {
    health: 'ok',
    latest_release: 'sandbox-v1',
    ci_status: 'passed',
    deployment_environment: 'sandbox',
    open_blockers: [],
    runtime_metrics: {
      todo_count: 3,
      completed_count: 1,
      error_rate: 0,
    },
  }
  await writeJson(filePath(project, 'operations/dashboard.json'), dashboard)
  project.operations_dashboard = dashboard
  project.owner_visible_summary = 'Todo app sandbox is ready. Health is ok, tests passed, and operations dashboard has no blockers.'
  completeStage(project, workitem, 'operation', 'operations/dashboard.json')
  workitem.status = 'done'
}

function completeStage(project, workitem, stageId, artifact) {
  const stage = workitem.stages.find((candidate) => candidate.id === stageId)
  if (!stage) throw new Error(`Unknown stage ${stageId}`)
  stage.status = 'done'
  project.artifacts.push({
    id: `${stageId}-artifact`,
    type: stageId,
    path: artifact,
    status: 'done',
    agent_role: stage.agent_role,
    agent_label: stage.agent_label,
  })

  const next = workitem.stages.find((candidate) => candidate.status === 'not_started')
  if (next) {
    next.status = 'ready'
    workitem.current_stage_id = next.id
  } else {
    workitem.current_stage_id = stageId
  }
}

async function writeText(project, relativePath, content) {
  const target = filePath(project, relativePath)
  await mkdir(path.dirname(target), { recursive: true })
  await writeFile(target, `${content}\n`, 'utf8')
}

async function writeJson(target, value) {
  await mkdir(path.dirname(target), { recursive: true })
  await writeFile(target, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

function filePath(project, relativePath) {
  return path.join(project.hidden_repo.path, relativePath)
}

async function assertPath(target) {
  await stat(target)
}

async function assertRepoFiles(repoPath, files) {
  for (const file of files) {
    await assertPath(path.join(repoPath, file))
  }
}

function assertStageOrder(workitem) {
  assertEqual(workitem.status, 'done', 'workitem status')
  for (const stage of stages) {
    const actual = workitem.stages.find((candidate) => candidate.id === stage.id)
    assertEqual(actual?.status, 'done', `stage ${stage.id}`)
  }
}

function assertNodeAgents(workitem) {
  const roles = new Set(workitem.stages.map((stage) => stage.agent_role))
  assertEqual(roles.size, stages.length, 'one distinct agent per workflow node')
  for (const stage of stages) {
    const actual = workitem.stages.find((candidate) => candidate.id === stage.id)
    assertEqual(actual?.agent_role, stage.agent_role, `stage ${stage.id} agent role`)
    assertEqual(actual?.agent_label, stage.agent_label, `stage ${stage.id} agent label`)
  }
}

function assertOperationsDashboard(project) {
  const dashboard = project.operations_dashboard
  assertEqual(dashboard.health, 'ok', 'dashboard health')
  assertEqual(dashboard.latest_release, 'sandbox-v1', 'dashboard latest release')
  assertEqual(dashboard.ci_status, 'passed', 'dashboard ci status')
  assertEqual(dashboard.deployment_environment, 'sandbox', 'dashboard deployment environment')
  assertEqual(dashboard.open_blockers.length, 0, 'dashboard open blockers')
  assertEqual(dashboard.runtime_metrics.error_rate, 0, 'dashboard error rate')
}

function assertOwnerCopy(text) {
  for (const term of forbiddenOwnerTerms) {
    if (text.includes(term)) {
      throw new Error(`Owner-visible copy leaked infrastructure term: ${term}`)
    }
  }
}

function assertAdminEvidence(project) {
  assertContains(project.admin_evidence.hidden_repo_path, 'todo-app-project')
  assertEqual(project.admin_evidence.local_gitlab_simulation, true, 'admin GitLab simulation evidence')
  assertEqual(project.admin_evidence.local_gcp_simulation, true, 'admin GCP simulation evidence')
  assertEqual(project.admin_evidence.latest_pipeline.status, 'passed', 'admin pipeline evidence')
  assertEqual(project.admin_evidence.deployment.status, 'deployed', 'admin deployment evidence')
}

function assertContains(value, expected) {
  if (!String(value).includes(expected)) {
    throw new Error(`Expected ${JSON.stringify(value)} to contain ${JSON.stringify(expected)}`)
  }
}

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`)
  }
}

await main()
