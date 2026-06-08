# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_return__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.set_return(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_return(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_return(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
