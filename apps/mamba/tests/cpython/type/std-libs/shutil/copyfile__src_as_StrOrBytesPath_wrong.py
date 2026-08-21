# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "type"
# case = "copyfile__src_as_StrOrBytesPath_wrong"
# subject = "shutil.copyfile(src: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: shutil.copyfile(src: StrOrBytesPath); call it with the wrong type.

typeshed contract: src is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shutil import copyfile
try:
    copyfile(_W(), None)  # src: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
