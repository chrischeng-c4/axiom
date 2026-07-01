# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "Dialect____new____dialect_as_typed_wrong"
# subject = "_csv.Dialect.__new__(dialect: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.Dialect.__new__(dialect: typed); call it with the wrong type.

typeshed contract: dialect is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import Dialect
obj = object.__new__(Dialect)
try:
    obj.__new__(_W())  # dialect: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
