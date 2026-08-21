# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__unreadline__line_as_str_wrong"
# subject = "distutils.text_file.TextFile.unreadline(line: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.unreadline(line: str); call it with the wrong type.

typeshed contract: line is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.text_file import TextFile
obj = object.__new__(TextFile)
try:
    obj.unreadline(12345)  # line: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
