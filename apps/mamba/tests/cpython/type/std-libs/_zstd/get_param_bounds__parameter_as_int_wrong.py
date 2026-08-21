# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "get_param_bounds__parameter_as_int_wrong"
# subject = "_zstd.get_param_bounds(parameter: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.get_param_bounds(parameter: int); call it with the wrong type.

typeshed contract: parameter is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _zstd import get_param_bounds
try:
    get_param_bounds("not_an_int", True)  # parameter: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
