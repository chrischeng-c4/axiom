# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "SequenceMatcher__init__isjunk_as_typed_wrong"
# subject = "difflib.SequenceMatcher.__init__(isjunk: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: difflib.SequenceMatcher.__init__(isjunk: typed); call it with the wrong type.

typeshed contract: isjunk is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from difflib import SequenceMatcher
try:
    SequenceMatcher(_W(), None, None)  # isjunk: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
