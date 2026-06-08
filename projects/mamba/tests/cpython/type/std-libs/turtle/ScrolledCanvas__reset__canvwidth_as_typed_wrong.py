# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "type"
# case = "ScrolledCanvas__reset__canvwidth_as_typed_wrong"
# subject = "turtle.ScrolledCanvas.reset(canvwidth: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/turtle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: turtle.ScrolledCanvas.reset(canvwidth: typed); call it with the wrong type.

typeshed contract: canvwidth is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from turtle import ScrolledCanvas
obj = object.__new__(ScrolledCanvas)
try:
    obj.reset(_W())  # canvwidth: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
