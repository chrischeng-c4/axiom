# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "TurtleScreen__bgcolor__color_as__Color_wrong"
# subject = "turtle.TurtleScreen.bgcolor(color: _Color)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed color"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed color
# mamba-strict-type: TypeError
"""Type wall: turtle.TurtleScreen.bgcolor(color: _Color); call it with the wrong type.

typeshed contract: color is _Color. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import TurtleScreen
obj = object.__new__(TurtleScreen)
try:
    obj.bgcolor(_W())  # color: _Color <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
