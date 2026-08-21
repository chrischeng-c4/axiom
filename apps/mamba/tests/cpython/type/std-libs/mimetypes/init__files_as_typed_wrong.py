# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "type"
# case = "init__files_as_typed_wrong"
# subject = "mimetypes.init(files: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mimetypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mimetypes.init(files: typed); call it with the wrong type.

typeshed contract: files is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mimetypes import init
try:
    init(_W())  # files: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
