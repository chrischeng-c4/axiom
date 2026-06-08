# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "type"
# case = "prlimit__pid_as_int_wrong"
# subject = "resource.prlimit(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/resource.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: resource.prlimit(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from resource import prlimit
try:
    prlimit("not_an_int", 0)  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
