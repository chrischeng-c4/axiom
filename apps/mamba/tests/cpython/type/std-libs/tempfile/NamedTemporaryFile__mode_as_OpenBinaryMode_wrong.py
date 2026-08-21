# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "NamedTemporaryFile__mode_as_OpenBinaryMode_wrong"
# subject = "tempfile.NamedTemporaryFile(mode: OpenBinaryMode)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode
# mamba-strict-type: TypeError
"""Type wall: tempfile.NamedTemporaryFile(mode: OpenBinaryMode); call it with the wrong type.

typeshed contract: mode is OpenBinaryMode. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import NamedTemporaryFile
try:
    NamedTemporaryFile(_W())  # mode: OpenBinaryMode <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
