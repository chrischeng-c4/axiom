# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "RawTurtle__shapesize__stretch_wid_as_typed_wrong"
# subject = "turtle.RawTurtle.shapesize(stretch_wid: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stretch_wid"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed stretch_wid
# mamba-strict-type: TypeError
"""Type wall: turtle.RawTurtle.shapesize(stretch_wid: typed); call it with the wrong type.

typeshed contract: stretch_wid is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import RawTurtle
obj = object.__new__(RawTurtle)
try:
    obj.shapesize(_W())  # stretch_wid: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
