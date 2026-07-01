# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "type"
# case = "testsource__module_as_typed_wrong"
# subject = "doctest.testsource(module: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/doctest.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: doctest.testsource(module: typed); call it with the wrong type.

typeshed contract: module is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from doctest import testsource
try:
    testsource(_W(), "")  # module: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
