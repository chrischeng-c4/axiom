# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_spawn"
# dimension = "type"
# case = "find_executable__executable_as_str_wrong"
# subject = "distutils.spawn.find_executable(executable: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/spawn.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.spawn.find_executable(executable: str); call it with the wrong type.

typeshed contract: executable is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.spawn import find_executable
try:
    find_executable(12345)  # executable: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
