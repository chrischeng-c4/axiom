# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipimport"
# dimension = "type"
# case = "zipimporter__get_data__pathname_as_str_wrong"
# subject = "zipimport.zipimporter.get_data(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipimport.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipimport.zipimporter.get_data(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipimport import zipimporter
obj = object.__new__(zipimporter)
try:
    obj.get_data(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
