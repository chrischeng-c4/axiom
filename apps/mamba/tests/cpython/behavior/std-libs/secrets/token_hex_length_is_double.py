# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_hex_length_is_double"
# subject = "secrets.token_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_hex: token_hex(n) returns a lowercase-hex str of length 2*n for n in 4,8,16,32; default is 64 chars (2*DEFAULT_ENTROPY)"""
import secrets

for _n in [4, 8, 16, 32]:
    _th = secrets.token_hex(_n)
    assert isinstance(_th, str), f"token_hex({_n}) type = {type(_th)!r}"
    assert len(_th) == 2 * _n, f"token_hex({_n}) len = {len(_th)!r}"
    assert all(c in "0123456789abcdef" for c in _th), f"token_hex({_n}) charset"

# Default is 2 * DEFAULT_ENTROPY = 64 hex chars.
assert len(secrets.token_hex()) == 64, "token_hex() default len"

print("token_hex_length_is_double OK")
