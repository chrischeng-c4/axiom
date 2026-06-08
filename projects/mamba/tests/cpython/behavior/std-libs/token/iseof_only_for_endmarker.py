# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "iseof_only_for_endmarker"
# subject = "token.ISEOF"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.ISEOF: ISEOF is True only for the ENDMARKER type (0) and False for ordinary token types"""
import token

# ISEOF flags the end-of-input marker only.
assert token.ENDMARKER == 0, token.ENDMARKER
assert token.ISEOF(token.ENDMARKER) is True
assert token.ISEOF(0) is True

# Every ordinary token type is not EOF.
for tok_type in [token.NAME, token.NUMBER, token.OP, token.STRING, 1, 256]:
    assert token.ISEOF(tok_type) is False, tok_type

print("iseof_only_for_endmarker OK")
