# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__guess_file_type__path_as_StrPath_wrong"
# subject = "mimetypes.MimeTypes.guess_file_type(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.guess_file_type(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.guess_file_type(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
