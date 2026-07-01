# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "type"
# case = "ZipExtFile__init__fileobj_as__ClosableZipStream_wrong"
# subject = "zipfile.ZipExtFile.__init__(fileobj: _ClosableZipStream)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile.ZipExtFile.__init__(fileobj: _ClosableZipStream); call it with the wrong type.

typeshed contract: fileobj is _ClosableZipStream. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile import ZipExtFile
try:
    ZipExtFile(_W(), None, None, None, None)  # fileobj: _ClosableZipStream <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
