# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_hex_outputs_are_unique"
# subject = "secrets.token_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_hex: 100 consecutive token_hex(16) draws are all distinct (no collision / no constant stub)"""
import secrets

# 16-byte tokens collide with probability ~2**-128; a duplicate signals a
# constant stub or a broken RNG, not a flake.
_seen = set()
for _draw in range(100):
    _seen.add(secrets.token_hex(16))
_collisions = 100 - len(_seen)
assert _collisions == 0, f"token_hex(16) collisions: {_collisions}"

print("token_hex_outputs_are_unique OK")
