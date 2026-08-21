# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "CoverageResults__write_results_file__path_as_StrPath_wrong"
# subject = "trace.CoverageResults.write_results_file(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.CoverageResults.write_results_file(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import CoverageResults
obj = object.__new__(CoverageResults)
try:
    obj.write_results_file(_W(), None, None, None)  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
