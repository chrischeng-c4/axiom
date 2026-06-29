# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryReader__replay__progress_callback_as_typed_wrong"
# subject = "_remote_debugging.BinaryReader.replay(progress_callback: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryReader.replay(progress_callback: typed); call it with the wrong type.

typeshed contract: progress_callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _remote_debugging import BinaryReader
obj = object.__new__(BinaryReader)
try:
    obj.replay(None, _W())  # progress_callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
