"""Render a Jinja2 template from a string source with autoescaping.

End-user scenario: a downstream web/templating tool creates an
`Environment`, compiles one template from a literal string, and
renders it with one variable. Two legs are asserted because Jinja2's
HTML autoescape is the security boundary tools rely on:

  1. A plain variable renders to its `str()` form.
  2. A variable containing HTML metacharacters is escaped (`<` → `&lt;`).

DoD: this script must exit 0 under both CPython and mamba.
"""

from jinja2 import Environment

# For from_string templates Jinja2 defaults autoescape to off, so we
# pass autoescape=True explicitly to make the escape leg deterministic
# regardless of select_autoescape() heuristics (which key off template
# names that don't exist here).
env = Environment(autoescape=True)
template = env.from_string("hello, {{ name }}!")

# Plain text — `world` has no metacharacters, comes through unchanged.
plain = template.render(name="world")
assert plain == "hello, world!", f"unexpected plain render: {plain!r}"

# Escape leg — `<script>` proves autoescape fires.
escaped = template.render(name="<script>")
assert escaped == "hello, &lt;script&gt;!", f"unexpected escaped render: {escaped!r}"

print("ok:", plain)
