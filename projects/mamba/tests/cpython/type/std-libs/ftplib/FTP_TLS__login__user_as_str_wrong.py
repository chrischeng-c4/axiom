# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "FTP_TLS__login__user_as_str_wrong"
# subject = "ftplib.FTP_TLS.login(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.FTP_TLS.login(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ftplib import FTP_TLS
obj = object.__new__(FTP_TLS)
try:
    obj.login(12345)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
