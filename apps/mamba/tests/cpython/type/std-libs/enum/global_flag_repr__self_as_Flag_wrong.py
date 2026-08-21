# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "global_flag_repr__self_as_Flag_wrong"
# subject = "enum.global_flag_repr(self: Flag)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.global_flag_repr(self: Flag); call it with the wrong type.

typeshed contract: self is Flag. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import global_flag_repr
try:
    global_flag_repr(_W())  # self: Flag <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
