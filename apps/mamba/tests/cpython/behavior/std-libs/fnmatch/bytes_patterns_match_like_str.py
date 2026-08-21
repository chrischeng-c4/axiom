# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "behavior"
# case = "bytes_patterns_match_like_str"
# subject = "fnmatch.fnmatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_fnmatch.py"
# status = "filled"
# ///
"""fnmatch.fnmatch: all-bytes args match like the str version: fnmatch(b'test.py', b'*.py') is True, fnmatch(b'test.rs', b'*.py') is False, fnmatchcase stays case-sensitive on bytes"""
import fnmatch

# fnmatch with all-bytes args matches like the str version (lowercase inputs
# avoid the OS-dependent case fold of fnmatch).
assert fnmatch.fnmatch(b"test.py", b"*.py") is True, "bytes fnmatch suffix"
assert fnmatch.fnmatch(b"test.rs", b"*.py") is False, "bytes fnmatch no match"

# fnmatchcase with bytes is strictly case-sensitive.
assert fnmatch.fnmatchcase(b"test.PY", b"*.PY") is True, "bytes upper suffix"
assert fnmatch.fnmatchcase(b"test.py", b"*.PY") is False, "bytes case mismatch"

print("bytes_patterns_match_like_str OK")
