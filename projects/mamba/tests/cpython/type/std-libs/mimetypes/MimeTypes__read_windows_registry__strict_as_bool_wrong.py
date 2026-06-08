# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "MimeTypes__read_windows_registry__strict_as_bool_wrong"
# subject = "mimetypes.MimeTypes.read_windows_registry(strict: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed strict"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed strict
# mamba-strict-type: TypeError
"""Type wall: mimetypes.MimeTypes.read_windows_registry(strict: bool); call it with the wrong type.

typeshed contract: strict is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mimetypes import MimeTypes
obj = object.__new__(MimeTypes)
try:
    obj.read_windows_registry("not_a_bool")  # strict: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
