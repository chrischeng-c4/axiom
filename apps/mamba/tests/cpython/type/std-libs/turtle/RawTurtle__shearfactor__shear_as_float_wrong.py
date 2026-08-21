# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "RawTurtle__shearfactor__shear_as_float_wrong"
# subject = "turtle.RawTurtle.shearfactor(shear: float)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed shear"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed shear
# mamba-strict-type: TypeError
"""Type wall: turtle.RawTurtle.shearfactor(shear: float); call it with the wrong type.

typeshed contract: shear is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from turtle import RawTurtle
obj = object.__new__(RawTurtle)
try:
    obj.shearfactor("not_a_float")  # shear: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
