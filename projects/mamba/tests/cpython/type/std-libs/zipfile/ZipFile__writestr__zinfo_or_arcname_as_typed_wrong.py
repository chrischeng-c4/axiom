# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipFile__writestr__zinfo_or_arcname_as_typed_wrong"
# subject = "zipfile.ZipFile.writestr(zinfo_or_arcname: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipFile.writestr(zinfo_or_arcname: typed); call it with the wrong type.

typeshed contract: zinfo_or_arcname is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipFile
obj = object.__new__(ZipFile)
try:
    obj.writestr(_W(), None)  # zinfo_or_arcname: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
