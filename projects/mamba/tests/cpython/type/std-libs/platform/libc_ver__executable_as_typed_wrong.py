# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "libc_ver__executable_as_typed_wrong"
# subject = "platform.libc_ver(executable: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.libc_ver(executable: typed); call it with the wrong type.

typeshed contract: executable is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from platform import libc_ver
try:
    libc_ver(_W())  # executable: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
