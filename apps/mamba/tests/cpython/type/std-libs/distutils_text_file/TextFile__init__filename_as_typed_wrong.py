# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_text_file"
# dimension = "type"
# case = "TextFile__init__filename_as_typed_wrong"
# subject = "distutils.text_file.TextFile.__init__(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/text_file.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.text_file.TextFile.__init__(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.text_file import TextFile
try:
    TextFile(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
