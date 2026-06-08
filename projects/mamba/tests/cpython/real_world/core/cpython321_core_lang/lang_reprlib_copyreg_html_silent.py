# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_reprlib_copyreg_html_silent"
# subject = "cpython321.lang_reprlib_copyreg_html_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_reprlib_copyreg_html_silent.py"
# status = "filled"
# ///
"""cpython321.lang_reprlib_copyreg_html_silent: execute CPython 3.12 seed lang_reprlib_copyreg_html_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the `reprlib` /
# `copyreg` / `html` three-pack pinned to atomic 215:
# `reprlib` (the documented `hasattr(reprlib, "aRepr") ==
# True` module-level sentinel hasattr surface + the
# documented `type(reprlib.aRepr).__name__ == "Repr"`
# module-singleton type-identity value contract + the
# documented
# `type(reprlib.aRepr.maxstring).__name__ == "int"`
# sentinel-attribute type-identity value contract),
# `copyreg` (the documented `hasattr(copyreg, "pickle") /
# "constructor" / "dispatch_table" / "__newobj__" /
# "__newobj_ex__" == True` full module-level helper /
# sentinel identifier hasattr surface + the documented
# `type(copyreg.dispatch_table).__name__ == "dict"`
# dispatch-table type-identity value contract), and
# `html` (the documented `hasattr(html, "entities") ==
# True` sub-module hasattr surface + the documented
# `html.unescape("&#34;x&#34;") == '"x"'` numeric-
# entity-decoding value contract).
#
# Behavioral edges that CONFORM on mamba
# (reprlib `Repr` / `repr` / `recursive_repr` hasattr
# surface + `reprlib.repr([1, 2, 3]) == "[1, 2, 3]"`
# short-repr value contract, html `escape` / `unescape`
# hasattr surface + `html.escape("<a&b>") ==
# "&lt;a&amp;b&gt;"` / `html.escape('"x"') ==
# "&quot;x&quot;"` / `html.unescape("&lt;a&gt;") ==
# "<a>"` / `html.unescape("&amp;") == "&"` named-entity
# html-codec value contract) are covered in the matching
# pass fixture `test_cmath_reprlib_html_quopri_value_ops`.
from typing import Any
import reprlib as _reprlib_mod
import copyreg as _copyreg_mod
import html as _html_mod

reprlib: Any = _reprlib_mod
copyreg: Any = _copyreg_mod
html: Any = _html_mod


_ledger: list[int] = []

# 1) reprlib — module-level sentinel hasattr surface
#    (mamba: aRepr False)
assert hasattr(reprlib, "aRepr") == True; _ledger.append(1)

# 2) reprlib — module-singleton type-identity value contract
#    (mamba: type(reprlib.aRepr).__name__ collapses to
#    "NoneType")
assert type(reprlib.aRepr).__name__ == "Repr"; _ledger.append(1)

# 3) reprlib — sentinel-attribute type-identity value contract
#    (mamba: reprlib.aRepr.maxstring collapses to None, so
#    its type is "NoneType")
assert type(reprlib.aRepr.maxstring).__name__ == "int"; _ledger.append(1)

# 4) copyreg — full module-level helper / sentinel identifier
#    hasattr surface
#    (mamba: pickle / constructor / dispatch_table /
#    __newobj__ / __newobj_ex__ all False)
assert hasattr(copyreg, "pickle") == True; _ledger.append(1)
assert hasattr(copyreg, "constructor") == True; _ledger.append(1)
assert hasattr(copyreg, "dispatch_table") == True; _ledger.append(1)
assert hasattr(copyreg, "__newobj__") == True; _ledger.append(1)
assert hasattr(copyreg, "__newobj_ex__") == True; _ledger.append(1)

# 5) copyreg — dispatch-table type-identity value contract
#    (mamba: type(copyreg.dispatch_table).__name__ collapses
#    to "NoneType")
assert type(copyreg.dispatch_table).__name__ == "dict"; _ledger.append(1)

# 6) html — sub-module hasattr surface
#    (mamba: entities False)
assert hasattr(html, "entities") == True; _ledger.append(1)

# 7) html — numeric-entity-decoding value contract
#    (mamba: html.unescape leaves "&#34;x&#34;" literal
#    in the output instead of decoding to '"x"')
assert html.unescape("&#34;x&#34;") == '"x"'; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_reprlib_copyreg_html_silent {sum(_ledger)} asserts")
