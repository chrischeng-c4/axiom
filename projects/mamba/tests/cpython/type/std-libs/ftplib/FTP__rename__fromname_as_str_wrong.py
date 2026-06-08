# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__rename__fromname_as_str_wrong"
# subject = "ftplib.FTP.rename(fromname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.rename(fromname: str); call it with the wrong type.

typeshed contract: fromname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.rename(12345, "")  # fromname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
