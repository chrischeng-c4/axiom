# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "type"
# case = "BaseSelector__select__timeout_as_typed_wrong"
# subject = "selectors.BaseSelector.select(timeout: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/selectors.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: selectors.BaseSelector.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from selectors import BaseSelector
obj = object.__new__(BaseSelector)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
