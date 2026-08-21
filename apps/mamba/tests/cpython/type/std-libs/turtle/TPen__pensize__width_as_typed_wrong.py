# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "TPen__pensize__width_as_typed_wrong"
# subject = "turtle.TPen.pensize(width: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed width"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed width
# mamba-strict-type: TypeError
"""Type wall: turtle.TPen.pensize(width: typed); call it with the wrong type.

typeshed contract: width is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import TPen
obj = object.__new__(TPen)
try:
    obj.pensize(_W())  # width: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
