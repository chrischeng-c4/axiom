# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_csv"
# dimension = "type"
# case = "writer__fileobj_as_SupportsWrite_wrong"
# subject = "_csv.writer(fileobj: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_csv.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _csv.writer(fileobj: SupportsWrite); call it with the wrong type.

typeshed contract: fileobj is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _csv import writer
try:
    writer(_W())  # fileobj: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
