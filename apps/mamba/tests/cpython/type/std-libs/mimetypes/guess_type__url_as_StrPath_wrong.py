# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "guess_type__url_as_StrPath_wrong"
# subject = "mimetypes.guess_type(url: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.guess_type(url: StrPath); call it with the wrong type.

typeshed contract: url is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import guess_type
try:
    guess_type(_W())  # url: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
