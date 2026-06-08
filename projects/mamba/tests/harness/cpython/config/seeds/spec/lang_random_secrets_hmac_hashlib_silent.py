# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(random, 'SystemRandom')` (the
# documented "random exposes the SystemRandom OS-entropy class" —
# mamba returns False), `hasattr(random, 'BPF')` (the documented
# "random exposes the BPF bit-precision constant" — mamba returns
# False), `hasattr(random, 'NV_MAGICCONST')` (the documented "random
# exposes the NV_MAGICCONST gauss helper" — mamba returns False),
# `hasattr(hmac, 'trans_36')` (the documented "hmac exposes the
# trans_36 inner-pad translation table" — mamba returns False),
# `hasattr(hashlib, 'pbkdf2_hmac')` (the documented "hashlib exposes
# pbkdf2_hmac" — mamba returns False), `hasattr(hashlib, 'scrypt')`
# (the documented "hashlib exposes scrypt KDF" — mamba returns False),
# `type(random.Random).__name__` (the documented "Random is a class
# whose metatype is 'type'" — mamba returns 'function' — Random is
# constructor-as-function), `type(hmac.HMAC).__name__` (the
# documented "HMAC is a class whose metatype is 'type'" — mamba
# returns 'function'), `type(hmac.new(b'k', b'm', 'sha256')).__name__`
# (the documented "hmac.new returns an HMAC instance" — mamba returns
# 'int' — handle-table pattern), and `type(hashlib.algorithms_
# guaranteed).__name__` (the documented "algorithms_guaranteed is a
# frozen set" — mamba returns 'list').
# Ten-pack pinned to atomic 279.
#
# Behavioral edges that CONFORM on mamba (random — hasattr Random/
# seed/random/randint/randrange/choice/sample/shuffle/uniform/
# getrandbits/getstate/setstate + bounded outputs + seed
# reproducibility. secrets — hasattr choice/randbelow/randbits/token_
# bytes/token_hex/token_urlsafe/compare_digest + token_bytes(4) len 4
# + token_hex(4) len 8 + bounded outputs + compare_digest True/False.
# hmac — hasattr new/HMAC/compare_digest/digest + sha256 hexdigest
# len 64 + digest len 32 + compare_digest True/False. hashlib —
# hasattr new/md5/sha1/sha256/sha512/sha3_256/blake2b/blake2s/
# algorithms_guaranteed/algorithms_available + md5/sha1 reference
# digests + sha256/sha512 lengths + 'md5'/'sha256' in
# algorithms_guaranteed + hashlib.new('md5', b'abc') roundtrip) are
# covered in the matching pass fixture `test_random_secrets_hmac_
# hashlib_value_ops`.
import random
import hmac
import hashlib


_ledger: list[int] = []

# 1) hasattr(random, 'SystemRandom') — OS-entropy class
#    (mamba: returns False)
assert hasattr(random, "SystemRandom") == True; _ledger.append(1)

# 2) hasattr(random, 'BPF') — bit-precision constant
#    (mamba: returns False)
assert hasattr(random, "BPF") == True; _ledger.append(1)

# 3) hasattr(random, 'NV_MAGICCONST') — gauss helper
#    (mamba: returns False)
assert hasattr(random, "NV_MAGICCONST") == True; _ledger.append(1)

# 4) hasattr(hmac, 'trans_36') — inner-pad translation table
#    (mamba: returns False)
assert hasattr(hmac, "trans_36") == True; _ledger.append(1)

# 5) hasattr(hashlib, 'pbkdf2_hmac') — PBKDF2 KDF
#    (mamba: returns False)
assert hasattr(hashlib, "pbkdf2_hmac") == True; _ledger.append(1)

# 6) hasattr(hashlib, 'scrypt') — scrypt KDF
#    (mamba: returns False)
assert hasattr(hashlib, "scrypt") == True; _ledger.append(1)

# 7) type(random.Random).__name__ == 'type' — Random metatype
#    (mamba: returns 'function')
assert type(random.Random).__name__ == "type"; _ledger.append(1)

# 8) type(hmac.HMAC).__name__ == 'type' — HMAC metatype
#    (mamba: returns 'function')
assert type(hmac.HMAC).__name__ == "type"; _ledger.append(1)

# 9) type(hmac.new(...)).__name__ == 'HMAC' — instance class
#    (mamba: returns 'int' — handle-table pattern)
assert type(hmac.new(b"k", b"m", "sha256")).__name__ == "HMAC"; _ledger.append(1)

# 10) type(hashlib.algorithms_guaranteed).__name__ == 'set' — frozen set
#     (mamba: returns 'list')
assert type(hashlib.algorithms_guaranteed).__name__ == "set"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_random_secrets_hmac_hashlib_silent {sum(_ledger)} asserts")
