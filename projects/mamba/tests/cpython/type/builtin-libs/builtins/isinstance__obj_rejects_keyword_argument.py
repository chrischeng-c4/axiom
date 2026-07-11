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

The generated TypeSpec contract retains `/` as `ParamSpecKind::PosOnly`, and
its structured binder is authoritative for call shape even though runtime
builtin registration also exposes a `Ty::Fn`. The companion case
`isinstance__class_or_tuple_as__ClassInfo_wrong.py` covers the argument type
contract; this fixture independently locks positional-only binding."""

try:
    isinstance(obj=1, class_or_tuple=int)  # isinstance is positional-only
    print("no_typeerror:")  # CPython accepted the keyword args; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
