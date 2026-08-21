# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "gmtime__seconds_as_typed_wrong"
# subject = "time.gmtime(seconds: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.gmtime(seconds: typed); call it with the wrong type.

typeshed contract: seconds is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import gmtime
try:
    gmtime(_W())  # seconds: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
