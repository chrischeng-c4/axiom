# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "gc"
# dimension = "type"
# case = "set_threshold__threshold0_as_int_wrong"
# subject = "gc.set_threshold(threshold0: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/gc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: gc.set_threshold(threshold0: int); call it with the wrong type.

typeshed contract: threshold0 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from gc import set_threshold
try:
    set_threshold("not_an_int")  # threshold0: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
