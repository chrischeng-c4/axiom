# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "positional_only"
# dimension = "surface"
# case = "mixed_signature_parses_and_calls"
# subject = "/"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""/: a mixed signature (pos-only `/`, regular, keyword-only `*`) parses and is callable with the regular param given positionally or by keyword"""

# A mixed signature: pos-only before `/`, a regular param, kw-only after `*`.
def _mixed(pos_only: int, /, regular: int, *, kw_only: int) -> int:
    return pos_only + regular + kw_only

# The regular param may be passed positionally or by keyword.
assert _mixed(1, 2, kw_only=3) == 6, _mixed(1, 2, kw_only=3)
assert _mixed(1, regular=2, kw_only=3) == 6, _mixed(1, regular=2, kw_only=3)

print("mixed_signature_parses_and_calls OK")
