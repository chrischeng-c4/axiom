# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "BufferedWriter__truncate__pos_as_typed_wrong"
# subject = "_io.BufferedWriter.truncate(pos: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _io.BufferedWriter.truncate(pos: typed); call it with the wrong type.

typeshed contract: pos is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import BufferedWriter
obj = object.__new__(BufferedWriter)
try:
    obj.truncate(_W())  # pos: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
