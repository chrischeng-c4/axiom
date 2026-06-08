# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_config"
# dimension = "type"
# case = "config__search_cpp__pattern_as_typed_wrong"
# subject = "distutils.command.config.config.search_cpp(pattern: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/config.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern
# mamba-strict-type: TypeError
"""Type wall: distutils.command.config.config.search_cpp(pattern: typed); call it with the wrong type.

typeshed contract: pattern is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.command.config import config
obj = object.__new__(config)
try:
    obj.search_cpp(_W())  # pattern: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
