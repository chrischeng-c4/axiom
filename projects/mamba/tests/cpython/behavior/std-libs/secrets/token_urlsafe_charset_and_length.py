# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_urlsafe_charset_and_length"
# subject = "secrets.token_urlsafe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_urlsafe: token_urlsafe(n) returns an unpadded URL-safe base64 str (chars in A-Za-z0-9-_) of length >= n; token_urlsafe(3) is exactly 4 chars"""
import secrets

# URL-safe base64 alphabet: letters, digits, '-' and '_'; never padding ('=')
# nor the non-URL-safe '+' / '/'.
_legal = set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")
for _n in [8, 16, 32]:
    _tu = secrets.token_urlsafe(_n)
    assert isinstance(_tu, str), f"token_urlsafe({_n}) type = {type(_tu)!r}"
    assert len(_tu) >= _n, f"token_urlsafe({_n}) len = {len(_tu)!r}"
    assert all(c in _legal for c in _tu), f"token_urlsafe({_n}) charset = {_tu!r}"
    assert "=" not in _tu, f"token_urlsafe({_n}) must not pad"

# n=3 -> ceil(4*3/3) = 4 chars, no padding stripped.
assert len(secrets.token_urlsafe(3)) == 4, "token_urlsafe(3) len"

print("token_urlsafe_charset_and_length OK")
