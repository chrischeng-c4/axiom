# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pytree"
# dimension = "type"
# case = "Base__replace__new_as_typed_wrong"
# subject = "lib2to3.pytree.Base.replace(new: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed new"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pytree.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed new
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pytree.Base.replace(new: typed); call it with the wrong type.

typeshed contract: new is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pytree import Base
obj = object.__new__(Base)
try:
    obj.replace(_W())  # new: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
