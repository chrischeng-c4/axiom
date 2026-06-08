# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "Shape__addcomponent__poly_as__PolygonCoords_wrong"
# subject = "turtle.Shape.addcomponent(poly: _PolygonCoords)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: turtle.Shape.addcomponent(poly: _PolygonCoords); call it with the wrong type.

typeshed contract: poly is _PolygonCoords. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import Shape
obj = object.__new__(Shape)
try:
    obj.addcomponent(_W(), None)  # poly: _PolygonCoords <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
