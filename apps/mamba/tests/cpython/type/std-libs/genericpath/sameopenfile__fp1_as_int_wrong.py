# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "type"
# case = "sameopenfile__fp1_as_int_wrong"
# subject = "genericpath.sameopenfile(fp1: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/genericpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: genericpath.sameopenfile(fp1: int); call it with the wrong type.

typeshed contract: fp1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from genericpath import sameopenfile
try:
    sameopenfile("not_an_int", 0)  # fp1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
