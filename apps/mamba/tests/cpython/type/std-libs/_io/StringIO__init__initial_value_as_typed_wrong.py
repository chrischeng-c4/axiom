# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "StringIO__init__initial_value_as_typed_wrong"
# subject = "_io.StringIO.__init__(initial_value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _io.StringIO.__init__(initial_value: typed); call it with the wrong type.

typeshed contract: initial_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import StringIO
try:
    StringIO(_W())  # initial_value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
