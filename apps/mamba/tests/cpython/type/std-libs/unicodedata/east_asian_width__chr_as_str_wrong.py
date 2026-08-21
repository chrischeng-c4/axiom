# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "east_asian_width__chr_as_str_wrong"
# subject = "unicodedata.east_asian_width(chr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.east_asian_width(chr: str); call it with the wrong type.

typeshed contract: chr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unicodedata import east_asian_width
try:
    east_asian_width(12345)  # chr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
