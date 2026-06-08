# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_distinguishes_length"
# subject = "secrets.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest reports unequal for same-prefix different-length operands and for equal-length last-char-differing operands, str and bytes alike"""
import secrets

# Long equal operands compare equal, str and bytes alike.
for _s in ("a", "bcd", "xyz123"):
    _a = _s * 100
    assert secrets.compare_digest(_a, _a), f"equal str x100: {_s!r}"
    _ab = _a.encode("utf-8")
    assert secrets.compare_digest(_ab, _ab), f"equal bytes x100: {_s!r}"

# Equal-length operands differing only in the last char compare unequal.
for _s in ("x", "mn", "a1b2c3"):
    _base = _s * 100
    assert not secrets.compare_digest(_base + "q", _base + "k"), f"last-char diff str: {_s!r}"
    assert not secrets.compare_digest(
        (_base + "q").encode("utf-8"), (_base + "k").encode("utf-8")
    ), f"last-char diff bytes: {_s!r}"

# Same-prefix different-length operands compare unequal (str and bytes).
assert not secrets.compare_digest("abc", "abcd"), "shorter vs longer str"
assert not secrets.compare_digest(b"abc", b"abcd"), "shorter vs longer bytes"

print("compare_digest_distinguishes_length OK")
