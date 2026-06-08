# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "behavior"
# case = "get_opcodes_reconstruct_target"
# subject = "difflib.SequenceMatcher"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""difflib.SequenceMatcher: applying the opcodes of ABCDE->ACE (taking equal/insert/replace from b, skipping delete) reconstructs 'ACE'"""
import difflib

_sm = difflib.SequenceMatcher(None, "ABCDE", "ACE")
_result = []
for _tag, _i1, _i2, _j1, _j2 in _sm.get_opcodes():
    if _tag == "equal":
        _result.extend("ABCDE"[_i1:_i2])
    elif _tag == "insert":
        _result.extend("ACE"[_j1:_j2])
    elif _tag == "replace":
        _result.extend("ACE"[_j1:_j2])
    # "delete" just skips the old span.
assert "".join(_result) == "ACE", f"opcode apply = {''.join(_result)!r}"
print("get_opcodes_reconstruct_target OK")
