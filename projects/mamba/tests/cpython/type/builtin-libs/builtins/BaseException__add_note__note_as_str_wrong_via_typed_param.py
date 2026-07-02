# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__add_note__note_as_str_wrong_via_typed_param"
# subject = "builtins.BaseException.add_note(note: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.add_note(note: str); call it with the wrong type.

Receiver-resolution variant (#886): the receiver here is a function
*parameter* annotated `e: BaseException`, never assigned via a direct
`x = Cls(...)` construction that `instance_origins` tracks. The Method wall
must still fire by falling back to the parameter's inferred `Ty::Class` name
(from resolving the `BaseException` annotation), not the assignment-based
instance_origins map.

typeshed contract: note is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise -- mamba's to
enforce)."""


def annotate(e: BaseException):
    e.add_note(12345)  # note: str <- wrong-typed


try:
    annotate(BaseException("boom"))
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as exc:
    print("typeerror:", type(exc).__name__)
except Exception as exc:
    print("setup_or_other:", type(exc).__name__)
