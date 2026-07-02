# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "chr__i_rejects_keyword_argument"
# subject = "builtins.chr(i: SupportsIndex, /)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.chr(i, /) is positional-only in CPython — calling it
with a keyword, `chr(i=65)`, raises `TypeError: chr() takes no keyword
arguments` even though `65` is a perfectly well-typed `SupportsIndex` value.

This is the #924 gap: `chr`/`ord`/`getattr`/`hasattr`/`setattr`/`format`/
`isinstance`/`issubclass` are dual-registered through `def_builtin`'s general
Ty::Fn call-checking mechanism (src/types/check_expr.rs), a separate path from
`check_stdlib_call`/StdlibSig. That path had no concept of "positional-only" —
a well-typed keyword call fell through uncaught to the runtime dispatch, which
packed the kwargs dict into the positional argument slot and raised an
unrelated `TypeError: 'dict' object cannot be interpreted as an integer`
instead of CPython's clean message. The companion case
`chr__i_as_SupportsIndex_wrong.py` covers the pre-existing wrong-TYPE wall;
this one covers the wrong-CALL-SHAPE wall (#881 landed the equivalent for the
StdlibSig path; this is its def_builtin/Ty::Fn twin)."""

try:
    chr(i=65)  # chr is positional-only; ANY keyword arg is rejected
    print("no_typeerror:")  # CPython accepted the keyword arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
