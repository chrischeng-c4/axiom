# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__check_lib__library_as_str_wrong"
# subject = "distutils.command.config.config.check_lib(library: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.check_lib(library: str); call it with the wrong type.

typeshed contract: library is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.check_lib(12345)  # library: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
