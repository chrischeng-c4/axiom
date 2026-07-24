# FocusFlow Python UI TD

`src/interface/todo_ui.py` is an authoring-only UI tech design. It is ordinary
Python syntax, but the `@page`, `@component`, `Event`, `Slot`, and `token`
names are compiler vocabulary — the module is parsed, never imported or run.

The first vertical slice proves that a concise Python component tree can lower
to the existing Wireframe, Component, and Design Token IR contracts without
writing the previous YAML forms by hand.
