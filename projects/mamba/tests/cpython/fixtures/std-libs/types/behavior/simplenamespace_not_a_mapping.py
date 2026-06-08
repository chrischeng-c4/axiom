# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_not_a_mapping"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: SimpleNamespace is not a mapping: len/iter/contains/getitem each raise TypeError"""
import types

plain = types.SimpleNamespace(spam="spamspamspam")
for op in (lambda: len(plain), lambda: iter(plain),
           lambda: "spam" in plain, lambda: plain["spam"]):
    _raised = False
    try:
        op()
    except TypeError:
        _raised = True
    assert _raised, "SimpleNamespace mapping op should raise TypeError"

print("simplenamespace_not_a_mapping OK")
