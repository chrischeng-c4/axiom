# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "IS_CHARACTER_JUNK__ch_as_str_wrong"
# subject = "difflib.IS_CHARACTER_JUNK(ch: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: difflib.IS_CHARACTER_JUNK(ch: str); call it with the wrong type.

typeshed contract: ch is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from difflib import IS_CHARACTER_JUNK
try:
    IS_CHARACTER_JUNK(12345)  # ch: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
