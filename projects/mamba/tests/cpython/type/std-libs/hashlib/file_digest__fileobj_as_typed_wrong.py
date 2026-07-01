# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "type"
# case = "file_digest__fileobj_as_typed_wrong"
# subject = "hashlib.file_digest(fileobj: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hashlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hashlib.file_digest(fileobj: typed); call it with the wrong type.

typeshed contract: fileobj is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hashlib import file_digest
try:
    file_digest(_W(), None)  # fileobj: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
