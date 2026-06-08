# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__print_line__func_as_str_wrong"
# subject = "pstats.Stats.print_line(func: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.print_line(func: str); call it with the wrong type.

typeshed contract: func is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.print_line(12345)  # func: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
