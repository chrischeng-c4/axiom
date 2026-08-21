# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "TarFile__add__name_as_StrPath_wrong"
# subject = "tarfile.TarFile.add(name: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tarfile.TarFile.add(name: StrPath); call it with the wrong type.

typeshed contract: name is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import TarFile
obj = object.__new__(TarFile)
try:
    obj.add(_W())  # name: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
