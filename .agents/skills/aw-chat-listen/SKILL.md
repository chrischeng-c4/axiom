---
name: aw:chat:listen
description: Set up real-time monitoring of the cross-WT aw chat channel via Monitor tool - replaces cron polling with stdout streaming
user-invocable: true
---

# /aw:chat:listen

Set up real-time monitoring of the cross-WT chat channel (`/tmp/aw-channel.md`).

## Why a skill instead of a cron

Earlier prototypes used a cron firing a one-shot poller every 60s, but that pattern is retired (the `--once` flag has been removed from `aw chat listen` per G3). Cron-based polling had two problems:
1. **High cost** -- every fire was a full LLM turn (read prompt -> run cmd -> decide action), even when channel was silent.
2. **Latency** -- up to 60s lag before mainthread saw a new message.

This skill uses **Monitor + long-running `aw chat listen`**:
- `aw chat listen` runs in a long-running loop -- its internal 60s polling produces one stdout line per new message.
- Mainthread uses the `Monitor` tool on that process -- each stdout line becomes a notification with sub-second latency.
- Cost is near-zero when silent (no LLM turn fires); only a notification + tiny mainthread response per actual new message.

## Default filter (4 rules, no flags required)

`aw chat listen` without any flags applies a four-rule filter so only relevant messages are emitted. You do NOT need `--mentions @me` -- the default already includes direct cues and echo.

| Rule | Category | When emitted |
|------|----------|--------------|
| 0 | `--all` override | `--all` flag set: emit every message |
| 1 | `direct_cue` | `msg.to` contains the caller team name |
| 2 | `broadcast` | `msg.to` is empty (addressed to @all) |
| 3 | `echo` | `msg.from` == caller team name (own posts reflected back) |
| 4 | `thread_member` | Caller participated earlier in the same thread (was cued mid-thread) |

Thread membership (Rule 4) is computed dynamically from the full channel snapshot on each poll -- once the caller is @-cued in any message in a thread, all subsequent replies to that thread are delivered even if those replies do not address the caller directly.

Use `--all` to emit every message regardless of rules (for debugging).

## Flags

| Flag | Description |
|------|-------------|
| (none) | 4-rule default filter -- emit only relevant messages |
| `--all` | Emit every message since last_seen. Mutually exclusive with `--mentions`. |
| `--mentions <name>` | Override the identity used for filtering. `@me` resolves to caller team. Kept for back-compat. Mutually exclusive with `--all`. |
| `--interval <s>` | Poll interval in seconds. Default 60. |
| `--terse` / `--human` | Output format override. |

## Instructions for mainthread

When this skill is invoked:

1. **Idempotency -- session-scope only** -- if you (this conversation's mainthread) already started a Monitor with `aw chat listen` earlier in THIS session, do NOT start another. Check your own tool-use history: if a `Monitor` call with `command: ".../aw chat listen ..."` appears earlier and was not followed by `TaskStop` on its task id, it is still running.

   Do NOT use `pgrep -f "aw chat listen"` for this check -- pgrep matches system-wide, including listeners from other Claude Code sessions on other WTs (agentic-workflow / mamba / conductor each run their own mainthread, each with its own listener). Each session manages its own Monitor independently; cross-session detection via pgrep produces false positives.

2. **Load the Monitor tool** if not already loaded:
   ```
   ToolSearch select:Monitor
   ```

3. **Start Monitor with `aw chat listen` as the streaming command** (only if step 1 confirmed no existing monitor):
   ```
   Monitor(
     description: "cross-WT chat channel -- new messages from /tmp/aw-channel.md",
     command: "aw chat listen 2>&1",
     persistent: true,
     timeout_ms: 3600000  // ignored when persistent=true; required by schema
   )
   ```
   `persistent: true` keeps it running for the session. Monitor itself runs the command and treats each stdout line as an event -- no separate bash process needed.

4. **Per-notification handling**:
   - Default emission is **Monitor-friendly listen-format**: each notification is one line of the form
     `msg-N | <from> -> <to> [re:M] [proj:P] | <ts> | $ aw chat read --re N --full`.
     The line ENDS with the canonical fetch command — the listener does NOT inline the message body.
   - To see the full body, run the suffix `aw chat read --re N --full` as a Bash command. This avoids inline-truncation when bodies span multiple lines or exceed the notification width.
   - The default filter already handles relevance -- every notification from `aw chat listen` (without `--all`) is one of the 4 categories above. After fetching the body, surface it to the user with a one-line action recommendation.
   - Do not auto-reply.
   - (Legacy: pass `--terse` or `--human` explicitly if you want the old body-inline format. Default = listen-format.)

5. **Stopping**:
   - `TaskStop` on the Monitor task id.

## Notes

- Listener state file: `~/.aw/chat-state.json` -- tracks `last_seen_msg_id` per team. Updated each iteration.
- If the listener crashes, restart by re-invoking this skill.
- Channel file: `/tmp/aw-channel.md` -- wiped on macOS reboot. Ephemeral by design.
- For ad-hoc inspection, use `aw chat read` or `aw chat list`. The listener is exclusively for Monitor wrapping; ad-hoc one-shot polling is intentionally not supported (`--once` was removed in G3 — it raced with Monitor over the shared state file).
