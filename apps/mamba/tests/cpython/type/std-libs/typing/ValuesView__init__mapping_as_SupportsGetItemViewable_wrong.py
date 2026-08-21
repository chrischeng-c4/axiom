# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "ValuesView__init__mapping_as_SupportsGetItemViewable_wrong"
# subject = "typing.ValuesView.__init__(mapping: SupportsGetItemViewable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing.ValuesView.__init__(mapping: SupportsGetItemViewable); call it with the wrong type.

typeshed contract: mapping is SupportsGetItemViewable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import ValuesView
try:
    ValuesView(_W())  # mapping: SupportsGetItemViewable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
