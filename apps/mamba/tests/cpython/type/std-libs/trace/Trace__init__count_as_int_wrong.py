# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__init__count_as_int_wrong"
# subject = "trace.Trace.__init__(count: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.__init__(count: int); call it with the wrong type.

typeshed contract: count is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from trace import Trace
try:
    Trace("not_an_int")  # count: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
