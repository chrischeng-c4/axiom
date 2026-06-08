# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____contains____frame_as_Frame_wrong"
# subject = "tracemalloc.Traceback.__contains__(frame: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__contains__(frame: Frame); call it with the wrong type.

typeshed contract: frame is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__contains__(_W())  # frame: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
