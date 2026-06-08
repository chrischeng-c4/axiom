# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP__set_pasv__val_as_typed_wrong"
# subject = "ftplib.FTP.set_pasv(val: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed val"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed val
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP.set_pasv(val: typed); call it with the wrong type.

typeshed contract: val is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ftplib import FTP
obj = object.__new__(FTP)
try:
    obj.set_pasv(_W())  # val: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
