# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "copy_is_independent_snapshot"
# subject = "hmac.HMAC.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.copy: copy() returns a distinct object that freezes the original's state at copy time; the original keeps accumulating after digest(), and sibling copies diverge independently"""
import hmac

key = b"key"

# A fresh copy is a distinct object with identical accumulated state.
h1 = hmac.new(key, digestmod="sha256")
h1.update(b"some random text")
h2 = h1.copy()
assert h1 is not h2, "copy is a distinct object"
assert h1.digest() == h2.digest(), "copy digest matches original"
assert h1.hexdigest() == h2.hexdigest(), "copy hexdigest matches original"

# Computing digest() does not finalize state: more updates still apply.
baseline = hmac.new(key, b"some random texttail", digestmod="sha256").digest()
h1.update(b"tail")
assert h1.digest() == baseline, "update after digest() keeps accumulating"

# The copy taken earlier is unaffected by mutations to the original.
assert h2.digest() != h1.digest(), "copy stayed independent of original"
assert h2.digest() == hmac.new(key, b"some random text", digestmod="sha256").digest(), \
    "copy froze the original's state at copy time"

# Copies can themselves diverge from each other.
h3 = h2.copy()
h2.update(b"AAA")
h3.update(b"BBB")
assert h2.digest() != h3.digest(), "sibling copies diverge independently"

print("copy_is_independent_snapshot OK")
