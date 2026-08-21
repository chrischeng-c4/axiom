# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_cmath_reprlib_html_quopri_value_ops"
# subject = "cpython321.test_cmath_reprlib_html_quopri_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_cmath_reprlib_html_quopri_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_cmath_reprlib_html_quopri_value_ops: execute CPython 3.12 seed test_cmath_reprlib_html_quopri_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `cmath` / `reprlib` / `html` / `quopri` four-pack pinned to
# atomic 215: `cmath` (the documented full module-level
# helper / sentinel identifier hasattr surface — `sqrt` /
# `sin` / `cos` / `tan` / `asin` / `acos` / `atan` / `sinh`
# / `cosh` / `tanh` / `exp` / `log` / `log10` / `phase` /
# `polar` / `rect` / `isfinite` / `isinf` / `isnan` /
# `isclose` / `pi` / `e` / `tau` / `inf` / `infj` / `nan` /
# `nanj` + the documented `cmath.sqrt(-1).imag == 1.0` /
# `cmath.phase(complex(3, 4)) > 0` /
# `len(cmath.polar(complex(3, 4))) == 2` /
# `cmath.polar(complex(3, 4))[0] == 5.0` /
# `type(cmath.pi).__name__ == "float"` /
# `cmath.pi > 3 and cmath.pi < 4` /
# `cmath.isfinite(complex(1, 2)) == True` /
# `cmath.isinf(complex(float("inf"), 0)) == True` /
# `type(cmath.infj).__name__ == "complex"`
# complex-arithmetic value contract), `reprlib` (the
# documented partial module-level helper / class
# identifier hasattr surface — `Repr` / `repr` /
# `recursive_repr` + the documented
# `reprlib.repr([1, 2, 3]) == "[1, 2, 3]"` short-repr
# value contract), `html` (the documented partial module-
# level helper identifier hasattr surface — `escape` /
# `unescape` + the documented
# `html.escape("<a&b>") == "&lt;a&amp;b&gt;"` /
# `html.escape('"x"') == "&quot;x&quot;"` /
# `html.unescape("&lt;a&gt;") == "<a>"` /
# `html.unescape("&amp;") == "&"` html-codec value
# contract), and `quopri` (the documented full module-
# level helper identifier hasattr surface — `encodestring`
# / `decodestring` / `encode` / `decode` + the documented
# `quopri.encodestring(b"hello=") == b"hello=3D"` /
# `quopri.decodestring(b"hello=3D") == b"hello="`
# quoted-printable codec value contract).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(reprlib, "aRepr") False on mamba +
# type(reprlib.aRepr).__name__ == "Repr" collapses to
# "NoneType" on mamba + reprlib.aRepr.maxstring type "int"
# collapses to "NoneType" on mamba, hasattr(copyreg,
# "pickle") / "constructor" / "dispatch_table" /
# "__newobj__" / "__newobj_ex__" all False on mamba +
# type(copyreg.dispatch_table).__name__ == "dict"
# collapses to "NoneType" on mamba, hasattr(html,
# "entities") False on mamba +
# html.unescape("&#34;x&#34;") == '"x"' numeric-entity
# decoding collapses to literal '&#34;x&#34;' on mamba)
# are covered in the matching spec fixture
# `lang_reprlib_copyreg_html_silent`.
import cmath
import reprlib
import html
import quopri


_ledger: list[int] = []

# 1) cmath — full module hasattr surface
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "sin") == True; _ledger.append(1)
assert hasattr(cmath, "cos") == True; _ledger.append(1)
assert hasattr(cmath, "tan") == True; _ledger.append(1)
assert hasattr(cmath, "asin") == True; _ledger.append(1)
assert hasattr(cmath, "acos") == True; _ledger.append(1)
assert hasattr(cmath, "atan") == True; _ledger.append(1)
assert hasattr(cmath, "sinh") == True; _ledger.append(1)
assert hasattr(cmath, "cosh") == True; _ledger.append(1)
assert hasattr(cmath, "tanh") == True; _ledger.append(1)
assert hasattr(cmath, "exp") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "log10") == True; _ledger.append(1)
assert hasattr(cmath, "phase") == True; _ledger.append(1)
assert hasattr(cmath, "polar") == True; _ledger.append(1)
assert hasattr(cmath, "rect") == True; _ledger.append(1)
assert hasattr(cmath, "isfinite") == True; _ledger.append(1)
assert hasattr(cmath, "isinf") == True; _ledger.append(1)
assert hasattr(cmath, "isnan") == True; _ledger.append(1)
assert hasattr(cmath, "isclose") == True; _ledger.append(1)
assert hasattr(cmath, "pi") == True; _ledger.append(1)
assert hasattr(cmath, "e") == True; _ledger.append(1)
assert hasattr(cmath, "tau") == True; _ledger.append(1)
assert hasattr(cmath, "inf") == True; _ledger.append(1)
assert hasattr(cmath, "infj") == True; _ledger.append(1)
assert hasattr(cmath, "nan") == True; _ledger.append(1)
assert hasattr(cmath, "nanj") == True; _ledger.append(1)

# 2) cmath — complex-arithmetic value contract
assert cmath.sqrt(-1).imag == 1.0; _ledger.append(1)
_z = complex(3, 4)
assert cmath.phase(_z) > 0; _ledger.append(1)
assert len(cmath.polar(_z)) == 2; _ledger.append(1)
assert cmath.polar(_z)[0] == 5.0; _ledger.append(1)
assert type(cmath.pi).__name__ == "float"; _ledger.append(1)
assert cmath.pi > 3; _ledger.append(1)
assert cmath.pi < 4; _ledger.append(1)
assert cmath.isfinite(complex(1, 2)) == True; _ledger.append(1)
assert cmath.isinf(complex(float("inf"), 0)) == True; _ledger.append(1)
assert type(cmath.infj).__name__ == "complex"; _ledger.append(1)

# 3) reprlib — partial module hasattr surface
#    (aRepr DIVERGE on mamba — moved to spec)
assert hasattr(reprlib, "Repr") == True; _ledger.append(1)
assert hasattr(reprlib, "repr") == True; _ledger.append(1)
assert hasattr(reprlib, "recursive_repr") == True; _ledger.append(1)

# 4) reprlib — short-repr value contract
assert reprlib.repr([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)

# 5) html — partial module hasattr surface
#    (entities DIVERGE on mamba — moved to spec)
assert hasattr(html, "escape") == True; _ledger.append(1)
assert hasattr(html, "unescape") == True; _ledger.append(1)

# 6) html — html-codec value contract
assert html.escape("<a&b>") == "&lt;a&amp;b&gt;"; _ledger.append(1)
assert html.escape('"x"') == "&quot;x&quot;"; _ledger.append(1)
assert html.unescape("&lt;a&gt;") == "<a>"; _ledger.append(1)
assert html.unescape("&amp;") == "&"; _ledger.append(1)

# 7) quopri — full module hasattr surface
assert hasattr(quopri, "encodestring") == True; _ledger.append(1)
assert hasattr(quopri, "decodestring") == True; _ledger.append(1)
assert hasattr(quopri, "encode") == True; _ledger.append(1)
assert hasattr(quopri, "decode") == True; _ledger.append(1)

# 8) quopri — quoted-printable codec value contract
assert quopri.encodestring(b"hello=") == b"hello=3D"; _ledger.append(1)
assert quopri.decodestring(b"hello=3D") == b"hello="; _ledger.append(1)

# NB: hasattr(reprlib, "aRepr") False on mamba +
# type(reprlib.aRepr).__name__ == "Repr" collapses to
# "NoneType" on mamba + reprlib.aRepr.maxstring type "int"
# collapses to "NoneType" on mamba, hasattr(copyreg,
# "pickle") / "constructor" / "dispatch_table" /
# "__newobj__" / "__newobj_ex__" all False on mamba +
# type(copyreg.dispatch_table).__name__ == "dict"
# collapses to "NoneType" on mamba, hasattr(html,
# "entities") False on mamba +
# html.unescape("&#34;x&#34;") == '"x"' numeric-entity
# decoding collapses to literal '&#34;x&#34;' on mamba —
# all DIVERGE on mamba — moved to the divergence-spec
# fixture.

print(f"MAMBA_ASSERTION_PASS: test_cmath_reprlib_html_quopri_value_ops {sum(_ledger)} asserts")
