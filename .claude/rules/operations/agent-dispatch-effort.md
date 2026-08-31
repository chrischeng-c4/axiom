# Select Agent effort before dispatch

The main session must select the effort before every `Agent` call.

- Use one of `low`, `medium`, `high`, `xhigh`, or `max`.
- Start the Agent description with `[effort=<level>]`.
- Select a project Agent whose frontmatter `effort:` has the same value.
- Do not use a built-in or unregistered Agent. Its effective effort cannot be
  checked by the repository hook.
- If no project Agent has the selected effort and correct ownership, keep the
  work in the main session or report the routing gap. Do not claim another
  effort only to pass the hook.

Example:

```text
description: [effort=high] Review the public API change
subagent_type: server-lifecycle-sr-dev
```

`.claude/hooks/require_agent_effort.py` runs before the call. It rejects a
missing marker, an unknown value, an unregistered Agent, or a value that does
not match the Agent definition.
