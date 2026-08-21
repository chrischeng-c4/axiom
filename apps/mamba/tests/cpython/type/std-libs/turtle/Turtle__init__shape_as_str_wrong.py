# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "Turtle__init__shape_as_str_wrong"
# subject = "turtle.Turtle.__init__(shape: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: turtle.Turtle.__init__(shape: str); call it with the wrong type.

typeshed contract: shape is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from turtle import Turtle
try:
    Turtle(12345)  # shape: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
