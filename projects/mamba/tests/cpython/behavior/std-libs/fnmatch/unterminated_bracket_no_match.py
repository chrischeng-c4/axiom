# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "unterminated_bracket_no_match"
# subject = "fnmatch.fnmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.fnmatch: a malformed pattern is forgiving, not an error: an unterminated bracket '[abc' does not raise and treats the bracket literally so it does not match 'a'"""
import fnmatch

# A bad bracket is not an error; it is treated literally and simply does not
# match. fnmatch is forgiving.
_result = fnmatch.fnmatch("a", "[abc")
assert _result is False, f"unterminated bracket no match = {_result!r}"

print("unterminated_bracket_no_match OK")
