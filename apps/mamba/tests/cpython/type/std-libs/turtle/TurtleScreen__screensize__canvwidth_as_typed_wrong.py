# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "TurtleScreen__screensize__canvwidth_as_typed_wrong"
# subject = "turtle.TurtleScreen.screensize(canvwidth: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed canvwidth"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed canvwidth
# mamba-strict-type: TypeError
"""Type wall: turtle.TurtleScreen.screensize(canvwidth: typed); call it with the wrong type.

typeshed contract: canvwidth is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import TurtleScreen
obj = object.__new__(TurtleScreen)
try:
    obj.screensize(_W())  # canvwidth: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
