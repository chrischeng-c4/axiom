# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "TemporaryFile__mode_as_OpenBinaryModeWriting_wrong"
# subject = "tempfile.TemporaryFile(mode: OpenBinaryModeWriting)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mode
# mamba-strict-type: TypeError
"""Type wall: tempfile.TemporaryFile(mode: OpenBinaryModeWriting); call it with the wrong type.

typeshed contract: mode is OpenBinaryModeWriting. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import TemporaryFile
try:
    TemporaryFile(_W())  # mode: OpenBinaryModeWriting <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
