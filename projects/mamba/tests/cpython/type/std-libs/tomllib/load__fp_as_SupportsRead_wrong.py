# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "type"
# case = "load__fp_as_SupportsRead_wrong"
# subject = "tomllib.load(fp: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tomllib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tomllib.load(fp: SupportsRead); call it with the wrong type.

typeshed contract: fp is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tomllib import load
try:
    load(_W())  # fp: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
