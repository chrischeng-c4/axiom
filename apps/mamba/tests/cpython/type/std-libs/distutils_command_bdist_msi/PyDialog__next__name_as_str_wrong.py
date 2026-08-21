# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_bdist_msi"
# dimension = "type"
# case = "PyDialog__next__name_as_str_wrong"
# subject = "distutils.command.bdist_msi.PyDialog.next(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/bdist_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.bdist_msi.PyDialog.next(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.bdist_msi import PyDialog
obj = object.__new__(PyDialog)
try:
    obj.next(None, None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
