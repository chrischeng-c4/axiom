# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipInfo__FileHeader__zip64_as_typed_wrong"
# subject = "zipfile.ZipInfo.FileHeader(zip64: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipInfo.FileHeader(zip64: typed); call it with the wrong type.

typeshed contract: zip64 is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipInfo
obj = object.__new__(ZipInfo)
try:
    obj.FileHeader(_W())  # zip64: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
