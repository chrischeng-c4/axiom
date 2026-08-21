# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_readers"
# dimension = "type"
# case = "FileReader__init__loader_as_FileLoader_wrong"
# subject = "importlib.readers.FileReader.__init__(loader: FileLoader)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/readers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.readers.FileReader.__init__(loader: FileLoader); call it with the wrong type.

typeshed contract: loader is FileLoader. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.readers import FileReader
try:
    FileReader(_W())  # loader: FileLoader <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
