# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "type"
# case = "set_coroutine_origin_tracking_depth__depth_as_int_wrong"
# subject = "sys.set_coroutine_origin_tracking_depth(depth: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys.set_coroutine_origin_tracking_depth(depth: int); call it with the wrong type.

typeshed contract: depth is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys import set_coroutine_origin_tracking_depth
try:
    set_coroutine_origin_tracking_depth("not_an_int")  # depth: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
