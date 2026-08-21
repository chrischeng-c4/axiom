"""Render a rich markup string to plain text via `Console(record=True)`.

End-user scenario: a downstream CLI uses rich to format colored /
styled output for human users, but needs the same content captured
as deterministic plain text for logging and tests. The fixture
proves the recording path produces stable text across terminals —
no ANSI escapes, no width inference from `$COLUMNS`.

DoD: this script must exit 0 under both CPython and mamba.
"""

import io

from rich.console import Console

# Force a deterministic environment: explicit non-terminal output
# (`file=io.StringIO()` plus `force_terminal=False`), fixed width, and
# disabled color so the captured plain text cannot drift between hosts.
console = Console(
    file=io.StringIO(),
    force_terminal=False,
    color_system=None,
    width=40,
    record=True,
)

console.print("[bold]hello, rich[/bold]")
plain = console.export_text()

# `export_text` strips markup and ANSI; the trailing newline comes from
# `console.print`'s soft-wrap behavior.
assert plain == "hello, rich\n", f"unexpected rich plain text: {plain!r}"

print("ok:", plain.strip())
