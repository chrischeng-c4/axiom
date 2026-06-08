# Reference: gemini-cli Architecture Notes

Status: legacy reference.

This document was written for the old Rust + ratatui terminal-agent direction.
cue is now a web-based Prompt-to-Governed-App platform. These notes should not
drive the product surface; keep only reusable harness concepts such as provider
abstraction, event streams, tool scheduling, sandboxing, and testability.

Source: https://github.com/google-gemini/gemini-cli (commit tip of `main`, fetched via
GitHub API and raw.githubusercontent.com).

Purpose: research notes for reusable agent-harness mechanics that may still
support cue's governed web platform. Borrow concepts, not code. The old
terminal-agent product direction is obsolete; only provider abstraction,
event streams, tool scheduling, sandboxing, and testability remain relevant.

All paths in this doc are repo-relative to `google-gemini/gemini-cli@main`.

## Overall Architecture

- TypeScript npm monorepo. Top-level workspaces live under `packages/`:
  - `packages/cli/` — terminal app (ink-based React TUI, slash commands, headless
    mode, non-interactive CLI entrypoints).
  - `packages/core/` — "engine": provider clients, chat/turn loop, tool registry,
    tool scheduler, sandbox, MCP client, routing, prompts, skills, subagents.
  - `packages/sdk/` — public SDK surface for embedding.
  - `packages/a2a-server/` — agent-to-agent server (remote agents / subagents
    over the wire).
  - `packages/vscode-ide-companion/` — VS Code extension that talks to the CLI
    via the IDE integration channel.
  - `packages/devtools/`, `packages/test-utils/` — internal tooling.
- Build: `esbuild` bundle, `tsconfig` project references, `eslint` + `prettier`.
  Single-executable builds produced under `sea/`.
- Three execution modes:
  1. Interactive TUI (`gemini` with ink).
  2. Non-interactive CLI (`gemini -p "..."` with text / JSON / stream-JSON
     output; see `packages/cli/src/nonInteractiveCli.ts`).
  3. Headless agent session (`packages/cli/src/nonInteractiveCliAgentSession.ts`),
     used by the GitHub Action and by ACP (agent-control-protocol) integrations.
- Split of concerns is strict: the CLI package owns ink/rendering/input; the core
  package owns everything that could be reused from a non-interactive or remote
  context. `packages/core/src/index.ts` is the single export boundary.

Key paths:
- `packages/cli/src/gemini.tsx` — entrypoint wiring ink render to core.
- `packages/cli/src/interactiveCli.tsx` — interactive session bootstrap.
- `packages/cli/src/ui/App.tsx` / `AppContainer.tsx` — top-level TUI component.
- `packages/core/src/index.ts` — core public API.

## Provider Abstraction and Streaming

- The provider surface lives in `packages/core/src/core/contentGenerator.ts` —
  a `ContentGenerator` abstraction wraps the model call. Concrete generators
  include the real Gemini API client, a `fakeContentGenerator` for tests, a
  `recordingContentGenerator` (tape-based replay), a `loggingContentGenerator`
  (telemetry wrapper), and a `localLiteRtLmClient` (on-device LiteRT models).
- The main conversational loop is `packages/core/src/core/geminiChat.ts`, with
  retry/backoff exercised in `geminiChat_network_retry.test.ts`. `client.ts` and
  `baseLlmClient.ts` sit between the chat loop and the transport layer.
- Streaming uses Gemini's native SSE/streamGenerateContent. The CLI consumes the
  stream as typed events (text deltas, tool-call deltas, usage/metadata). Events
  propagate up into the ink render loop so the UI can render chunks as they
  arrive.
- Non-interactive output can be `stream-json` (newline-delimited JSON events),
  which makes this loop scriptable — matches what `claude -p --output-format
  stream-json` does.
- `contentGenerator` is the seam you'd extend for multi-provider (Anthropic,
  OpenAI) support. gemini-cli itself only targets Gemini (+ LiteRT local), so
  the interface is narrower than we will need; not found in public docs: any
  OpenAI/Anthropic shim, since gemini-cli has never supported them.

Key paths:
- `packages/core/src/core/contentGenerator.ts`
- `packages/core/src/core/geminiChat.ts`
- `packages/core/src/core/client.ts`
- `packages/core/src/core/baseLlmClient.ts`
- `packages/core/src/core/localLiteRtLmClient.ts`

## TUI Architecture (ink)

- Renderer is [ink](https://github.com/vadimdemedes/ink) — React reconciler that
  renders JSX to ANSI via yoga-layout. The whole TUI is a tree of functional
  components with hooks; there is no manual draw loop.
- Top-level structure under `packages/cli/src/ui/`:
  - `App.tsx` / `AppContainer.tsx` — root component, owns global state provider
    tree.
  - `components/` — message list, input box, tool confirmation, status bar,
    dialogs (auth, settings, theme).
  - `layouts/` — higher-level layout composition (header / scrollback / footer).
  - `hooks/` — custom React hooks for input handling, stream consumption, tool
    confirmation queueing.
  - `contexts/` — React context providers for config, session, theme, policy.
  - `state/` — app-level state store.
  - `themes/` + `colors.ts` + `semantic-colors.ts` — theme tokens; see
    `docs/cli/themes.md`.
  - `editors/` — in-line editor integrations (`/editor` dialog).
  - `key/` — keybinding table; `/vim` mode toggle and `/terminal-setup` tie in
    here.
- Input handling is driven by ink's `useInput` hook plus the project's own
  keybinding layer. Multi-line input requires terminal-specific keybindings
  (`/terminal-setup`).
- Message rendering is component-based: each message role (user, model, tool,
  tool-result, system) has its own React component; markdown is rendered via an
  in-process renderer that maps markdown AST to ink's `<Text>` / `<Box>`.
- For `cue` (ratatui): the conceptual takeaway is to model the TUI as a
  component tree with a unidirectional data-flow store, not a procedural draw
  loop. ratatui's immediate-mode paradigm is different from ink's retained VDOM,
  but the boundary between "state store -> derived view models -> widgets" is
  the same design pattern.

Key paths:
- `packages/cli/src/ui/App.tsx`
- `packages/cli/src/ui/AppContainer.tsx`
- `packages/cli/src/ui/components/` (tool confirmation, message list, etc.)
- `packages/cli/src/ui/hooks/`
- `packages/cli/src/ui/themes/`

## Tool System

- Built-in tools live in `packages/core/src/tools/`. Each tool is a TypeScript
  class implementing a common tool interface, with schema declared in code.
- Catalog of built-ins (from `docs/tools/` + `packages/core/src/tools/`):
  - File system: `read_file`, `write_file`, `read_many_files`, `replace` (aka
    `edit`), `ls` (list_directory), `glob`, `grep` (with `ripGrep` variant),
    `jit-context` (just-in-time GEMINI.md discovery).
  - Shell: `run_shell_command` (`shell.ts`), plus background-shell variants.
  - Web: `web-fetch`, `web-search` (Google Search grounding).
  - Interaction: `ask-user`, `complete-task`, `todos`, `tracker`.
  - Agent control: `enter-plan-mode`, `exit-plan-mode`, `activate-skill`.
  - Memory: `memoryTool` (legacy `save_memory`), internal-docs accessor.
  - MCP: `mcp-tool`, `list-mcp-resources`, `read-mcp-resource`, plus
    `mcp-client` / `mcp-client-manager` for discovery.
- Schemas are JSON-Schema-ish objects declared inline in each tool file; the
  tool list is advertised to the model via Gemini's `tools` / `function_declarations`
  field.
- Tool approval / permissions are centralized in the Policy Engine
  (`packages/core/src/policy/`, documented in `docs/reference/policy-engine.md`).
  There's also a "context-aware security checker" — an LLM-driven policy layer
  that generates per-request policies dynamically.
- Prefix-match allowlist for shell: `tools.core` with entries like
  `run_shell_command(git)` allows everything starting with `git`. Blocklist
  entries take precedence. Chained commands (`&&`, `||`, `;`) are split and
  each segment validated independently.
- `modifiable-tool.ts` provides a pattern where a tool exposes an "edit the
  proposed call before confirming" affordance (user can tweak args in the
  approval dialog).

Key paths:
- `packages/core/src/tools/shell.ts`
- `packages/core/src/tools/edit.ts`, `read-file.ts`, `write_file` (via edit)
- `packages/core/src/tools/glob.ts`, `grep.ts`, `ripGrep.ts`
- `packages/core/src/tools/mcp-client.ts`, `mcp-tool.ts`
- `packages/core/src/tools/confirmation-policy.ts`
- `packages/core/src/policy/` (policy engine)
- `docs/reference/tools.md`, `docs/reference/policy-engine.md`

## Tool Scheduler

- Separate subsystem at `packages/core/src/scheduler/` owns the execution of a
  batch of model-requested tool calls:
  - `scheduler.ts` — main scheduler loop; supports parallel execution
    (`scheduler_parallel.test.ts`).
  - `state-manager.ts` — tracks per-tool-call state (queued, awaiting approval,
    executing, done, errored).
  - `confirmation.ts` — user approval flow, integrates with the `ask-user` and
    modifiable-tool paths.
  - `tool-executor.ts` — actually invokes the tool; wraps sandbox boundary.
  - `tool-modifier.ts` — apply user edits to a pending tool call before run.
  - `hook-utils.ts` — fires lifecycle hooks (pre-tool / post-tool) per
    `docs/hooks/`.
- Parallel tool execution is first-class (scheduler can run multiple calls from
  one model turn concurrently). Model gets all results back in one response
  round-trip.
- Confirmation is a first-class FSM, not an ad-hoc prompt: tool calls queue,
  user approves or rejects, approved ones run, results stream back.

Key paths:
- `packages/core/src/scheduler/scheduler.ts`
- `packages/core/src/scheduler/state-manager.ts`
- `packages/core/src/scheduler/confirmation.ts`
- `packages/core/src/scheduler/tool-executor.ts`

## Sandbox / Execution Safety

- Implementation under `packages/core/src/sandbox/`, split by OS:
  - `macos/` — Seatbelt via `sandbox-exec`. `MacOsSandboxManager.ts` is the
    entry point; `seatbeltArgsBuilder.ts` builds the `-D` parameters;
    `baseProfile.ts` has the shared `.sb` profile template.
  - `linux/` — gVisor / LXC / container-based.
  - `windows/` — native integrity-level sandbox (persistent changes, needs
    `icacls` reset — caveat in `docs/cli/sandbox.md`).
- Cross-platform fallback: Docker or Podman container. The container images ship
  pre-baked with Node + CLI bundle; sandbox mounts the project dir read-write
  and a tight set of system paths read-only.
- Tool-level sandboxing: individual tools (e.g. `run_shell_command`) can be
  sandboxed while the rest of the agent runs on the host. This is the common
  case — gives the local environment ergonomics (you can read GEMINI.md and
  project files normally) while still isolating shelled-out commands.
- Sandbox expansion: when a sandboxed command fails on a permission error, the
  CLI raises a "Sandbox Expansion Request" dialog letting the user widen the
  profile for that session.
- Enabled via `gemini -s`, `GEMINI_SANDBOX=true`, or `settings.json`.
- "Trusted folders": per-directory policy saying "in this workspace, I trust
  this class of operations without prompting". See `docs/cli/trusted-folders.md`.

Key paths:
- `packages/core/src/sandbox/macos/MacOsSandboxManager.ts`
- `packages/core/src/sandbox/macos/seatbeltArgsBuilder.ts`
- `packages/core/src/sandbox/macos/baseProfile.ts`
- `docs/cli/sandbox.md`

## Slash Commands

- Listed comprehensively in `docs/reference/commands.md`. Highlights:
  - Session: `/chat`, `/resume`, `/clear`, `/compress`, `/rewind`, `/restore`,
    `/stats`, `/copy`, `/quit`.
  - Config: `/settings`, `/auth`, `/model`, `/theme`, `/editor`, `/vim`,
    `/terminal-setup`, `/permissions`, `/policies`.
  - Knowledge: `/memory`, `/init` (generates GEMINI.md), `/docs`, `/help`,
    `/about`, `/bug`.
  - Agents: `/agents` (subagents), `/skills`, `/plan`, `/hooks`, `/commands`
    (reload custom commands), `/extensions`, `/mcp`, `/tools`, `/directory`,
    `/shells`.
  - IDE / integrations: `/ide`, `/setup-github`, `/upgrade`, `/privacy`.
- Built-in commands are implemented under `packages/cli/src/ui/commands/`.
- User-defined commands: TOML files under `~/.gemini/commands/` (global) or
  `<project>/.gemini/commands/` (project). Schema from
  `docs/cli/custom-commands.md`:
  - Required: `prompt` (single or multi-line).
  - Optional: `description`.
  - Name derives from file path: `git/commit.toml` -> `/git:commit`.
  - `{{args}}` placeholder injects raw user args (escaped inside shell blocks).
  - `!{...}` blocks run shell commands and inject their output (requires
    confirmation).
  - `@{...}` blocks inject file content / directory listings; respects
    `.gitignore`.
  - Order of substitution: files -> shell -> args.
- `/commands reload` hot-reloads TOML files without restarting.

Key paths:
- `packages/cli/src/ui/commands/`
- `docs/cli/custom-commands.md`
- `docs/reference/commands.md`

## Context and Memory Management

- Three layers:
  1. **Conversation history** — kept in `packages/core/src/core/geminiChat.ts`.
     Token-budget aware; when close to the limit, triggers compression.
  2. **GEMINI.md hierarchy** — global (`~/.gemini/GEMINI.md`), workspace (walks
     up from cwd to project root), and just-in-time (injected when a tool
     accesses a path). `@file.md` syntax imports other files
     (`docs/cli/gemini-md.md`). Default filename configurable in
     `settings.json` via `context.fileName` (people alias to `AGENTS.md`, etc.).
  3. **Agent Skills** — directories under `.gemini/skills/` (workspace) or
     `~/.gemini/skills/` (user). A `SKILL.md` inside each; only the name +
     description are eagerly loaded so the model can decide to activate.
     Activation goes through the `activate_skill` tool, gated by user
     confirmation; after activation the folder contents are injected.
- Compression is "information-preserving": the chat loop summarises older
  turns rather than dropping them. `/compress` triggers it manually.
- Checkpointing: every significant state change writes a snapshot under
  `~/.gemini/tmp/<project_hash>/checkpoints` (JSON of history + tool call)
  AND a Git commit in a shadow repo at `~/.gemini/history/<project_hash>`.
  `/restore` reverts both files and conversation to a chosen checkpoint.
  `/rewind` is the interactive "step backwards" variant (`docs/cli/rewind.md`).
- Memory discovery service (`packages/core/src/context/`) finds GEMINI.md files
  walking up directories; `jit-context.ts` is the tool side that injects them
  on demand when a path is touched.

Key paths:
- `packages/core/src/core/geminiChat.ts` (compression lives here)
- `packages/core/src/context/` (memory / file discovery)
- `packages/core/src/tools/jit-context.ts`
- `docs/cli/gemini-md.md`, `docs/cli/skills.md`, `docs/cli/checkpointing.md`

## Subagents and Remote Agents

- `docs/core/subagents.md` documents first-class subagent support.
- Built-in subagents: `codebase_investigator`, `cli_help`, `generalist`,
  `browser_agent` (experimental).
- Custom subagents: markdown files with YAML frontmatter under
  `.gemini/agents/` (project) or `~/.gemini/agents/` (user). Frontmatter:
  `name`, `tools` (subset; omit to inherit), `model` (override routing),
  `max_turns`. The markdown body becomes the system prompt.
- Dispatch: automatic by main agent, or explicit via `@subagent_name`.
- Isolation: subagents keep independent history; **recursion is blocked**
  (subagents cannot call other subagents). Deliberate no-recursion rule to
  prevent token blowups.
- Remote agents: `packages/a2a-server/` exposes subagents over the wire; ACP
  mode (`docs/cli/acp-mode.md`, `packages/cli/src/acp/`) lets another tool
  drive the CLI as an agent backend.

## MCP Support

- Model Context Protocol is first-class; see `docs/tools/mcp-server.md`.
- Configure servers in `settings.json` under `mcpServers`. Transports:
  `stdio` (subprocess), `sse`, `http`.
- Tool naming is namespaced: `mcp_<serverName>_<toolName>` to avoid collisions.
- Env var handling: auto-redacts patterns matching `*TOKEN*`/`*SECRET*`.
  Explicit values bypass redaction; use `$VAR` expansion to avoid hardcoding.
- Filter tools per-server via `includeTools` / `excludeTools`.
- Management commands (non-interactive too): `gemini mcp add/list/remove
  /enable/disable`. In-session: `/mcp`.
- Implementation: `packages/core/src/mcp/` and
  `packages/core/src/tools/mcp-client{,-manager}.ts`.

Key paths:
- `packages/core/src/mcp/`
- `packages/core/src/tools/mcp-client.ts`
- `docs/tools/mcp-server.md`

## Configuration, Settings, Auth

- Settings hierarchy (workspace overrides user):
  - `~/.gemini/settings.json`
  - `<project>/.gemini/settings.json`
  - Overridden further by env vars and CLI flags.
- Auth options (via `/auth`):
  - OAuth with Google account (free tier).
  - Gemini API key (direct billing).
  - Vertex AI for enterprise.
- Credentials storage: `packages/core/src/core/apiKeyCredentialStorage.ts`.
- Settings categories from `docs/cli/settings.md`:
  - General: vim mode, approval modes, auto-update.
  - UI: theme, window title, accessibility (screen reader, inline thinking).
  - Security: sandbox, tool approval, env-var redaction, context-aware
    security checker.
  - Model & Context: active model, history limit, compression threshold.
  - Tools: sandbox paths, shell behavior, ripgrep toggle, output truncation.
  - Experimental: git-worktree management, memory v2 (markdown-backed),
    web-fetch behavior.
- Folder-trust (`docs/cli/trusted-folders.md`): gates execution policy per
  directory — running `gemini` in an untrusted dir restricts tool access.

## Model Routing

- `packages/core/src/routing/modelRouterService.ts` orchestrates per-turn model
  selection using a composite-strategy pipeline. Registered order:
  1. `FallbackStrategy`
  2. `OverrideStrategy`
  3. `ApprovalModeStrategy`
  4. `GemmaClassifierStrategy` (when enabled)
  5. `ClassifierStrategy`
  6. `NumericalClassifierStrategy` (when enabled)
  7. `DefaultStrategy` (terminal)
- First strategy that returns a decision wins. On exception the router falls
  back to the configured default model. All decisions logged via telemetry.
- `docs/cli/plan-mode.md` documents one concrete use: in Plan Mode, router
  sends planning turns to Pro (high reasoning) and implementation turns to
  Flash (fast). `Shift+Tab` toggles mode.
- `docs/cli/model-steering.md` / `docs/cli/model.md` expose user-facing model
  selection; model precedence is `--model` flag > `GEMINI_MODEL` env var >
  `settings.json` > local-routing (Gemma) > auto default.

Key paths:
- `packages/core/src/routing/modelRouterService.ts`
- `packages/core/src/routing/strategies/` (7 strategy files)
- `packages/core/src/fallback/`

## Other Notable Subsystems

- **Hooks** (`docs/hooks/`): pre/post tool-call hooks users can register to
  intercept or modify behavior — akin to Claude Code's hooks. Wired through
  `packages/core/src/hooks/` and `scheduler/hook-utils.ts`.
- **Policy engine** (`packages/core/src/policy/`, `docs/reference/policy-engine.md`):
  declarative rules governing what tools can run in what mode.
- **Safety** (`packages/core/src/safety/`): content-safety checks separate from
  policy.
- **Skills** vs custom commands vs subagents — three distinct extensibility
  surfaces aimed at different granularities (skill = capability module,
  command = prompt macro, subagent = isolated sub-context).
- **IDE integration** (`packages/core/src/ide/`, `packages/vscode-ide-companion/`):
  the CLI can talk to a VS Code companion extension to share workspace state.

## Takeaways for cue

1. **Split the crate tree the way gemini-cli splits packages.** The `cli <-> core`
   boundary (ink / rendering / input vs. provider + tools + scheduler + sandbox)
   pays for itself the moment you add a second frontend (non-interactive CLI,
   remote/ACP, embed in another app). For cue: put the ratatui layer in one
   crate and everything model/tool/scheduler-shaped in another, with a thin SDK
   crate exported for scripting. Resist letting TUI concerns leak into core.

2. **Make the tool scheduler a first-class FSM, not inline logic.** gemini-cli
   has a dedicated `scheduler/` subsystem with explicit states (queued / awaiting
   approval / executing / done), parallel execution, user-modifiable tool calls,
   and lifecycle hooks. For cue this maps to a tokio actor or state machine; the
   SDD "Crrr validate" pipeline is just another client of this scheduler.

3. **Model routing = composite strategy chain, not `if/else`.** The ordered
   strategy list in `modelRouterService.ts` is exactly the abstraction cue needs
   for "route this SDD task to Claude Opus, that one to GPT-5, classify via a
   cheap model first". Steal the shape: `Override -> ApprovalMode ->
   Classifier -> Default`. Each strategy is a trait impl in Rust terms.

4. **Context is three layers, treat them distinctly.** Conversation history
   (with compression) + GEMINI.md hierarchy (+ `@file` imports, + configurable
   filename so AGENTS.md / CUE.md can share) + on-demand Skills packages. Don't
   dump everything into the system prompt; lazy-load skill bodies only when
   activated, same as gemini-cli's `activate_skill` flow. For cue this fits
   neatly on top of the existing score-style spec-as-source-of-truth stance.

5. **Sandbox should be a first-class seam, per-OS, with tool-level granularity.**
   gemini-cli's approach (macOS Seatbelt profiles, Linux gVisor/LXC, Windows
   integrity levels, Docker fallback, plus "Sandbox Expansion Request" UX on
   denied ops) is the right target. For cue/Rust: start with macOS
   `sandbox-exec` via a `seatbeltArgsBuilder`-equivalent; don't try to sandbox
   the whole agent process, sandbox the shell tool like gemini-cli does.

Things to explicitly **not** borrow:
- Ink/React VDOM retained-mode TUI — ratatui is immediate-mode; copying ink's
  component structure mechanically will fight ratatui's draw model. Borrow the
  unidirectional state-store pattern, not the reconciler.
- Tight coupling between tool schemas and a specific provider's function-calling
  format. gemini-cli advertises tools as Gemini `function_declarations`. cue is
  multi-provider day one, so the tool schema needs to be provider-agnostic with
  per-provider adapters at the edge (similar to how MCP tool schemas are
  translated per server).
- Custom-command TOML format. TOML is fine but the surface is narrow; cue's SDD
  slash commands already need structured metadata (section router, phase,
  envelope) — extend with YAML frontmatter like subagents use, not TOML.

---

Further reading worth pulling when deeper questions come up:
- `docs/hooks/` (intercepting tool execution).
- `docs/reference/policy-engine.md` (declarative safety).
- `docs/cli/acp-mode.md` + `packages/cli/src/acp/` (driving the CLI as an agent
  backend — relevant for cue's "conductor can dispatch cue sessions" story).
- `docs/cli/headless.md` + `packages/cli/src/nonInteractiveCliAgentSession.ts`
  (stream-json output format for scripting).
