# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "type"
# case = "copyfileobj__fsrc_as_SupportsRead_wrong"
# subject = "shutil.copyfileobj(fsrc: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shutil.copyfileobj(fsrc: SupportsRead); call it with the wrong type.

typeshed contract: fsrc is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shutil import copyfileobj
try:
    copyfileobj(_W(), None)  # fsrc: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
