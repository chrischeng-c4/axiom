# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "type"
# case = "samestat__s1_as_stat_result_wrong"
# subject = "genericpath.samestat(s1: stat_result)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/genericpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: genericpath.samestat(s1: stat_result); call it with the wrong type.

typeshed contract: s1 is stat_result. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from genericpath import samestat
try:
    samestat(_W(), None)  # s1: stat_result <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
