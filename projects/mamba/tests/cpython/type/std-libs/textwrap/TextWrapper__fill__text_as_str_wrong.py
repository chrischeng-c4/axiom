# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "type"
# case = "TextWrapper__fill__text_as_str_wrong"
# subject = "textwrap.TextWrapper.fill(text: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/textwrap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: textwrap.TextWrapper.fill(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from textwrap import TextWrapper
obj = object.__new__(TextWrapper)
try:
    obj.fill(12345)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
