# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "CoverageResults__write_results__show_missing_as_bool_wrong"
# subject = "trace.CoverageResults.write_results(show_missing: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed show_missing"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed show_missing
# mamba-strict-type: TypeError
"""Type wall: trace.CoverageResults.write_results(show_missing: bool); call it with the wrong type.

typeshed contract: show_missing is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from trace import CoverageResults
obj = object.__new__(CoverageResults)
try:
    obj.write_results("not_a_bool")  # show_missing: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
