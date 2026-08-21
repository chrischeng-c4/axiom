# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "ZipReader__init__loader_as_zipimporter_wrong"
# subject = "importlib.readers.ZipReader.__init__(loader: zipimporter)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.ZipReader.__init__(loader: zipimporter); call it with the wrong type.

typeshed contract: loader is zipimporter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import ZipReader
try:
    ZipReader(_W(), "")  # loader: zipimporter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
