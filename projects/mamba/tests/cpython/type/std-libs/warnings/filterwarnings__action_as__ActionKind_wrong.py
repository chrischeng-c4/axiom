# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "filterwarnings__action_as__ActionKind_wrong"
# subject = "warnings.filterwarnings(action: _ActionKind)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.filterwarnings(action: _ActionKind); call it with the wrong type.

typeshed contract: action is _ActionKind. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import filterwarnings
try:
    filterwarnings(_W())  # action: _ActionKind <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
