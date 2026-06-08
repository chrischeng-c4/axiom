# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cProfile"
# dimension = "type"
# case = "label__code_as_typed_wrong"
# subject = "cProfile.label(code: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cProfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cProfile.label(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cProfile import label
try:
    label(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
