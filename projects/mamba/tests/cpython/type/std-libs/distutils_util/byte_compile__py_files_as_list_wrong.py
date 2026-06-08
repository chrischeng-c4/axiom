# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_util"
# dimension = "type"
# case = "byte_compile__py_files_as_list_wrong"
# subject = "distutils.util.byte_compile(py_files: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed py_files"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/util.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed py_files
# mamba-strict-type: TypeError
"""Type wall: distutils.util.byte_compile(py_files: list); call it with the wrong type.

typeshed contract: py_files is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.util import byte_compile
try:
    byte_compile(12345)  # py_files: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
