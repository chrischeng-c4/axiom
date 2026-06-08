# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_bytes_length_matches_request"
# subject = "secrets.token_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_bytes: token_bytes(n) returns exactly n bytes for n in 0,1,8,16,32,64 (default is DEFAULT_ENTROPY=32)"""
import secrets

for _n in [0, 1, 8, 16, 32, 64]:
    _tb = secrets.token_bytes(_n)
    assert isinstance(_tb, bytes), f"token_bytes({_n}) type = {type(_tb)!r}"
    assert len(_tb) == _n, f"token_bytes({_n}) len = {len(_tb)!r}"

# Default size is DEFAULT_ENTROPY (32 bytes in CPython 3.12).
assert len(secrets.token_bytes()) == secrets.DEFAULT_ENTROPY, "token_bytes() default len"
assert secrets.DEFAULT_ENTROPY == 32, "DEFAULT_ENTROPY value"

print("token_bytes_length_matches_request OK")
