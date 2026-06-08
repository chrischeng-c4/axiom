# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "restore_recovers_original_sequences"
# subject = "difflib.restore"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.restore: restore(ndiff_output, 1) rebuilds the from-sequence and restore(..., 2) rebuilds the to-sequence"""
import difflib

_from = ["one\n", "two\n", "three\n"]
_to = ["ore\n", "tree\n", "emu\n"]
_diff = list(difflib.ndiff(_from, _to))
assert list(difflib.restore(_diff, 1)) == _from, "restore(diff, 1) rebuilds from-seq"
assert list(difflib.restore(_diff, 2)) == _to, "restore(diff, 2) rebuilds to-seq"
print("restore_recovers_original_sequences OK")
