# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "get_type_hints__obj_as__get_type_hints_obj_allowed_types_wrong"
# subject = "typing.get_type_hints(obj: _get_type_hints_obj_allowed_types)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj
# mamba-strict-type: TypeError
"""Type wall: typing.get_type_hints(obj: _get_type_hints_obj_allowed_types); call it with the wrong type.

typeshed contract: obj is _get_type_hints_obj_allowed_types. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import get_type_hints
try:
    get_type_hints(_W())  # obj: _get_type_hints_obj_allowed_types <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
