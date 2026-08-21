# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "tar_filter__member_as_TarInfo_wrong"
# subject = "tarfile.tar_filter(member: TarInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tarfile.tar_filter(member: TarInfo); call it with the wrong type.

typeshed contract: member is TarInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import tar_filter
try:
    tar_filter(_W(), "")  # member: TarInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
