# Operational AssertionPass seed for SILENT divergences across the
# re module-qualified Match / Pattern type-name contract +
# re.IGNORECASE RegexFlag enum-repr contract + string.printable
# sentinel identifier + textwrap.TextWrapper class identifier
# pinned by atomic 190: `re` (the documented
# `type(re.match(...)).__name__ == "Match"` / `type(re.compile
# (...)).__name__ == "Pattern"` unqualified class-identity
# contract — CPython uses the bare class name, never module-
# qualified, in __name__ + the documented `str(re.IGNORECASE)
# == "re.IGNORECASE"` RegexFlag enum repr contract — re's
# flags are an IntFlag enum whose str() form is the qualified
# name, not the underlying int), `string` (the documented
# `printable` sentinel identifier — the union of digits,
# ascii_letters, punctuation, and whitespace), and `textwrap`
# (the documented `TextWrapper` class identifier surface).
#
# The matching subset (full re module hasattr + match /
# search / findall / sub / split / escape / IGNORECASE
# integer value, partial string module hasattr + value, partial
# textwrap module hasattr + value, full keyword module hasattr
# + value) is covered by
# `test_re_string_textwrap_keyword_value_ops`; this fixture
# pins the CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(re.match(r"\w+", "abc")).__name__ == "Match" —
#     documented unqualified class identity (mamba: returns
#     "re.Match" — the module-qualified name leaks into
#     __name__);
#   • type(re.compile(r"\d+")).__name__ == "Pattern" —
#     documented unqualified class identity (mamba: returns
#     "re.Pattern" — same module-qualified name leak);
#   • str(re.IGNORECASE) == "re.IGNORECASE" — documented
#     RegexFlag enum repr (mamba: returns "2" — the
#     IntFlag enum repr layer is missing, the raw integer
#     leaks through);
#   • hasattr(string, "printable") is True — documented
#     sentinel identifier (mamba: False — the printable
#     union is not exposed at module level);
#   • hasattr(textwrap, "TextWrapper") is True — documented
#     class identifier (mamba: False — the TextWrapper
#     class is not exposed at module level).
import re as _re_mod
import string as _string_mod
import textwrap as _textwrap_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance-method / value-contract behavior
# that mamba's bundled type stubs do not surface accurately.
re: Any = _re_mod
string: Any = _string_mod
textwrap: Any = _textwrap_mod


_ledger: list[int] = []

# 1) re.match — unqualified class-identity contract
_m = re.match(r"\w+", "abc")
assert _m is not None; _ledger.append(1)
assert type(_m).__name__ == "Match"; _ledger.append(1)

# 2) re.compile — unqualified class-identity contract
_p = re.compile(r"\d+")
assert type(_p).__name__ == "Pattern"; _ledger.append(1)

# 3) re.IGNORECASE — RegexFlag enum repr contract
assert str(re.IGNORECASE) == "re.IGNORECASE"; _ledger.append(1)

# 4) string.printable — sentinel identifier surface
assert hasattr(string, "printable") == True; _ledger.append(1)

# 5) textwrap.TextWrapper — class identifier surface
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_re_string_textwrap_silent {sum(_ledger)} asserts")
