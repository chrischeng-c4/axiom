# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_sharedctypes"
# dimension = "type"
# case = "copy__obj_as__CT_wrong"
# subject = "multiprocessing.sharedctypes.copy(obj: _CT)"
# kind = "semantic"
# xfail = ""
# xfail = "force-typed arg enforcement pending; TypeVar param must stay unwalled (#955 regression fix) — needs a different probe design (#861)"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/sharedctypes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; TypeVar param must stay unwalled (#955 regression fix) — needs a different probe design (#861)
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.sharedctypes.copy(obj: _CT); call it with the wrong type.

typeshed contract: obj is _CT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.sharedctypes import copy
try:
    copy(_W())  # obj: _CT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
