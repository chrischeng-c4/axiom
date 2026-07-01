# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "type"
# case = "Completer__init__namespace_as_typed_wrong"
# subject = "rlcompleter.Completer.__init__(namespace: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/rlcompleter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: rlcompleter.Completer.__init__(namespace: typed); call it with the wrong type.

typeshed contract: namespace is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from rlcompleter import Completer
try:
    Completer(_W())  # namespace: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
