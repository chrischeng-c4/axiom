# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "read_mime_types__file_as_StrPath_wrong"
# subject = "mimetypes.read_mime_types(file: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.read_mime_types(file: StrPath); call it with the wrong type.

typeshed contract: file is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import read_mime_types
try:
    read_mime_types(_W())  # file: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
