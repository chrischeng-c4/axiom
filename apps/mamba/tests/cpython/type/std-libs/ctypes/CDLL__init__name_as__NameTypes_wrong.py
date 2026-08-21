# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "CDLL__init__name_as__NameTypes_wrong"
# subject = "ctypes.CDLL.__init__(name: _NameTypes)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.CDLL.__init__(name: _NameTypes); call it with the wrong type.

typeshed contract: name is _NameTypes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import CDLL
try:
    CDLL(_W())  # name: _NameTypes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
