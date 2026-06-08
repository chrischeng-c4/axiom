# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "behavior"
# case = "token_constants_are_ints"
# subject = "token.NAME"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/token.py"
# status = "filled"
# ///
"""token.NAME: the public token-type constants NAME/NUMBER/OP/STRING/ENDMARKER are plain ints"""
import token

# Every public token-type constant is a plain int (not bool, not a subclass).
for name in ["NAME", "NUMBER", "OP", "STRING", "ENDMARKER"]:
    value = getattr(token, name)
    assert isinstance(value, int), (name, type(value).__name__)
    assert type(value) is int, (name, type(value).__name__)

print("token_constants_are_ints OK")
