# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "create__unboundop_as_Literal_wrong"
# subject = "_interpchannels.create(unboundop: Literal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.create(unboundop: Literal); call it with the wrong type.

typeshed contract: unboundop is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import create
try:
    create(_W())  # unboundop: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
