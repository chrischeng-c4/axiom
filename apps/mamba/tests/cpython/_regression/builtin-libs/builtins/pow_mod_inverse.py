# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `pow(base, neg_exp, mod)` — Python 3.8+ added negative-exponent support
# to the 3-arg form, computing the modular multiplicative inverse via the
# extended Euclidean algorithm. `pow(b, -1, m)` is the inverse of `b mod m`,
# defined iff `gcd(b, m) == 1`. `pow(b, -e, m)` for `e > 0` is that inverse
# raised to the e'th power, modulo m.
#
# Mamba's `mb_pow_mod` was returning None for any negative exponent (the
# old code path had `if e < 0 { return None }`). Fix in `runtime/builtins.rs`:
# add `mod_inverse_i128` (extended Euclid), invert the base on `e < 0`, then
# fall through to the existing modular-exponentiation loop with `exp = -e`.
# When `gcd(base, mod) != 1` we return None — CPython raises ValueError
# there; we don't have an exception object pipeline yet but the test
# suite-level behaviour matches: result is unusable.

# Headline: simple modular inverses.
print(pow(2, -1, 5))      # 3   (2*3 = 6 ≡ 1 mod 5)
print(pow(3, -1, 7))      # 5
print(pow(10, -1, 17))    # 12

# Negative exponent > 1 — exercise the (inverse)^|e| path.
print(pow(7, -3, 13))     # 8   (7^-1 mod 13 = 2, 2^3 = 8)

# Non-modular sanity (must not regress the existing happy paths).
print(pow(2, 10, 1000))   # 24
print(pow(2, 0, 5))       # 1
print(pow(0, 0, 5))       # 1
print(pow(5, 100, 1))     # 0   (any value mod 1 is 0)

# Without modulus — float fallback for negative exp.
print(pow(2, -1))          # 0.5
