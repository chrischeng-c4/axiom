# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "CompleteDirs__make__source_as_ZipFile_wrong"
# subject = "zipfile._path.CompleteDirs.make(source: ZipFile)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed source"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed source
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.CompleteDirs.make(source: ZipFile); call it with the wrong type.

typeshed contract: source is ZipFile. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path import CompleteDirs
try:
    CompleteDirs.make(_W())  # source: ZipFile <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
