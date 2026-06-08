# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "type"
# case = "crypt__word_as_str_wrong"
# subject = "crypt.crypt(word: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/crypt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: crypt.crypt(word: str); call it with the wrong type.

typeshed contract: word is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from crypt import crypt
try:
    crypt(12345)  # word: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
