# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "dump__fp_as_SupportsWrite_wrong"
# subject = "json.dump(fp: SupportsWrite)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.dump(fp: SupportsWrite); call it with the wrong type.

typeshed contract: fp is SupportsWrite. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import dump
try:
    dump(None, _W())  # fp: SupportsWrite <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
