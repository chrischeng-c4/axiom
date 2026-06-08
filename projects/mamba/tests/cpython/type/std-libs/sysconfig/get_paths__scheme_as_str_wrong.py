# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sysconfig"
# dimension = "type"
# case = "get_paths__scheme_as_str_wrong"
# subject = "sysconfig.get_paths(scheme: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sysconfig.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sysconfig.get_paths(scheme: str); call it with the wrong type.

typeshed contract: scheme is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sysconfig import get_paths
try:
    get_paths(12345)  # scheme: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
