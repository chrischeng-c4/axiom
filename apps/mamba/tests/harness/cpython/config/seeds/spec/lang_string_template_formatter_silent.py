# Operational AssertionPass seed for SILENT divergences in `string`
# module helpers — `string.printable` exhaustive content, capwords
# with a custom separator, `string.Template` substitution, and
# `string.Formatter().format` positional substitution. The matching
# subset (character-class constants + whitespace-separator capwords)
# is covered by `test_string_constants_capwords_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • string.printable — should be the 100-char union of digits +
#     letters + punctuation + whitespace (mamba returns empty);
#   • string.capwords(s, sep) — should re-split on sep, capitalize
#     each piece, and join with sep (mamba ignores the sep argument);
#   • string.Template(s) — should return a Template instance whose
#     `.substitute(**kwargs)` returns the interpolated string (mamba
#     returns an empty dict, so `.substitute` raises AttributeError);
#   • string.Template.safe_substitute — leaves missing keys as `$key`
#     in the output (mamba: AttributeError);
#   • string.Formatter().format("{0} and {1}", a, b) — positional
#     substitution (mamba returns None even though the kwargs form
#     produces a string).
import string
from typing import Any

_ledger: list[int] = []

# 1) string.printable — CPython spec: 100 chars (digits + letters +
#    punctuation + whitespace, in that documented order)
assert isinstance(string.printable, str); _ledger.append(1)
assert len(string.printable) == 100; _ledger.append(1)
# Letters / digits / common punctuation are in there
assert "a" in string.printable; _ledger.append(1)
assert "Z" in string.printable; _ledger.append(1)
assert "0" in string.printable; _ledger.append(1)
assert "!" in string.printable; _ledger.append(1)
assert " " in string.printable; _ledger.append(1)
# NUL byte is NOT printable
assert "\x00" not in string.printable; _ledger.append(1)

# 2) capwords with a custom sep — splits on the sep, capitalizes each
#    piece, joins back with the sep
_cap_h: Any = string.capwords("hello-world-foo", "-")
assert _cap_h == "Hello-World-Foo"; _ledger.append(1)
_cap_d: Any = string.capwords("alpha.beta.gamma", ".")
assert _cap_d == "Alpha.Beta.Gamma"; _ledger.append(1)
_cap_c: Any = string.capwords("a:b:c", ":")
assert _cap_c == "A:B:C"; _ledger.append(1)
# Mixed case in the input is re-cased
_cap_m: Any = string.capwords("HELLO-WORLD", "-")
assert _cap_m == "Hello-World"; _ledger.append(1)
# Single-element input still capitalizes
_cap_s: Any = string.capwords("hello", "-")
assert _cap_s == "Hello"; _ledger.append(1)

# 3) string.Template(...) — CPython spec: returns a Template instance
_t: Any = string.Template("Hello $name!")
# It is NOT a plain dict
assert not isinstance(_t, dict); _ledger.append(1)
# substitute with kwargs
_r1: Any = _t.substitute(name="World")
assert _r1 == "Hello World!"; _ledger.append(1)
# substitute with a mapping
_r2: Any = _t.substitute({"name": "Claude"})
assert _r2 == "Hello Claude!"; _ledger.append(1)
# Multi-placeholder
_t_multi: Any = string.Template("$a + $b = $c")
_r_multi: Any = _t_multi.substitute(a=1, b=2, c=3)
assert _r_multi == "1 + 2 = 3"; _ledger.append(1)
# Identifier with ${...} braces
_t_brace: Any = string.Template("${greeting}, ${name}")
_r_brace: Any = _t_brace.substitute(greeting="Hi", name="there")
assert _r_brace == "Hi, there"; _ledger.append(1)

# 4) Template.safe_substitute — leaves missing keys as $key in output
_t_safe: Any = string.Template("$missing")
_r_safe: Any = _t_safe.safe_substitute()
assert _r_safe == "$missing"; _ledger.append(1)
_t_safe2: Any = string.Template("$a $b")
_r_safe2: Any = _t_safe2.safe_substitute(a="hi")
assert _r_safe2 == "hi $b"; _ledger.append(1)

# 5) string.Formatter().format with POSITIONAL args
_fmt: Any = string.Formatter()
_pos: Any = _fmt.format("{0} and {1}", "a", "b")
assert _pos == "a and b"; _ledger.append(1)
_pos2: Any = _fmt.format("{0} {1} {2}", "x", "y", "z")
assert _pos2 == "x y z"; _ledger.append(1)
# Mixed positional + kwargs
_mix: Any = _fmt.format("{0} = {value}", "key", value=42)
assert _mix == "key = 42"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_template_formatter_silent {sum(_ledger)} asserts")
