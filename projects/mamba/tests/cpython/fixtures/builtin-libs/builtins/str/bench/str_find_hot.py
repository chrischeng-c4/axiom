"""Hot-loop bench for builtins.str: str.find in a fixed string.

Domain: builtins
Feature: str
Tier: compute

# type-regime: monomorphic

End-user scenario: tight loop calling str.find on a fixed string with
a fixed pattern — monomorphic on str inputs.
"""
# tier: compute


ITERS = 500_000

_text = "the quick brown fox jumps over the lazy dog"
_patterns = ["fox", "dog", "the", "xyz", "quick"]
_n = len(_patterns)

acc = 0
for i in range(ITERS):
    result = _text.find(_patterns[i % _n])
    acc ^= (result + 1)

# Stdout sink — byte-equal across runtimes.
print(f"str_find: {ITERS}")
