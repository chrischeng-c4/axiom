# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_returns_real_bool"
# subject = "secrets.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest returns a genuine bool (True for equal, False for unequal) for both str and bytes operands"""
import secrets

# Return value is a real bool (not just truthy/falsy), both directions.
_eq = secrets.compare_digest("abc", "abc")
_neq = secrets.compare_digest("abc", "xyz")
assert type(_eq) is bool, f"equal result type = {type(_eq)!r}"
assert type(_neq) is bool, f"unequal result type = {type(_neq)!r}"
assert _eq == True, f"equal result = {_eq!r}"
assert _neq == False, f"unequal result = {_neq!r}"

# Same for bytes operands.
_eq_b = secrets.compare_digest(b"abc", b"abc")
_neq_b = secrets.compare_digest(b"abc", b"xyz")
assert type(_eq_b) is bool, f"equal bytes result type = {type(_eq_b)!r}"
assert type(_neq_b) is bool, f"unequal bytes result type = {type(_neq_b)!r}"
assert _eq_b == True, f"equal bytes result = {_eq_b!r}"
assert _neq_b == False, f"unequal bytes result = {_neq_b!r}"

print("compare_digest_returns_real_bool OK")
