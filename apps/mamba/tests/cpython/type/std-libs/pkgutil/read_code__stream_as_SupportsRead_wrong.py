# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "read_code__stream_as_SupportsRead_wrong"
# subject = "pkgutil.read_code(stream: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.read_code(stream: SupportsRead); call it with the wrong type.

typeshed contract: stream is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pkgutil import read_code
try:
    read_code(_W())  # stream: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
