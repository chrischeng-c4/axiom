# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "TNavigator__sety__y_as_float_wrong"
# subject = "turtle.TNavigator.sety(y: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: turtle.TNavigator.sety(y: float); call it with the wrong type.

typeshed contract: y is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from turtle import TNavigator
obj = object.__new__(TNavigator)
try:
    obj.sety("not_a_float")  # y: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
