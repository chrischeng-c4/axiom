# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "autojunk_true_marks_popular"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_difflib.py"
# status = "filled"
# ///
"""difflib.SequenceMatcher: default autojunk=True marks the repeated 'b' as bpopular for a 200+ sequence, collapsing ratio toward 0"""
import difflib

_seq1 = "b" * 200
_seq2 = "a" + "b" * 200
_sm = difflib.SequenceMatcher(None, _seq1, _seq2)  # default autojunk=True
assert round(_sm.ratio(), 3) == 0.0, f"ratio = {_sm.ratio()!r}"
assert _sm.bpopular == {"b"}, f"bpopular = {_sm.bpopular!r}"
print("autojunk_true_marks_popular OK")
