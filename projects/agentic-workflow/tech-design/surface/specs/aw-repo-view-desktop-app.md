---
id: aw-repo-view-desktop-app
summary: Read-only AW repo visual-reader desktop app and snapshot for aw view.
capability_refs:
  - id: repo-view-desktop-app
    role: primary
    gap: repo-desktop-reader
    claim: repo-desktop-reader
    coverage: full
    rationale: "This TD/EC source defines the externally observable repo reader desktop app and snapshot contract for aw view."
---

# AW Repo View Desktop App

`aw view` is a read-only repository reader. The repository catalog is the root
surface; projects and libraries are selectable detail panes. The app is a native
desktop window over the same repo snapshot and renderer-neutral UI element tree
used by tests. Its primary desktop layout is a three-column reader: the left
column is the projects/libs list, and the right-side workspace can lay out the
agent terminal/status stream and current README, capability, EC, and TD detail
view either left-right or top-bottom. The default layout is left-right, and the
native app exposes a layout toggle button backed by the same semantic surface
node used by headless tests.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: aw-view-repo-snapshot
    capability_id: repo-view-desktop-app
    claim_id: repo-desktop-reader
    contract_id: aw-view-repo-snapshot
    category: behavior
    command: "./target/debug/aw view --snapshot"
    assertions:
      - "snapshot root is the repository, not a single project"
      - "snapshot includes the projects/libs catalog"
      - "snapshot includes terminal status for watching agent-side changes"
      - "snapshot includes selected README detail and capability rows"
      - "snapshot includes selected EC inventory and TD summary"
      - "snapshot includes a renderer-neutral surface tree for renderer-independent tests"
  - id: aw-view-repo-check
    capability_id: repo-view-desktop-app
    claim_id: repo-desktop-reader
    contract_id: aw-view-repo-check
    category: behavior
    command: "./target/debug/aw view --check"
    assertions:
      - "headless repo view check contains a projects/libs catalog"
      - "headless repo view check contains a terminal pane"
      - "headless repo view check contains README, capability, EC, and TD detail panes"
      - "headless repo view check exposes a semantic layout toggle button"
      - "headless repo view check preserves stable semantic IDs for e2e assertions"
  - id: aw-view-repo-screenshot
    capability_id: repo-view-desktop-app
    claim_id: repo-desktop-reader
    contract_id: aw-view-repo-screenshot
    category: behavior
    command: "./target/debug/aw view --screenshot /private/tmp/aw-view-app.png"
    assertions:
      - "app screenshot is produced without a browser or desktop screen capture"
      - "app screenshot is rendered from the same renderer-neutral surface tree"
      - "app screenshot contains project list, terminal status, and selected README, capability, EC, and TD detail panes"
  - id: aw-view-repo-layout-option
    capability_id: repo-view-desktop-app
    claim_id: repo-desktop-reader
    contract_id: aw-view-repo-layout-option
    category: behavior
    command: "./target/debug/aw view --layout top-bottom --screenshot /private/tmp/aw-view-app-top-bottom.png"
    assertions:
      - "project list stays fixed while the terminal/detail region can switch layout"
      - "top-bottom layout screenshot is rendered without changing the default left-right layout"
      - "layout is an explicit CLI option for the native repo view renderer"
      - "app screenshot includes the visible layout toggle control"
  - id: aw-view-repo-app-bundle
    capability_id: repo-view-desktop-app
    claim_id: repo-desktop-reader
    contract_id: aw-view-repo-app-bundle
    category: behavior
    command: "./target/debug/aw view --app /private/tmp/AWRepoView.app"
    assertions:
      - "native desktop bundle is produced as a macOS .app launcher"
      - "app bundle launches the repo-built aw view desktop surface"
      - "app bundle is produced without a browser or web runtime dependency"
```
