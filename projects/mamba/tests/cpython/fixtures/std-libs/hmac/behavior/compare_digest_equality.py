# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "compare_digest_equality"
# subject = "hmac.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.compare_digest: compare_digest is True for equal bytes/str, False for differing or different-length inputs, and never raises for same-type args"""
import hmac

# bytes vs bytes.
assert hmac.compare_digest(b"same", b"same"), "bytes same"
assert not hmac.compare_digest(b"abc", b"xyz"), "bytes different"

# str vs str.
assert hmac.compare_digest("same", "same"), "str same"
assert not hmac.compare_digest("abc", "xyz"), "str different"
assert hmac.compare_digest("", ""), "empty str equal"

# Different lengths return False (not raise).
assert not hmac.compare_digest(b"short", b"much_longer_string"), "diff lengths"
assert not hmac.compare_digest("ab", "abc"), "diff str lengths"

print("compare_digest_equality OK")
