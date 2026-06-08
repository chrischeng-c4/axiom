import type {
  AdminReviewTicketKind,
  AdminReviewTicketStatus,
  ArtifactStatus,
  Persona,
  RouteKey,
  StoryState,
  StudioTab,
} from './types'

export type Locale = 'zh-TW' | 'en-US'

export type GovernanceCopy = {
  labels: {
    spec: string
    ownership: string
    risk: string
    runtimeData: string
    production: string
    audit: string
  }
  details: {
    governedEntity: (count: number) => string
    missingOwner: string
    riskBlocked: string
    sandboxEligible: (tier: string) => string
    runtimeData: string
    approvalGate: (count: number) => string
    noApprover: string
    auditRetention: (days: number) => string
    auditDisabled: string
  }
}

export type CueCopy = {
  shell: {
    navAria: string
    ownerNavAria: string
    adminNavAria: string
    brandSubtitle: string
    adminSubtitle: string
    owner: string
    environment: string
    appStateAria: string
    reviewBadge: (count: number) => string
    personaAria: string
    personas: Record<Persona, { label: string; description: string }>
    workspace: string
  }
  workspaces: {
    currentAria: string
    labels: {
      allowedScope: string
      adminGate: string
    }
    reviewSummary: (count: number) => string
    user: {
      eyebrow: string
      title: string
      detail: string
      scope: string[]
      gate: string
    }
    admin: {
      eyebrow: string
      title: string
      detail: string
      scope: string[]
      gate: string
    }
  }
  ownerGuide: {
    ariaLabel: string
    eyebrow: string
    title: string
    detail: string
    nextActionLabel: string
    adminGateLabel: string
    adminGate: string
    nextByRoute: Record<RouteKey, string>
  }
  artifacts: {
    eyebrow: string
    title: string
    ownerOnlyTitle: string
    platformOnlyTitle: string
    empty: string
    status: Record<ArtifactStatus, string>
  }
  routes: Record<RouteKey, { label: string; description: string }>
  storyStates: Record<StoryState, string>
  storyStepper: {
    ariaLabel: string
  }
  promptBuilder: {
    eyebrow: string
    title: string
    conversationLabel: string
    generateDraft: string
    loadExample: string
    clarificationEyebrow: string
    readyForStudio: string
    answersPending: (count: number) => string
    governanceRequired: string
    draftInReview: string
    agentTeamAria: string
    clarifications: Record<string, { label: string; value: string }>
    agentTeam: Record<string, { label: string; value: string }>
  }
  studio: {
    previewEyebrow: string
    previewTitle: string
    editorEyebrow: string
    tabsAria: string
    tabs: Record<StudioTab, string>
    riskEyebrow: string
    sandboxEligible: string
    blocked: string
    requestSandbox: string
    requestProduction: string
    workflowTransitionBy: string
    permissionsCount: (count: number) => string
    ownerGuidance: string
    platformGuidance: string
  }
  sandbox: {
    eyebrow: string
    sampleData: string
    resetAria: string
    runtimeTenantEyebrow: string
    ownerReviewEyebrow: string
    ownerReviewTitle: string
    ownerReviewRows: Array<{ label: string; value: string }>
    requestProduction: string
    ownerGuidance: string
    platformGuidance: string
    rows: {
      tenant: string
      cluster: string
      database: string
      isolation: string
      retention: string
      migration: string
      families: string
      dayArchive: (days: number) => string
      runtimeFamilies: (count: number) => string
    }
  }
  registry: {
    eyebrow: string
    title: string
    searchAria: string
    tableAria: string
    headers: string[]
    filters: string[]
    production: string
    healthy: string
    open: string
    ownerView: string
    platformView: string
  }
  admin: {
    approvalEyebrow: string
    productionRequest: string
    approveRelease: string
    policyEyebrow: string
    findings: string
    platformEyebrow: string
    hiddenInfrastructure: string
    reviewEyebrow: string
    reviewQueue: string
    approveTicket: string
    ticketApproved: string
    reviewGuidance: string
    ticketKind: Record<AdminReviewTicketKind, string>
    ticketStatus: Record<AdminReviewTicketStatus, string>
    reviewRows: {
      request: string
      workspace: string
      app: string
      requestedBy: string
      kind: string
      resource: string
      risk: string
      environment: string
      data: string
      agentOutput: string
      rationale: string
    }
    releaseEvents: Array<{ label: string; value: string }>
    policyFindings: Array<{ label: string; value: string; state: 'pass' | 'review' | 'block' }>
    hiddenRows: {
      gitlabProject: string
      userVisible: string
      runtimeCluster: string
      runtimeDatabase: string
      quota: string
      emergencyContact: string
      sandboxApps: (count: number) => string
    }
    ownerReadonly: string
    platformMutable: string
  }
  previewTable: {
    ariaLabel: string
    headers: string[]
  }
  mock: {
    defaultPrompt: string
  }
  governance: GovernanceCopy
}

export const defaultLocale: Locale = 'zh-TW'

export function resolveLocale(value?: string | null): Locale {
  if (value === 'en-US' || value === 'en') return 'en-US'
  if (value === 'zh-TW' || value === 'zh-Hant' || value === 'zh') return 'zh-TW'
  return defaultLocale
}

export function configuredLocale(): Locale {
  const params = new URLSearchParams(window.location.search)
  return resolveLocale(params.get('locale') ?? window.localStorage.getItem('cue.locale'))
}

export function getCopy(locale: Locale = defaultLocale): CueCopy {
  return dictionaries[locale]
}

const dictionaries: Record<Locale, CueCopy> = {
  'zh-TW': {
    shell: {
      navAria: 'Cue 導覽',
      ownerNavAria: 'Cue 使用者工作區導覽',
      adminNavAria: 'Cue Admin 導覽',
      brandSubtitle: '使用者工作區',
      adminSubtitle: '平台後台',
      owner: 'Owner',
      environment: 'Environment',
      appStateAria: 'App 狀態',
      reviewBadge: (count) => `${count} review`,
      personaAria: '切換檢視角色',
      workspace: 'Workspace',
      personas: {
        owner: { label: 'App Owner', description: '管理設定與核准' },
        platform: { label: 'Platform', description: '維運 infra 與 release' },
      },
    },
    workspaces: {
      currentAria: '目前工作區邊界',
      labels: {
        allowedScope: 'Allowed scope',
        adminGate: 'Admin gate',
      },
      reviewSummary: (count) => (count === 0 ? '沒有待審 Admin ticket' : `${count} 張 Admin ticket 待審`),
      user: {
        eyebrow: 'User workspace',
        title: 'Operations Workspace',
        detail: '用戶只描述需求；受控 agent team 在這裡整理需求、產生 App Spec、做實作與測試輸出。',
        scope: ['對話需求整理', 'App Spec 與實作', '測試結果與 release package', 'Admin review ticket'],
        gate: '部署測試、SaaS API、昂貴資源、PII access 與高風險 capability 都只能送審，不能直接 grant。',
      },
      admin: {
        eyebrow: 'Admin workspace',
        title: 'Cue Platform Admin',
        detail: 'Platform admin 審核 agent 完成後產生的 ticket、runtime 權限、SaaS API、昂貴資源與 release gate。',
        scope: ['deployment ticket', 'SaaS API grant', 'resource budget', 'policy exception', 'production release'],
        gate: '核准後才會把 tool、data scope、runtime permission 或資源 quota 下放給該 workspace 的 agents。',
      },
    },
    ownerGuide: {
      ariaLabel: '一般使用者工作區說明',
      eyebrow: 'User workspace',
      title: '你只要說明需求，Cue 會整理成可審核的 App',
      detail:
        '一般使用者不需要理解 GitLab、資料庫、CI 或雲端資源。這些會被 agent 整理成規格、試用結果與需要 Admin 核准的 ticket。',
      nextActionLabel: '目前要做的事',
      adminGateLabel: '需要審核才會執行',
      adminGate: 'Production、SaaS API、昂貴資源與高風險權限會送 Admin ticket。',
      nextByRoute: {
        new: '描述你想解決的工作流程，agent 會補問 owner、資料、角色與成功條件。',
        studio: '確認 agent 整理出的方案、畫面預覽、權限摘要與下一步。',
        sandbox: '用範例資料試跑流程，確認可以交給同事使用。',
        registry: '查看你擁有的 Apps、目前狀態與下一個 action。',
        admin: '平台維護者審核資源、API、部署與 release ticket。',
      },
    },
    artifacts: {
      eyebrow: 'Artifacts',
      title: '權威狀態',
      ownerOnlyTitle: 'Owner artifacts',
      platformOnlyTitle: 'Platform artifacts',
      empty: '沒有可顯示的 artifact',
      status: {
        draft: 'Draft',
        ready: 'Ready',
        review: 'Review',
        blocked: 'Blocked',
      },
    },
    routes: {
      new: { label: '說需求', description: '新增或整理 App' },
      studio: { label: '確認方案', description: '規格與預覽' },
      sandbox: { label: '試用測試', description: 'Sandbox 驗收' },
      registry: { label: '我的 Apps', description: '狀態與管理' },
      admin: { label: '平台審核', description: 'Admin tickets' },
    },
    storyStates: {
      PromptDraft: '對話草稿',
      Clarifying: '待釐清',
      SpecDraft: 'App Spec 草稿',
      StudioPreview: '方案確認',
      SandboxReady: 'Sandbox ready',
      ProductionRequested: 'Production 申請中',
      ProductionReady: 'Production ready',
    },
    storyStepper: {
      ariaLabel: 'App lifecycle',
    },
    promptBuilder: {
      eyebrow: '對話入口',
      title: '先說你需要什麼 App',
      conversationLabel: '需求描述',
      generateDraft: '整理需求與草稿',
      loadExample: '載入範例',
      clarificationEyebrow: '待釐清',
      readyForStudio: '可以確認方案',
      answersPending: (count) => `${count} 項待回答`,
      governanceRequired: '需要補治理資訊',
      draftInReview: '草稿保留在 spec review',
      agentTeamAria: '指派的 agent team',
      clarifications: {
        owner: { label: 'Owner', value: 'Operations / ops-lead@example.com' },
        fields: { label: 'Fields', value: 'title、status、owner、due date' },
        roles: { label: 'Roles', value: 'Requester 與 request manager' },
        runtime: { label: 'Runtime data', value: 'shared cluster，app database 隔離' },
        approval: { label: 'Approval', value: '進 production 前需要 App owner 核准' },
      },
      agentTeam: {
        pm: { label: 'PM', value: '目標、owner、scope' },
        designer: { label: 'Designer', value: 'UI primitives 與 layout' },
        dev: { label: 'Dev', value: 'hidden repo artifacts' },
        data: { label: 'Data', value: 'tenant 與 data contracts' },
        qaPolicy: { label: 'QA / Policy', value: 'permission 與 release gates' },
      },
    },
    studio: {
      previewEyebrow: '方案預覽',
      previewTitle: '確認 App 預覽',
      editorEyebrow: '要確認的設定',
      tabsAria: 'Studio 區段',
      tabs: {
        fields: '欄位',
        workflow: '流程',
        permissions: '權限',
        notifications: '通知',
        dashboard: 'Dashboard',
      },
      riskEyebrow: '風險與 release',
      sandboxEligible: '可進 Sandbox',
      blocked: 'Blocked',
      requestSandbox: '申請 Sandbox',
      requestProduction: '申請 Production',
      workflowTransitionBy: '由',
      permissionsCount: (count) => `${count} permissions`,
      ownerGuidance: '看不懂的技術細節會交給 agent 和 Admin；你只需要確認需求、試用結果與是否送審。',
      platformGuidance: 'Platform 可檢查 hidden repo、runtime tenant、policy/test 與 release refs。',
    },
    sandbox: {
      eyebrow: 'Sandbox',
      sampleData: 'sample data',
      resetAria: '重設 Sandbox',
      runtimeTenantEyebrow: 'Runtime tenant',
      ownerReviewEyebrow: '試用驗收',
      ownerReviewTitle: '試用重點',
      ownerReviewRows: [
        { label: '流程', value: '新增 request、指派 owner、變更狀態、完成通知' },
        { label: '資料', value: '目前使用範例資料；不會影響正式環境' },
        { label: '權限', value: 'Requester 只能看自己的 request，manager 可以處理全部 request' },
        { label: '下一步', value: '確認沒問題後送 Production 申請，由 Admin 審核 release 與資源' },
      ],
      requestProduction: '申請 Production',
      ownerGuidance: 'Sandbox 是 owner 驗收區，資料可重設，不會影響 production。',
      platformGuidance: 'Platform 視角顯示 tenant、database isolation、migration 與 retention 設定。',
      rows: {
        tenant: 'Tenant',
        cluster: 'Cluster',
        database: 'Database',
        isolation: 'Isolation',
        retention: 'Retention',
        migration: 'Migration',
        families: 'Families',
        dayArchive: (days) => `${days} 天 archive`,
        runtimeFamilies: (count) => `${count} runtime families`,
      },
    },
    registry: {
      eyebrow: 'App Registry',
      title: '受治理的 Apps',
      searchAria: '搜尋 Apps',
      tableAria: 'App registry',
      headers: ['App', 'Owner', 'Namespace', 'Lifecycle', 'Risk', 'Health', 'Version', 'Action'],
      filters: ['Owner', 'Namespace', 'Lifecycle', 'Risk', 'Health'],
      production: 'production',
      healthy: 'healthy',
      open: '開啟',
      ownerView: 'Owner 看到 catalog、health、version 與下一步 action。',
      platformView: 'Platform 額外確認 hidden GitLab mapping 與 runtime release 狀態。',
    },
    admin: {
      approvalEyebrow: 'Approval queue',
      productionRequest: 'Production 申請',
      approveRelease: '核准 release',
      policyEyebrow: 'Policy',
      findings: 'Findings',
      platformEyebrow: 'Platform',
      hiddenInfrastructure: 'Hidden infrastructure',
      reviewEyebrow: 'Admin tickets',
      reviewQueue: '待審 Review tickets',
      approveTicket: '核准 ticket',
      ticketApproved: 'Ticket 已核准',
      reviewGuidance: '用戶只描述需求；agent 負責整理成 App Spec、實作、測試與 release package。部署測試、SaaS API、昂貴資源或高風險 capability 一律轉成 Admin review ticket。',
      ticketKind: {
        deployment_test: '部署測試',
        saas_api: 'SaaS API',
        resource_budget: '昂貴資源',
        capability: 'Capability',
      },
      ticketStatus: {
        pending: '待 Admin 審核',
        approved: '已核准',
      },
      reviewRows: {
        request: 'Request',
        workspace: 'Workspace',
        app: 'App',
        requestedBy: 'Requested by',
        kind: 'Kind',
        resource: 'Resource',
        risk: 'Risk',
        environment: 'Environment',
        data: 'Data scope',
        agentOutput: 'Agent output',
        rationale: 'Rationale',
      },
      releaseEvents: [
        { label: 'Draft spec', value: 'Ready' },
        { label: 'Policy check', value: 'Passed' },
        { label: 'Sandbox', value: 'Provisioned' },
        { label: 'Production', value: 'Waiting approval' },
      ],
      policyFindings: [
        { label: 'Risk tier', value: 'Tier 1 可進 sandbox', state: 'pass' },
        { label: 'Runtime data', value: 'shared cluster，app database 邊界', state: 'pass' },
        { label: 'Production', value: '需要 owner approval', state: 'review' },
        { label: 'External data', value: '尚未選 connector', state: 'pass' },
      ],
      hiddenRows: {
        gitlabProject: 'GitLab project',
        userVisible: 'User visible',
        runtimeCluster: 'Runtime cluster',
        runtimeDatabase: 'Runtime database',
        quota: 'Quota',
        emergencyContact: 'Emergency contact',
        sandboxApps: (count) => `${count} sandbox apps`,
      },
      ownerReadonly: 'Owner 在這裡只能看到 approval 結果；infra 細節由 Platform 維護。',
      platformMutable: 'Platform 可處理 policy exceptions、runtime binding、quota 與 emergency release。',
    },
    previewTable: {
      ariaLabel: 'Tracker preview',
      headers: ['ID', '標題', 'Owner', '狀態', '時間'],
    },
    mock: {
      defaultPrompt:
        '幫 Operations 做一個 request tracker，需要 owner 指派、due date、requester visibility、月度 review，以及 production approval。',
    },
    governance: {
      labels: {
        spec: 'Spec',
        ownership: 'Ownership',
        risk: 'Risk',
        runtimeData: 'Runtime data',
        production: 'Production',
        audit: 'Audit',
      },
      details: {
        governedEntity: (count) => `${count} 個 governed entity`,
        missingOwner: '缺少 owner',
        riskBlocked: 'Tier 4 暫不支援 MVP',
        sandboxEligible: (tier) => `${tier} 可進 sandbox`,
        runtimeData: '本地用 PostgreSQL，目標 shared AlloyDB cluster',
        approvalGate: (count) => `${count} 個 approval gate`,
        noApprover: '沒有 approver',
        auditRetention: (days) => `${days} 天 retention`,
        auditDisabled: 'Audit disabled',
      },
    },
  },
  'en-US': {
    shell: {
      navAria: 'Cue navigation',
      ownerNavAria: 'Cue user workspace navigation',
      adminNavAria: 'Cue Admin navigation',
      brandSubtitle: 'User workspace',
      adminSubtitle: 'Platform console',
      owner: 'Owner',
      environment: 'Environment',
      appStateAria: 'App state',
      reviewBadge: (count) => `${count} review`,
      personaAria: 'Switch view persona',
      workspace: 'Workspace',
      personas: {
        owner: { label: 'App Owner', description: 'Manage settings and approvals' },
        platform: { label: 'Platform', description: 'Operate infra and release' },
      },
    },
    workspaces: {
      currentAria: 'Current workspace boundary',
      labels: {
        allowedScope: 'Allowed scope',
        adminGate: 'Admin gate',
      },
      reviewSummary: (count) => (count === 0 ? 'No Admin tickets pending' : `${count} Admin tickets pending`),
      user: {
        eyebrow: 'User workspace',
        title: 'Operations Workspace',
        detail: 'Users describe needs; governed agent teams organize requirements, create App Specs, implement, and produce test outputs here.',
        scope: ['Requirement intake', 'App Spec and implementation', 'Test result and release package', 'Admin review ticket'],
        gate: 'Deployment tests, SaaS APIs, costly resources, PII access, and high-risk capabilities can only be requested, not granted directly.',
      },
      admin: {
        eyebrow: 'Admin workspace',
        title: 'Cue Platform Admin',
        detail: 'Platform admins review agent-produced tickets, runtime permissions, SaaS APIs, costly resources, and release gates.',
        scope: ['Deployment ticket', 'SaaS API grant', 'Resource budget', 'Policy exception', 'Production release'],
        gate: 'Approval grants tool, data scope, runtime permission, or resource quota to the agents assigned to that workspace.',
      },
    },
    ownerGuide: {
      ariaLabel: 'General user workspace guidance',
      eyebrow: 'User workspace',
      title: 'Describe the need; Cue turns it into a reviewable app',
      detail:
        'General users do not need to understand GitLab, databases, CI, or cloud resources. Agents turn that work into specs, sandbox results, and Admin approval tickets.',
      nextActionLabel: 'Current action',
      adminGateLabel: 'Requires review',
      adminGate: 'Production, SaaS APIs, costly resources, and high-risk permissions become Admin tickets.',
      nextByRoute: {
        new: 'Describe the workflow you need. Agents will ask about owner, data, roles, and success criteria.',
        studio: 'Review the agent proposal, preview, permission summary, and next step.',
        sandbox: 'Try the workflow with sample data before sharing it with teammates.',
        registry: 'Review your owned apps, status, and next action.',
        admin: 'Platform maintainers review resource, API, deploy, and release tickets.',
      },
    },
    artifacts: {
      eyebrow: 'Artifacts',
      title: 'Authoritative state',
      ownerOnlyTitle: 'Owner artifacts',
      platformOnlyTitle: 'Platform artifacts',
      empty: 'No visible artifact',
      status: {
        draft: 'Draft',
        ready: 'Ready',
        review: 'Review',
        blocked: 'Blocked',
      },
    },
    routes: {
      new: { label: 'Describe Need', description: 'Create or refine an app' },
      studio: { label: 'Confirm Plan', description: 'Spec and preview' },
      sandbox: { label: 'Try Sandbox', description: 'Owner acceptance' },
      registry: { label: 'My Apps', description: 'Status and management' },
      admin: { label: 'Platform Review', description: 'Admin tickets' },
    },
    storyStates: {
      PromptDraft: 'Prompt draft',
      Clarifying: 'Clarifying',
      SpecDraft: 'App Spec draft',
      StudioPreview: 'Plan review',
      SandboxReady: 'Sandbox ready',
      ProductionRequested: 'Production requested',
      ProductionReady: 'Production ready',
    },
    storyStepper: {
      ariaLabel: 'App lifecycle',
    },
    promptBuilder: {
      eyebrow: 'Conversation intake',
      title: 'Start with the app you need',
      conversationLabel: 'Need description',
      generateDraft: 'Organize need and draft',
      loadExample: 'Load example',
      clarificationEyebrow: 'Clarification',
      readyForStudio: 'Ready to confirm',
      answersPending: (count) => `${count} answer pending`,
      governanceRequired: 'Governance input required',
      draftInReview: 'Draft stays in spec review',
      agentTeamAria: 'Assigned agent team',
      clarifications: {
        owner: { label: 'Owner', value: 'Operations / ops-lead@example.com' },
        fields: { label: 'Fields', value: 'title, status, owner, due date' },
        roles: { label: 'Roles', value: 'Requester and request manager' },
        runtime: { label: 'Runtime data', value: 'Shared cluster, app database isolation' },
        approval: { label: 'Approval', value: 'App owner before production' },
      },
      agentTeam: {
        pm: { label: 'PM', value: 'Goal, owner, scope' },
        designer: { label: 'Designer', value: 'Approved UI primitives' },
        dev: { label: 'Dev', value: 'Hidden repo artifacts' },
        data: { label: 'Data', value: 'Tenant and data contracts' },
        qaPolicy: { label: 'QA / Policy', value: 'Permission and release gates' },
      },
    },
    studio: {
      previewEyebrow: 'Plan preview',
      previewTitle: 'Confirm app preview',
      editorEyebrow: 'Settings to confirm',
      tabsAria: 'Studio sections',
      tabs: {
        fields: 'Fields',
        workflow: 'Workflow',
        permissions: 'Permissions',
        notifications: 'Notifications',
        dashboard: 'Dashboard',
      },
      riskEyebrow: 'Risk and release',
      sandboxEligible: 'Sandbox eligible',
      blocked: 'Blocked',
      requestSandbox: 'Request sandbox',
      requestProduction: 'Request production',
      workflowTransitionBy: 'by',
      permissionsCount: (count) => `${count} permissions`,
      ownerGuidance: 'Technical details go to agents and Admin; owners confirm needs, sandbox results, and review requests.',
      platformGuidance: 'Platform can inspect hidden repo, runtime tenant, policy/test, and release refs.',
    },
    sandbox: {
      eyebrow: 'Sandbox',
      sampleData: 'sample data',
      resetAria: 'Reset sandbox',
      runtimeTenantEyebrow: 'Runtime tenant',
      ownerReviewEyebrow: 'Acceptance test',
      ownerReviewTitle: 'What to try',
      ownerReviewRows: [
        { label: 'Workflow', value: 'Create request, assign owner, change status, receive completion notice' },
        { label: 'Data', value: 'Uses sample data only; production is unaffected' },
        { label: 'Permissions', value: 'Requesters see their requests; managers can process all requests' },
        { label: 'Next step', value: 'Request Production when ready; Admin reviews release and resources' },
      ],
      requestProduction: 'Request production',
      ownerGuidance: 'Sandbox is the owner acceptance area; data is resettable and isolated from production.',
      platformGuidance: 'Platform view shows tenant, database isolation, migration, and retention settings.',
      rows: {
        tenant: 'Tenant',
        cluster: 'Cluster',
        database: 'Database',
        isolation: 'Isolation',
        retention: 'Retention',
        migration: 'Migration',
        families: 'Families',
        dayArchive: (days) => `${days} day archive`,
        runtimeFamilies: (count) => `${count} runtime families`,
      },
    },
    registry: {
      eyebrow: 'App Registry',
      title: 'Governed apps',
      searchAria: 'Search apps',
      tableAria: 'App registry',
      headers: ['App', 'Owner', 'Namespace', 'Lifecycle', 'Risk', 'Health', 'Version', 'Action'],
      filters: ['Owner', 'Namespace', 'Lifecycle', 'Risk', 'Health'],
      production: 'production',
      healthy: 'healthy',
      open: 'Open',
      ownerView: 'Owners see catalog, health, version, and next actions.',
      platformView: 'Platform also verifies hidden GitLab mapping and runtime release state.',
    },
    admin: {
      approvalEyebrow: 'Approval queue',
      productionRequest: 'Production request',
      approveRelease: 'Approve release',
      policyEyebrow: 'Policy',
      findings: 'Findings',
      platformEyebrow: 'Platform',
      hiddenInfrastructure: 'Hidden infrastructure',
      reviewEyebrow: 'Admin tickets',
      reviewQueue: 'Review tickets',
      approveTicket: 'Approve ticket',
      ticketApproved: 'Ticket approved',
      reviewGuidance: 'Users describe needs; agents turn them into App Specs, implementation, tests, and release packages. Deployment tests, SaaS APIs, costly resources, and high-risk capabilities always become Admin review tickets.',
      ticketKind: {
        deployment_test: 'Deployment test',
        saas_api: 'SaaS API',
        resource_budget: 'Costly resource',
        capability: 'Capability',
      },
      ticketStatus: {
        pending: 'Pending Admin review',
        approved: 'Approved',
      },
      reviewRows: {
        request: 'Request',
        workspace: 'Workspace',
        app: 'App',
        requestedBy: 'Requested by',
        kind: 'Kind',
        resource: 'Resource',
        risk: 'Risk',
        environment: 'Environment',
        data: 'Data scope',
        agentOutput: 'Agent output',
        rationale: 'Rationale',
      },
      releaseEvents: [
        { label: 'Draft spec', value: 'Ready' },
        { label: 'Policy check', value: 'Passed' },
        { label: 'Sandbox', value: 'Provisioned' },
        { label: 'Production', value: 'Waiting approval' },
      ],
      policyFindings: [
        { label: 'Risk tier', value: 'Tier 1 allowed for sandbox', state: 'pass' },
        { label: 'Runtime data', value: 'Shared cluster, app database boundary', state: 'pass' },
        { label: 'Production', value: 'Owner approval required', state: 'review' },
        { label: 'External data', value: 'No connector selected', state: 'pass' },
      ],
      hiddenRows: {
        gitlabProject: 'GitLab project',
        userVisible: 'User visible',
        runtimeCluster: 'Runtime cluster',
        runtimeDatabase: 'Runtime database',
        quota: 'Quota',
        emergencyContact: 'Emergency contact',
        sandboxApps: (count) => `${count} sandbox apps`,
      },
      ownerReadonly: 'Owners only see approval results here; Platform maintains infrastructure details.',
      platformMutable: 'Platform can handle policy exceptions, runtime binding, quota, and emergency release.',
    },
    previewTable: {
      ariaLabel: 'Tracker preview',
      headers: ['ID', 'Title', 'Owner', 'Status', 'Age'],
    },
    mock: {
      defaultPrompt:
        'Build a tracker for operations requests with owner assignment, due dates, requester visibility, monthly review, and production approval.',
    },
    governance: {
      labels: {
        spec: 'Spec',
        ownership: 'Ownership',
        risk: 'Risk',
        runtimeData: 'Runtime data',
        production: 'Production',
        audit: 'Audit',
      },
      details: {
        governedEntity: (count) => `${count} governed entity`,
        missingOwner: 'Missing owner',
        riskBlocked: 'Tier 4 blocked for MVP',
        sandboxEligible: (tier) => `${tier} sandbox eligible`,
        runtimeData: 'PostgreSQL locally, shared AlloyDB cluster target',
        approvalGate: (count) => `${count} approval gate`,
        noApprover: 'No approver',
        auditRetention: (days) => `${days} day retention`,
        auditDisabled: 'Audit disabled',
      },
    },
  },
}
