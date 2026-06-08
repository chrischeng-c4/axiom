"""Offline MarkupSafe escape + Markup concatenation (Jinja2/Flask base).

End-user scenario: Jinja2 and Flask call `markupsafe.escape` on
every interpolated value and rely on `Markup` to mark pre-escaped
strings safe for HTML embedding. This fixture exercises the four
canonical behaviors a templating engine cares about:

  1. escape() returns a Markup that HTML-encodes the five
     dangerous characters (& < > " ').
  2. Markup(x) marks x as already-safe and is idempotent under
     escape() (double-escape would corrupt template output).
  3. Markup + str promotes the str via escape, preserving safety.
  4. Markup.format() / __mod__ escape interpolated arguments,
     not the template literal.

DoD: this script must exit 0 under both CPython and mamba.
"""

from markupsafe import Markup, escape

# -- 1. escape encodes the five HTML-dangerous characters -------------------

raw = '<script>alert("XSS & evil");</script>'
escaped = escape(raw)

# All five dangerous characters must be replaced.
assert "<" not in str(escaped), f"escape must remove '<', got {escaped!r}"
assert ">" not in str(escaped), f"escape must remove '>', got {escaped!r}"
assert '"' not in str(escaped), f"escape must remove '\"', got {escaped!r}"

# Specific entity expectations (the canonical Jinja2-compatible set).
expected = "&lt;script&gt;alert(&#34;XSS &amp; evil&#34;);&lt;/script&gt;"
assert str(escaped) == expected, (
    f"escape output mismatch:\n  got:      {str(escaped)!r}\n  expected: {expected!r}"
)

# escape's return type must be Markup (or compatible) so the templating
# engine knows the string is already safe.
assert isinstance(escaped, Markup), (
    f"escape() must return a Markup instance, got {type(escaped).__name__}"
)

# -- 2. Markup is idempotent under escape -----------------------------------

safe = Markup("<b>bold</b>")
twice = escape(safe)
# Re-escaping pre-marked-safe content must NOT double-encode.
assert str(twice) == "<b>bold</b>", (
    f"escape(Markup(x)) must be identity, got {str(twice)!r}"
)
assert isinstance(twice, Markup), (
    f"escape(Markup(...)) must stay Markup, got {type(twice).__name__}"
)

# -- 3. Markup + str promotes the str via escape ----------------------------

prefix = Markup("<b>hello</b> ")
suffix = "<script>nope</script>"  # str — must be escaped on concat
combined = prefix + suffix
assert isinstance(combined, Markup), (
    f"Markup + str must yield Markup, got {type(combined).__name__}"
)
assert str(combined) == "<b>hello</b> &lt;script&gt;nope&lt;/script&gt;", (
    f"Markup + str must escape the str half, got {str(combined)!r}"
)

# str + Markup must escape the str half as well (left side promotion).
left = "<i>plain</i>" + Markup("<b>safe</b>")
assert isinstance(left, Markup), (
    f"str + Markup must yield Markup, got {type(left).__name__}"
)
assert str(left) == "&lt;i&gt;plain&lt;/i&gt;<b>safe</b>", (
    f"str + Markup must escape the str half, got {str(left)!r}"
)

# -- 4. Markup.format escapes interpolated args, not the template ----------

template = Markup("<a href=\"{}\">{}</a>")
formatted = template.format("/users?q=<x>", "<click>")
assert isinstance(formatted, Markup), (
    f"Markup.format must return Markup, got {type(formatted).__name__}"
)
assert str(formatted) == '<a href="/users?q=&lt;x&gt;">&lt;click&gt;</a>', (
    f"Markup.format must escape args only, got {str(formatted)!r}"
)

# -- 5. % interpolation escapes args, not template (legacy Jinja path) -----

legacy = Markup("<p>%s</p>") % "<bad>"
assert isinstance(legacy, Markup), (
    f"Markup %% str must return Markup, got {type(legacy).__name__}"
)
assert str(legacy) == "<p>&lt;bad&gt;</p>", (
    f"Markup %% must escape the rhs, got {str(legacy)!r}"
)

print("ok: escape +", len(str(escaped)), "chars, concat +", len(str(combined)), "chars")
