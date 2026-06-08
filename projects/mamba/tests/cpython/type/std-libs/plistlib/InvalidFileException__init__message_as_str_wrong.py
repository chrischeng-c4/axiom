# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "InvalidFileException__init__message_as_str_wrong"
# subject = "plistlib.InvalidFileException.__init__(message: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.InvalidFileException.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from plistlib import InvalidFileException
try:
    InvalidFileException(12345)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
