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

The generated TypeSpec contract retains `/` as `ParamSpecKind::PosOnly`, and
its structured binder is authoritative for call shape even though runtime
builtin registration also exposes a `Ty::Fn`. The companion case
`chr__i_as_SupportsIndex_wrong.py` covers the argument type contract; this
fixture independently locks positional-only binding."""

try:
    chr(i=65)  # chr is positional-only; ANY keyword arg is rejected
    print("no_typeerror:")  # CPython accepted the keyword arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
