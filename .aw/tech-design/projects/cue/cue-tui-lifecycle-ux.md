---
id: cue-tui-lifecycle-ux
fill_sections: [wireframe, component, design-token, scenarios, changes]
---

## Wireframe
<!-- type: wireframe lang: yaml -->

```yaml
# Layout note: column widths are dynamic. Issues pane is 20% when issues
# is non-empty (else 0%). Detail pane is 20% when detail_body is Some
# (else 0%). Chat pane fills remainder (60–100%). Modal overlay is scoped
# to top-pane area only. HANDWRITE region: ratatui codegen target not yet
# in template registry; follow-up issue for ratatui wireframe codegen.
id: cue-tui-root
x-sdd:
  id: wireframe-cue-tui-root
  refs:
    - $ref: "#component-chat-bubble"
    - $ref: "#component-status-bar"
    - $ref: "#component-error-recovery-modal"
    - $ref: "#component-action-bar"
direction: vertical
width: auto
height: auto
children:
  - id: top-pane
    direction: horizontal
    flex: 1
    scroll: none
    description: "Top pane — contains issues list, chat, and detail; modal overlay scoped to this area only"
    children:
      - id: issues-list
        direction: vertical
        width: "20% iff issues.is_empty() == false, else 0%"
        scroll: vertical
        component: IssueList
        description: "Issues list panel — 20% width when issues exist, collapses to 0% otherwise"

      - id: chat-pane
        direction: vertical
        width: "100% - issues_pct - detail_pct"
        scroll: none
        description: "Chat pane — fills remaining width (60–100%); split into transcript + input"
        children:
          - id: chat-transcript
            direction: vertical
            flex: 1
            scroll: none
            description: "Transcript area; overflow handled by whole-bubble dropping (see design-token scroll-budget); one ChatBubble per message"
            children:
              - id: chat-bubble-slot
                component: ChatBubble
                description: "Repeated per message; role/pending/model/content from ChatState.messages"
                props:
                  role: "derived from ChatRole variant: User→User, Author→Author, Reviewer→Reviewer, Reviser→Reviser, System→System"
                  pending: "ChatMessage.pending"
                  model: "ChatMessage.model (optional)"
                  content: "ChatMessage.content"

          - id: chat-input
            height: 3
            scroll: none
            description: "Single-line input box; focus indicator ▶ prefix"

      - id: detail-pane
        direction: vertical
        width: "20% iff detail_body.is_some() == true, else 0%"
        scroll: none
        description: "Issue detail pane — 20% when detail_body is Some, 0% otherwise; no PhaseGauge in this pane (status bar badge is sole lifecycle indicator)"
        children:
          - id: issue-body
            flex: 1
            scroll: vertical
            description: "Markdown body of the selected issue"

  - id: log-pane
    height: 8
    scroll: vertical
    description: "Envelope/debug log ring buffer (last 200 lines). Entry format: dispatch → <role> (e.g. dispatch → score-issue-reviewer) or invoke → <verb> (last word of invoke.command, e.g. invoke → validate). Slug omitted."

  - id: action-bar
    height: 1
    scroll: none
    component: ActionBar
    description: "Context-aware keyboard shortcut hints — content depends on active modal and focus state"

  - id: status-bar
    height: 1
    scroll: none
    component: StatusBar
    props:
      state: "App.lifecycle_state"
      message: "App.status"

modal-layer:
  description: "Overlay scoped to top-pane area only (not log-pane, action-bar, or status-bar). Width: 100% of top-pane. Height: 60% of top-pane. Active when App.modal is Modal::ErrorRecovery or Modal::Gate or Modal::Message."
  component: ErrorRecoveryModal
  props:
    error_kind: "App.modal.error_kind"
    message: "App.modal.message"
    active_phase: "App.modal.active_phase"
    actions: "App.modal.actions"
    cursor: "App.modal.cursor"
  width: "100% of top-pane"
  height: "60% of top-pane"
  position: center
```
## Component
<!-- type: component lang: yaml -->

```yaml
components:
  - id: ChatBubble
    x-sdd:
      id: component-chat-bubble
      refs:
        - $ref: "#color-role-author"
        - $ref: "#color-role-reviewer"
        - $ref: "#color-role-reviser"
        - $ref: "#glyph-status-pending"
    props:
      role:
        type: string
        enum: [User, Author, Reviewer, Reviser, System]
        description: "Determines badge color from design-token color.role.{lowercase(role)}"
      pending:
        type: boolean
        description: "When true the bubble label line shows glyph.status.pending (…) appended to the role label; marker is on the label line, not embedded in content"
      model:
        type: string
        description: "LLM model identifier displayed on the label line, e.g. gemini-2.5-pro"
      content:
        type: string
        description: "Message body text; may contain newlines; rendered below the label line"
    rendering:
      label_line: "┌─ <role> (<model>)  [… if pending]"
      body_line: "content (rendered on lines below the label line)"
      note: "Role label is ALWAYS on its own dedicated line prefixed with ┌─, never concatenated with body content"
    states:
      streaming: "pending === true"
      complete: "pending === false"
      system_message: "role === 'System'"
    auto_scroll:
      algorithm: "wrap-aware row estimation; when total estimated rows exceed visible viewport rows, drop WHOLE bubbles from the front (label + body together); only when a single surviving bubble alone overflows do we fall back to line-level clipping inside it"
      note: "Ensures the most-recent bubble (including reviser body) is always visible without manual scrolling"
    interactions:
      "scroll_up": "emit:scroll-up"
      "scroll_down": "emit:scroll-down"
    slots: []
    events:
      - name: scroll-up
        payload:
          type: object
          properties:
            lines: { type: integer }
      - name: scroll-down
        payload:
          type: object
          properties:
            lines: { type: integer }

  - id: StatusBar
    x-sdd:
      id: component-status-bar
      refs:
        - $ref: "#color-phase-merged"
        - $ref: "#color-phase-error"
        - $ref: "#glyph-status-ok"
        - $ref: "#glyph-status-error"
    props:
      state:
        type: string
        enum: [Idle, Drafting, AwaitingReview, Revising, Merged, Error]
        description: "Application-level lifecycle state; determines badge glyph and color rendered by draw_status"
      message:
        type: string
        description: "Plain message body — no glyph or label prefix; draw_status prepends the [<glyph> <label>] badge separately. App.status MUST carry only the plain body (e.g. 'merged', not '✓ merged (merged)')."
    rendering:
      badge: "[<glyph> <label>] rendered by draw_status from state enum"
      body: "App.status — plain text only, no duplicated glyph prefix"
      note: "Combined status bar line is badge + body; double-glyph sequences are a contract violation"
    states:
      idle: "state === 'Idle'"
      active: "state === 'Drafting' || state === 'AwaitingReview' || state === 'Revising'"
      terminal_ok: "state === 'Merged'"
      terminal_err: "state === 'Error'"
    slots: []
    events: []

  - id: ErrorRecoveryModal
    x-sdd:
      id: component-error-recovery-modal
      refs:
        - $ref: "#color-phase-error"
    props:
      error_kind:
        type: string
        enum: [ScoreProcess, Llm, Internal]
        description: |
          Category of error; determines the kind banner and suggested recovery actions.
          ScoreProcess: DEFAULT for SessionError messages not matching other patterns.
          Llm: triggered when message contains 'llm', 'stream', or 'provider'.
          Internal: triggered for 'no model route', 'unsupported mainthread verb',
            'missing both agent and invoke', or 'unknown agent role'.
      message:
        type: string
        description: "Verbatim error text from the runner (SessionEvent::Error.message); the kind banner is rendered separately by the modal header"
      active_phase:
        type: string
        description: "The CRRR phase that was active when the error occurred"
      actions:
        type: array
        items:
          type: string
          enum: [retry, dismiss, new-issue]
        description: "Ordered list of recovery actions to display; cursor navigates among them"
      cursor:
        type: integer
        description: "Index of currently highlighted action (0-based)"
    rendering:
      overlay_area: "top-pane area only (not log-pane, action-bar, or status-bar)"
      width: "100% of top-pane area"
      height: "60% of top-pane area"
      kind_banner: "Modal header shows kind label: ScoreProcess / Llm / Internal"
      note: "System chat bubbles for errors MUST NOT add 'error: ' prefix; System role yellow color is the sole visual error signal; the modal kind banner carries the classification"
    states:
      retry_focused: "actions[cursor] === 'retry'"
      dismiss_focused: "actions[cursor] === 'dismiss'"
      new_issue_focused: "actions[cursor] === 'new-issue'"
    interactions:
      "key_up": "cursor = (cursor - 1 + actions.length) % actions.length"
      "key_down": "cursor = (cursor + 1) % actions.length"
      "key_enter": "emit:action-selected"
      "key_esc": "emit:dismissed"
    slots: []
    events:
      - name: action-selected
        payload:
          type: object
          properties:
            action: { type: string, enum: [retry, dismiss, new-issue] }
      - name: dismissed
        payload:
          type: "null"

  - id: ActionBar
    x-sdd:
      id: component-action-bar
    props:
      modal:
        type: string
        enum: [None, ErrorRecovery, Gate, Message]
        description: "Currently active modal kind"
      focus:
        type: string
        enum: [Chat, List, Detail]
        description: "Currently focused pane"
    rendering:
      context_rules:
        - condition: "modal === 'ErrorRecovery' || modal === 'Gate'"
          hint: "↑/↓ move | Enter confirm | Esc dismiss"
        - condition: "modal === 'Message'"
          hint: "Enter/Esc dismiss"
        - condition: "modal === 'None' && focus === 'Chat'"
          hint: "Enter submit | Tab/Esc focus list | q quit"
        - condition: "modal === 'None' && (focus === 'List' || focus === 'Detail')"
          hint: "n new | j/k nav | Enter open | a approve | v revise | q quit"
    states:
      modal_active: "modal !== 'None'"
      chat_focused: "modal === 'None' && focus === 'Chat'"
      list_focused: "modal === 'None' && (focus === 'List' || focus === 'Detail')"
    slots: []
    events: []
```
## Design Tokens
<!-- type: design-token lang: yaml -->

```yaml
$schema: "https://design-tokens.org/schema.json"
color:
  role:
    author:
      $value: "#00BCD4"
      $type: color
      $description: "Cyan — gemini-2.5-pro author agent chat bubble"
      $extensions:
        sdd:
          id: color-role-author
    reviewer:
      $value: "#4CAF50"
      $type: color
      $description: "Green — gpt-5 reviewer agent chat bubble"
      $extensions:
        sdd:
          id: color-role-reviewer
    reviser:
      $value: "#E040FB"
      $type: color
      $description: "Magenta — claude-opus-4-7 reviser agent chat bubble"
      $extensions:
        sdd:
          id: color-role-reviser
    user:
      $value: "#FFFFFF"
      $type: color
      $description: "White — human user chat bubble (256-color: Color::White)"
      $extensions:
        sdd:
          id: color-role-user
    system:
      $value: "#FFEB3B"
      $type: color
      $description: "Yellow — system/error notifications (256-color: Color::Yellow); sole visual signal for system-role messages; no 'error: ' prefix is added to message content"
      $extensions:
        sdd:
          id: color-role-system
  phase:
    drafting:
      $value: "#FFEB3B"
      $type: color
      $description: "Yellow — active authoring phase; requirements/scope/refctx in progress"
      $extensions:
        sdd:
          id: color-phase-drafting
    awaiting-review:
      $value: "#00BCD4"
      $type: color
      $description: "Cyan — all fill sections complete, reviewer not yet dispatched"
      $extensions:
        sdd:
          id: color-phase-awaiting-review
    revising:
      $value: "#E040FB"
      $type: color
      $description: "Magenta — reviser dispatched after needs-revision verdict"
      $extensions:
        sdd:
          id: color-phase-revising
    merged:
      $value: "#4CAF50"
      $type: color
      $description: "Green — Done envelope received; lifecycle complete"
      $extensions:
        sdd:
          id: color-phase-merged
    error:
      $value: "#F44336"
      $type: color
      $description: "Red — score error or LLM stream error; used by status bar badge and modal overlay"
      $extensions:
        sdd:
          id: color-phase-error
glyph:
  status:
    pending:
      $value: "..."
      $type: content
      $description: "Ellipsis appended to bubble label line when pending=true (not embedded in content body)"
      $extensions:
        sdd:
          id: glyph-status-pending
    ok:
      $value: "✓"
      $type: content
      $description: "Check mark shown in status bar badge when lifecycle reaches merged terminal state"
      $extensions:
        sdd:
          id: glyph-status-ok
    error:
      $value: "✗"
      $type: content
      $description: "Cross mark shown in status bar badge when lifecycle reaches error terminal state"
      $extensions:
        sdd:
          id: glyph-status-error
scroll-budget:
  chat-overflow:
    $value: "wrap-aware"
    $type: content
    $description: "Auto-scroll budget for chat transcript. Row estimation is wrap-aware (accounts for line wrapping at pane width). When total estimated rows exceed visible viewport rows: drop WHOLE bubbles from the front (label + body together, never split). Only when a single surviving bubble alone overflows do we fall back to line-level clipping within that bubble. Ensures most-recent bubble is always visible."
    $extensions:
      sdd:
        id: scroll-budget-chat-overflow
degradation:
  role-prefix:
    author:
      $value: "[author]"
      $type: content
      $description: "Text prefix used when terminal does not support 256-color mode"
    reviewer:
      $value: "[reviewer]"
      $type: content
      $description: "Text prefix for reviewer in non-256-color terminals"
    reviser:
      $value: "[reviser]"
      $type: content
      $description: "Text prefix for reviser in non-256-color terminals"
    user:
      $value: "[user]"
      $type: content
      $description: "Text prefix for user in non-256-color terminals"
    system:
      $value: "[system]"
      $type: content
      $description: "Text prefix for system in non-256-color terminals"
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
x-sdd:
  id: cue-tui-ux-scenarios
  refs:
    - $ref: "#component-chat-bubble"
    - $ref: "#component-status-bar"
    - $ref: "#component-error-recovery-modal"
    - $ref: "#component-action-bar"
    - $ref: "#scroll-budget-chat-overflow"

acceptance:
  note: "Each scenario's acceptance criterion is: rendered terminal buffer matches the cited snapshot file under projects/cue/tests/snapshots/"

scenarios:
  - id: sdd_issue_artifacts_populate_work_item_list
    title: "Work-item list renders SDD issue artifacts"
    actors:
      - id: User
        kind: actor
      - id: TUI
        kind: system
      - id: Runtime
        kind: system
    steps:
      - step: 1
        actor: TUI
        action: starts in a repository with `.aw/issues/open/*.md` and `.aw/issues/closed/*.md`
        expected:
          - IssueList is populated from `.aw/issues`, not `.cue/issues`
          - Open and closed SDD issue frontmatter fields phase/review_count/flagged_sections are reflected in IssueEntry rows
      - step: 2
        actor: Runtime
        action: emits Action::IssueCreated with a newly-created slug
        expected:
          - IssueList inserts the new slug exactly once
          - The new entry points at `.aw/issues/open/<slug>.md`
          - The new entry becomes selected so Enter opens the work item detail
      - step: 3
        actor: User
        action: presses `x` while an open work item is selected
        expected:
          - TUI dispatches Action::CloseIssue through Session::close_issue
          - On success, Action::IssueClosed marks the row closed and points it at `.aw/issues/closed/<slug>.md`
          - ActionBar includes the wired `x close` shortcut in list/detail focus
    acceptance:
      - test: cue::tui::issues::tests::scans_open_and_closed_issues
        assertion: scanner reads `.aw/issues/{open,closed}` frontmatter into IssueEntry records
      - test: cue::tui::app::tests::issue_created_adds_sdd_issue_to_list
        assertion: IssueCreated inserts a selected `.aw/issues/open/<slug>.md` row without duplicates
      - test: cue::tui::event::tests::x_emits_close_issue_for_selected_open_issue
        assertion: list/detail key handling emits Action::CloseIssue for the selected open work item
      - test: cue::tui::runner::tests::close_issue_dispatches_session_and_marks_issue_closed
        assertion: Runner calls Session::close_issue and the reducer marks the selected row closed

  - id: happy_path_full_crrr
    title: "Full CRRR lifecycle — happy path"
    snapshots:
      - "e2e_lifecycle_ux__happy_path__01_after_submit.snap"
      - "e2e_lifecycle_ux__happy_path__02_after_first_section.snap"
      - "e2e_lifecycle_ux__happy_path__03_terminal_merged.snap"
    actors:
      - id: User
        kind: actor
      - id: TUI
        kind: system
      - id: Runtime
        kind: system
    steps:
      - step: 1
        actor: User
        action: types issue title and presses Enter
        expected:
          - ChatBubble role=User content=<title> appended to transcript
          - StatusBar state=Drafting message="drafting requirements..."
          - ActionBar hint "Enter submit | Tab/Esc focus list | q quit"
          - snapshot: e2e_lifecycle_ux__happy_path__01_after_submit.snap
      - step: 2
        actor: Runtime
        action: AssistantDelta events arrive (author role, model=gemini-2.5-pro)
        expected:
          - ChatBubble role=Author pending=true model=gemini-2.5-pro; label line shows "┌─ Author (gemini-2.5-pro) ..."
          - color matches color.role.author (#00BCD4)
          - ChatBubble body renders on lines below the label line (no concatenation)
      - step: 3
        actor: Runtime
        action: AssistantMessageComplete for requirements section
        expected:
          - ChatBubble role=Author pending=false
          - StatusBar state=Drafting message updated
          - snapshot: e2e_lifecycle_ux__happy_path__02_after_first_section.snap
      - step: 4
        actor: Runtime
        action: scope and reference-context author turns complete
        expected:
          - Two more ChatBubble role=Author entries in transcript
          - StatusBar state=AwaitingReview message="awaiting review..."
      - step: 5
        actor: Runtime
        action: reviewer turn begins (model=gpt-5)
        expected:
          - ChatBubble role=Reviewer pending=true model=gpt-5; label line shows "┌─ Reviewer (gpt-5) ..."
          - color matches color.role.reviewer (#4CAF50)
      - step: 6
        actor: Runtime
        action: Envelope Done received (merge succeeded)
        expected:
          - StatusBar state=Merged; badge shows "✓ merged"; badge rendered by draw_status (no double-glyph in App.status)
          - StatusBar uses color.phase.merged (#4CAF50) + bold
          - ChatBubble role=System content="lifecycle complete — <slug>" (no "error:" prefix)
          - snapshot: e2e_lifecycle_ux__happy_path__03_terminal_merged.snap

  - id: needs_revision_branch
    title: "CRRR with needs-revision on first review"
    snapshots:
      - "e2e_lifecycle_ux__needs_revision__01_after_review_apply.snap"
      - "e2e_lifecycle_ux__needs_revision__02_reviser_engaged.snap"
      - "e2e_lifecycle_ux__needs_revision__03_terminal_merged.snap"
    actors:
      - id: User
        kind: actor
      - id: TUI
        kind: system
      - id: Runtime
        kind: system
    steps:
      - step: 1
        actor: Runtime
        action: reviewer turn emits needs-revision verdict
        expected:
          - ChatBubble role=Reviewer complete in transcript
          - StatusBar state=Revising message="revising flagged sections..."
          - color matches color.phase.revising (#E040FB)
          - snapshot: e2e_lifecycle_ux__needs_revision__01_after_review_apply.snap
      - step: 2
        actor: Runtime
        action: reviser turn begins (model=claude-opus-4-7)
        expected:
          - ChatBubble role=Reviser pending=true model=claude-opus-4-7; label line shows "┌─ Reviser (claude-opus-4-7) ..."
          - color matches color.role.reviser (#E040FB)
          - auto-scroll drops front bubbles whole if needed to keep reviser bubble visible
          - snapshot: e2e_lifecycle_ux__needs_revision__02_reviser_engaged.snap
      - step: 3
        actor: Runtime
        action: reviser turn complete; second reviewer round begins
        expected:
          - ChatBubble role=Reviser pending=false
          - ChatBubble role=Reviewer pending=true (second review)
          - StatusBar state=AwaitingReview
      - step: 4
        actor: Runtime
        action: second reviewer approves; Done envelope received
        expected:
          - StatusBar state=Merged; badge shows glyph.status.ok
          - snapshot: e2e_lifecycle_ux__needs_revision__03_terminal_merged.snap

  - id: score_process_failure_at_validate
    title: "score error after author completes"
    snapshots:
      - "e2e_lifecycle_ux__score_process_failure__01_modal_open.snap"
      - "e2e_lifecycle_ux__score_process_failure__02_cursor_dismiss.snap"
      - "e2e_lifecycle_ux__score_process_failure__03_after_new_issue.snap"
    actors:
      - id: User
        kind: actor
      - id: TUI
        kind: system
      - id: Runtime
        kind: system
    steps:
      - step: 1
        actor: Runtime
        action: author turn completes then Envelope Error (kind=ScoreProcess, default classifier) arrives
        expected:
          - StatusBar state=Error; badge shows "✗ <message>" from draw_status; App.status carries plain body only
          - color matches color.phase.error (#F44336)
          - ErrorRecoveryModal opens with kind banner "ScoreProcess", actions=[retry, dismiss, new-issue]
          - modal overlays top-pane only (log-pane and status bar remain visible)
          - ActionBar hint "↑/↓ move | Enter confirm | Esc dismiss"
          - snapshot: e2e_lifecycle_ux__score_process_failure__01_modal_open.snap
      - step: 2
        actor: User
        action: presses down/up to navigate modal, moves cursor to dismiss
        expected:
          - cursor moves among retry/dismiss/new-issue entries
          - active entry highlighted with bold
          - snapshot: e2e_lifecycle_ux__score_process_failure__02_cursor_dismiss.snap
      - step: 3
        actor: User
        action: presses n for new issue (or selects new-issue from modal)
        expected:
          - ErrorRecoveryModal closes (Modal::None)
          - new issue creation flow begins
          - snapshot: e2e_lifecycle_ux__score_process_failure__03_after_new_issue.snap

  - id: llm_stream_error_mid_author
    title: "LLM stream error mid-section during author turn"
    snapshots:
      - "e2e_lifecycle_ux__llm_stream_error__01_modal_open.snap"
      - "e2e_lifecycle_ux__llm_stream_error__02_after_esc_dismiss.snap"
    actors:
      - id: User
        kind: actor
      - id: TUI
        kind: system
      - id: Runtime
        kind: system
    steps:
      - step: 1
        actor: Runtime
        action: AssistantDelta events arriving (author streaming)
        expected:
          - ChatBubble role=Author pending=true partial content visible
      - step: 2
        actor: Runtime
        action: SessionEvent::Error arrives mid-stream (message contains 'stream' → kind=Llm)
        expected:
          - ChatBubble role=Author pending=false content=<partial text received so far>
          - ChatBubble role=System appended with plain error message — NO "error:" prefix prepended; System yellow color signals error context
          - StatusBar state=Error; badge shows glyph.status.error; App.status carries plain body
          - ErrorRecoveryModal opens with kind banner "Llm", actions=[retry, dismiss, new-issue]
          - modal overlays top-pane only
          - snapshot: e2e_lifecycle_ux__llm_stream_error__01_modal_open.snap
      - step: 3
        actor: User
        action: presses Esc to dismiss
        expected:
          - ErrorRecoveryModal closes
          - StatusBar state=Idle (awaiting user input)
          - snapshot: e2e_lifecycle_ux__llm_stream_error__02_after_esc_dismiss.snap
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cue/src/tui/chat.rs
    action: modify
    impl_mode: hand-written
    description: |
      ChatBubble label renders on its own dedicated line prefixed with "┌─ <role> (<model>)".
      Pending marker (…) is appended to the label line, not embedded in content body.
      ChatMessage gains model: Option<String> field for the model-name suffix.
      ChatState::begin_assistant accepts ChatRole parameter for role-aware variants.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.

  - path: projects/cue/src/tui/ui.rs
    action: modify
    impl_mode: hand-written
    description: |
      HANDWRITE region (ratatui codegen target not yet in template registry;
      follow-up issue: "add ratatui target to wireframe/component codegen template registry").
      draw_chat: ChatBubble label line rendered separately from body content (┌─ prefix);
        pending marker appended to label; role-to-color mapping from design-token constants.
      draw_status: badge [<glyph> <label>] rendered by draw_status; App.status carries plain
        body only (no glyph prefix); double-glyph sequences eliminated.
      draw_error_recovery_modal: overlay scoped to top-pane area at 100% width, 60% height;
        kind banner (ScoreProcess/Llm/Internal) shown in modal header.
      Dynamic column widths: issues_pct=20 iff !issues.is_empty() else 0;
        detail_pct=20 iff detail_body.is_some() else 0; chat fills remainder.
      PhaseGauge/CRRR Gauge removed from detail pane (status bar badge is sole indicator).
      System chat bubbles: no "error:" prefix prepended; yellow color is sole error signal.
      Auto-scroll: wrap-aware row estimation; whole-bubble dropping from front;
        line-level clipping only for single-bubble overflow.
      ActionBar list/detail focus includes `x close` only where the key is actually wired.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.

  - path: projects/cue/src/tui/app.rs
    action: modify
    impl_mode: hand-written
    description: |
      Modal::ErrorRecovery with kind classifier (ScoreProcess default; Llm for stream/llm/provider;
        Internal for no-model-route / unsupported-verb / missing-invoke / unknown-role).
      App::apply(Action::SessionError) opens modal with correct kind derived from message content.
      App.status carries plain message body only (no glyph prefix).
      Detail pane detail_body field drives column visibility; no lifecycle_phase for PhaseGauge.
      Action::IssueCreated inserts the new SDD work item into the IssueList exactly once and
        selects it so the user can immediately open/manage the newly-created issue.
      Action::CloseIssue records the pending close request; Action::IssueClosed marks the
        selected IssueEntry closed and updates its canonical `.aw/issues/closed/<slug>.md` path.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.

  - path: projects/cue/src/tui/actions.rs
    action: modify
    impl_mode: hand-written
    description: |
      Adds Action::CloseIssue as the TUI-to-runner command for selected work-item management.
      Adds Action::IssueClosed as the async backend success event consumed by App::apply.

  - path: projects/cue/src/tui/issues.rs
    action: modify
    impl_mode: hand-written
    description: |
      Issue scanner reads `.aw/issues/{open,closed}` as the work-item source of truth.
      IssueEntry::sdd_open constructs the canonical path for newly-created local SDD issues.
      IssueEntry::mark_closed updates the selected row to the canonical closed SDD issue path.

  - path: projects/cue/src/tui/mod.rs
    action: modify
    impl_mode: hand-written
    description: |
      Startup scan status/error context references `.aw/issues`, matching the SDD artifact layout.

  - path: projects/cue/src/tui/event.rs
    action: modify
    impl_mode: hand-written
    description: |
      Modal navigation keys for ErrorRecovery, Gate, Message modal kinds.
      ActionBar context-aware key routing: modal-active vs chat-focused vs list/detail-focused.
      List/detail key `x` emits Action::CloseIssue for an open selected work item.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.

  - path: projects/cue/src/tui/runner.rs
    action: modify
    impl_mode: hand-written
    description: |
      ChatRole derived from last_dispatch.agent field for role-aware bubble creation.
      SessionEvent::Error kind derived from message content (Llm/Internal/ScoreProcess default).
      Log entry format: dispatch → <role> or invoke → <verb>; slug omitted.
      Action::CloseIssue spawns Session::close_issue and emits Action::IssueClosed or SessionError.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.

  - path: projects/cue/tests/e2e_lifecycle_ux.rs
    action: modify
    impl_mode: hand-written
    description: |
      E2e test file exercising the four scenarios: happy_path_full_crrr,
      needs_revision_branch, score_process_failure_at_validate, llm_stream_error_mid_author.
      Snapshot baselines at projects/cue/tests/snapshots/e2e_lifecycle_ux__*.snap
      serve as the visual acceptance contract (11 snapshot files total).
      Acceptance criterion: rendered terminal buffer matches the cited snapshot.
      Implementation shipped on cue branch at commits f83ac54de → 6a824a8b0.
      Post-iteration visual fixes: 6 P0/P1 + 1 P2 committed at
      4a46e620b, e433badef, 07686f90e, 6a824a8b0, 94ee46766.

  - path: projects/cue/tests/support/harness.rs
    action: modify
    impl_mode: hand-written
    description: |
      Test harness action-kind rendering includes CloseIssue and IssueClosed so
      every Action variant remains exhaustively covered by e2e diagnostics.
```
# Reviews

### Review 1
**Verdict:** approved

- [wireframe] Dynamic width rules are unambiguous: issues-list 20%/0%, detail-pane 20%/0%, chat fills remainder (60–100%). Modal overlay correctly scoped to top-pane only. PhaseGauge correctly absent from detail-pane description. All four `$ref` ids (`#component-chat-bubble`, `#component-status-bar`, `#component-error-recovery-modal`, `#component-action-bar`) resolve to matching `x-sdd.id` values in the Component section.
- [component] ChatBubble label-on-own-line rule is explicit (`note:` field, line 139). StatusBar badge/body split contract stated clearly (double-glyph sequences flagged as contract violation). ErrorRecoveryModal kind classifier rules cover all three enum values with concrete trigger patterns. ActionBar renders 4 context rules covering all modal/focus combinations.
- [design-token] `scroll-budget.chat-overflow` present with wrap-aware algorithm and whole-bubble-drop policy. `glyph.status.pending` placement on label line (not content body) documented. `color.role.system` clarified with explicit no-`error:`-prefix note.
- [scenarios] All 4 scenario ids cite snapshot files by exact filename; 11 snapshot files confirmed present at `projects/cue/tests/snapshots/`. Acceptance criterion stated as buffer-match against cited snapshot. Snapshot count in Changes (11 total) matches the actual file count.
- [changes] All 5 source files listed (`tui/{ui,chat,app,event,runner}.rs`) plus `tests/e2e_lifecycle_ux.rs`. `impl_mode: hand-written` consistent with ratatui codegen-gap rationale noted in Wireframe. Post-iteration commits (4a46e620b → 94ee46766) referenced in Changes for traceability.
