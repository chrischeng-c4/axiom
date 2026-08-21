# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "TarFile__gettarinfo__name_as_typed_wrong"
# subject = "tarfile.TarFile.gettarinfo(name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tarfile.TarFile.gettarinfo(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import TarFile
obj = object.__new__(TarFile)
try:
    obj.gettarinfo(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
