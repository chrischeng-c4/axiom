# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path_glob"
# dimension = "type"
# case = "Translator__init__seps_as_str_wrong"
# subject = "zipfile._path.glob.Translator.__init__(seps: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed seps"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path/glob.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed seps
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.glob.Translator.__init__(seps: str); call it with the wrong type.

typeshed contract: seps is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zipfile._path.glob import Translator
try:
    Translator(12345)  # seps: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
