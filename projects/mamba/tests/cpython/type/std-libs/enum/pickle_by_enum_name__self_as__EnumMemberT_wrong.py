# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "pickle_by_enum_name__self_as__EnumMemberT_wrong"
# subject = "enum.pickle_by_enum_name(self: _EnumMemberT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed self"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed self
# mamba-strict-type: TypeError
"""Type wall: enum.pickle_by_enum_name(self: _EnumMemberT); call it with the wrong type.

typeshed contract: self is _EnumMemberT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import pickle_by_enum_name
try:
    pickle_by_enum_name(_W(), 0)  # self: _EnumMemberT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
