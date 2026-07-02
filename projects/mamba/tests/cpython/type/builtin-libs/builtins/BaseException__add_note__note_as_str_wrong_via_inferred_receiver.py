# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__add_note__note_as_str_wrong_via_inferred_receiver"
# subject = "builtins.BaseException.add_note(note: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.add_note(note: str); call it with the wrong type.

Receiver-resolution variant (#886): `BaseException` is a global builtin, so
`e = BaseException(...)` carries no `from builtins import BaseException`
import statement -- the `instance_origins` provenance map (populated only by
a direct construction through an *imported* class qualifier) never sees `e`.
The Method wall must still fire by falling back to `e`'s already-inferred
`Ty::Class` name. A second hop (`e2 = e`) exercises the same fallback through
an indirect assignment chain.

typeshed contract: note is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise -- mamba's to
enforce)."""

e = BaseException("boom")
e2 = e
try:
    e2.add_note(12345)  # note: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as exc:
    print("typeerror:", type(exc).__name__)
except Exception as exc:
    print("setup_or_other:", type(exc).__name__)
