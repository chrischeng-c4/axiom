# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "TNavigator__setx__x_as_float_wrong"
# subject = "turtle.TNavigator.setx(x: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: turtle.TNavigator.setx(x: float); call it with the wrong type.

typeshed contract: x is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from turtle import TNavigator
obj = object.__new__(TNavigator)
try:
    obj.setx("not_a_float")  # x: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
