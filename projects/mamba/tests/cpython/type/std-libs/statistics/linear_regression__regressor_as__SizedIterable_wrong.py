# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "type"
# case = "linear_regression__regressor_as__SizedIterable_wrong"
# subject = "statistics.linear_regression(regressor: _SizedIterable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed regressor"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/statistics.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed regressor
# mamba-strict-type: TypeError
"""Type wall: statistics.linear_regression(regressor: _SizedIterable); call it with the wrong type.

typeshed contract: regressor is _SizedIterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from statistics import linear_regression
try:
    linear_regression(_W(), None)  # regressor: _SizedIterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
