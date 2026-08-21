# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "StackSummary__extract__frame_gen_as_Iterable_wrong"
# subject = "traceback.StackSummary.extract(frame_gen: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.StackSummary.extract(frame_gen: Iterable); call it with the wrong type.

typeshed contract: frame_gen is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import StackSummary
try:
    StackSummary.extract(_W())  # frame_gen: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
