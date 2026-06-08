# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "SequenceMatcher__set_seq2__b_as_Sequence_wrong"
# subject = "difflib.SequenceMatcher.set_seq2(b: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed b"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed b
# mamba-strict-type: TypeError
"""Type wall: difflib.SequenceMatcher.set_seq2(b: Sequence); call it with the wrong type.

typeshed contract: b is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from difflib import SequenceMatcher
obj = object.__new__(SequenceMatcher)
try:
    obj.set_seq2(_W())  # b: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
