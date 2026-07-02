# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "isinstance__obj_rejects_keyword_argument"
# subject = "builtins.isinstance(obj, class_or_tuple, /)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.isinstance(obj, class_or_tuple, /) is positional-only
in CPython — calling it with keywords, `isinstance(obj=1, class_or_tuple=int)`,
raises `TypeError: isinstance() takes no keyword arguments` even though both
values are well-typed.

This is the #924 gap: `isinstance`/`issubclass`/`chr`/`ord`/`getattr`/
`hasattr`/`setattr`/`format` are dual-registered through `def_builtin`'s
general Ty::Fn call-checking mechanism (src/types/check_expr.rs), a separate
path from `check_stdlib_call`/StdlibSig. That path had no concept of
"positional-only" — a well-typed keyword call fell through uncaught to the
runtime dispatch instead of raising CPython's clean message. The companion
case `isinstance__class_or_tuple_as__ClassInfo_wrong.py` covers the
pre-existing wrong-TYPE wall; this one covers the wrong-CALL-SHAPE wall (#881
landed the equivalent for the StdlibSig path; this is its def_builtin/Ty::Fn
twin)."""

try:
    isinstance(obj=1, class_or_tuple=int)  # isinstance is positional-only
    print("no_typeerror:")  # CPython accepted the keyword args; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
