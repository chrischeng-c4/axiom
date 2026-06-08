interface CueWasmAppProps {
  initial_route: number;
  initial_spec_version: number;
  initial_approvals: number;
}

export function CueWasmApp({
  initial_route,
  initial_spec_version,
  initial_approvals,
}: CueWasmAppProps) {
  const [route, set_route] = useState(initial_route);
  const [spec_version, set_spec_version] = useState(initial_spec_version);
  const [approvals, set_approvals] = useState(initial_approvals);
  const [sandbox_ready, set_sandbox_ready] = useState(0);

  return (
    <div id="cue-wasm-root" className="cue-shell">
      <div id="cue-sidebar" className="cue-sidebar">
        <div id="cue-brand" className="cue-brand">
          <span id="cue-brand-mark">Cue</span>
          <span id="cue-brand-subtitle">Jet WASM control plane</span>
        </div>
        <button id="nav-new-app" className="nav-button" onClick={() => set_route(0)}>
          新增 App
        </button>
        <button id="nav-studio" className="nav-button" onClick={() => set_route(1)}>
          App Studio
        </button>
        <button id="nav-sandbox" className="nav-button" onClick={() => set_route(2)}>
          Sandbox
        </button>
        <button id="nav-registry" className="nav-button" onClick={() => set_route(3)}>
          Registry
        </button>
        <button id="nav-admin" className="nav-button" onClick={() => set_route(4)}>
          Admin
        </button>
      </div>

      <div id="cue-main" className="cue-main">
        <div id="cue-topbar" className="cue-topbar">
          <span id="cue-eyebrow">對話到 governed app</span>
          <span id="cue-app-title">Team Request Tracker</span>
          <span id="cue-spec-version">Spec v{spec_version}</span>
          <span id="cue-approval-count">Approvals {approvals}</span>
          {sandbox_ready == 1 && <span id="cue-sandbox-state">Sandbox ready</span>}
        </div>

        <div id="cue-story" className="cue-story">
          <span id="story-prompt">對話</span>
          <span id="story-spec">App Spec</span>
          <span id="story-repo">Hidden Repo</span>
          <span id="story-policy">Policy</span>
          <span id="story-sandbox">Sandbox</span>
          <span id="story-release">Release</span>
        </div>

        {route == 0 && (
          <div id="view-new-app" className="cue-view">
            <span id="new-app-title">建立追蹤 App</span>
            <span id="new-app-copy">描述團隊 workflow，Cue 會指派 PM、designer、dev、data、QA、release agents。</span>
            <span id="new-app-artifacts">對話會落成 App Spec、preview、tests、approvals 與 release refs。</span>
            <button id="generate-spec" onClick={() => set_spec_version(spec_version + 1)}>
              產生 App Spec
            </button>
            <button id="open-studio" onClick={() => set_route(1)}>
              開啟 Studio
            </button>
          </div>
        )}

        {route == 1 && (
          <div id="view-studio" className="cue-view">
            <span id="studio-title">App Studio：從 App Spec 產生 preview</span>
            <span id="studio-fields">Fields: title, status, owner, due date</span>
            <span id="studio-workflow">Workflow: New to Triaged to In Progress to Done</span>
            <button id="provision-sandbox" onClick={() => set_sandbox_ready(1)}>
              Provision Sandbox
            </button>
            <button id="open-sandbox" onClick={() => set_route(2)}>
              開啟 Sandbox
            </button>
          </div>
        )}

        {route == 2 && (
          <div id="view-sandbox" className="cue-view">
            <span id="sandbox-title">Sandbox runtime</span>
            <span id="sandbox-data">Runtime data 使用 shared cluster，並以 app database 隔離。</span>
            {sandbox_ready == 1 && <span id="sandbox-ready-copy">Release candidate 可以申請 approval。</span>}
            <button id="request-approval" onClick={() => set_approvals(approvals + 1)}>
              申請 Approval
            </button>
            <button id="open-admin" onClick={() => set_route(4)}>
              開啟 Admin
            </button>
          </div>
        )}

        {route == 3 && (
          <div id="view-registry" className="cue-view">
            <span id="registry-title">App Registry</span>
            <span id="registry-app">team-request-tracker 對應到 hidden GitLab project。</span>
            <span id="registry-release">目前 release tag 跟隨已核准的 App Spec refs。</span>
            <button id="registry-studio" onClick={() => set_route(1)}>
              Review Spec
            </button>
          </div>
        )}

        {route == 4 && (
          <div id="view-admin" className="cue-view">
            <span id="admin-title">Governance admin</span>
            <span id="admin-policy">這裡顯示 risk tier、owner、permission、connector 與 audit gates。</span>
            <button id="approve-production" onClick={() => set_approvals(approvals + 1)}>
              核准 Production
            </button>
            <button id="open-registry" onClick={() => set_route(3)}>
              開啟 Registry
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
