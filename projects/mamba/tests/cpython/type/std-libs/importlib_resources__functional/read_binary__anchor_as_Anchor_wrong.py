# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "read_binary__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.read_binary(anchor: Anchor)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed anchor"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed anchor
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.read_binary(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import read_binary
try:
    read_binary(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
