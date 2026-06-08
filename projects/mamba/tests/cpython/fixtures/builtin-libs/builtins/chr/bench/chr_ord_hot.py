"""Hot-loop bench for builtins.chr and builtins.ord: roundtrip.

Domain: builtins
Feature: chr
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling chr(n) then ord() on the result —
monomorphic on int codepoints in printable ASCII range.
"""
# tier: compute


ITERS = 500_000

_codes = list(range(32, 127))  # printable ASCII
_n = len(_codes)

acc = 0
for i in range(ITERS):
    c = chr(_codes[i % _n])
    acc ^= ord(c)

# Stdout sink — byte-equal across runtimes.
print(f"chr_ord: {ITERS}")
