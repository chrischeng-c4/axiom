# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "type"
# case = "ParserCreate__encoding_as_typed_wrong"
# subject = "pyexpat.ParserCreate(encoding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyexpat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyexpat.ParserCreate(encoding: typed); call it with the wrong type.

typeshed contract: encoding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pyexpat import ParserCreate
try:
    ParserCreate(_W())  # encoding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
