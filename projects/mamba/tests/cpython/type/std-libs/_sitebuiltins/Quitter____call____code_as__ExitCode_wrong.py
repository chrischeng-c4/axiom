# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sitebuiltins"
# dimension = "type"
# case = "Quitter____call____code_as__ExitCode_wrong"
# subject = "_sitebuiltins.Quitter.__call__(code: _ExitCode)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sitebuiltins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sitebuiltins.Quitter.__call__(code: _ExitCode); call it with the wrong type.

typeshed contract: code is _ExitCode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _sitebuiltins import Quitter
obj = object.__new__(Quitter)
try:
    obj.__call__(_W())  # code: _ExitCode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
