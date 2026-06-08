# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "TarInfo__fromtarfile__tarfile_as_TarFile_wrong"
# subject = "tarfile.TarInfo.fromtarfile(tarfile: TarFile)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tarfile.TarInfo.fromtarfile(tarfile: TarFile); call it with the wrong type.

typeshed contract: tarfile is TarFile. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import TarInfo
try:
    TarInfo.fromtarfile(_W())  # tarfile: TarFile <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
