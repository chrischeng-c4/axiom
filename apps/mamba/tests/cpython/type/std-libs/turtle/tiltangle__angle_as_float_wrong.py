# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "tiltangle__angle_as_float_wrong"
# subject = "turtle.tiltangle(angle: float)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed angle"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed angle
# mamba-strict-type: TypeError
"""Type wall: turtle.tiltangle(angle: float); call it with the wrong type.

typeshed contract: angle is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from turtle import tiltangle
try:
    tiltangle("not_a_float")  # angle: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
