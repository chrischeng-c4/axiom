# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "TarInfo__tarfile__tarfile_as_typed_wrong"
# subject = "tarfile.TarInfo.tarfile(tarfile: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed tarfile"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed tarfile
# mamba-strict-type: TypeError
"""Type wall: tarfile.TarInfo.tarfile(tarfile: typed); call it with the wrong type.

typeshed contract: tarfile is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import TarInfo
obj = object.__new__(TarInfo)
try:
    obj.tarfile(_W())  # tarfile: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
