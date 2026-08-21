# Operational AssertionPass seed for the value contract of the
# `hashlib` / `hmac` / `secrets` / `random` / `uuid` / `math`
# six-pack pinned to atomic 177: `hashlib` (the documented
# `md5` / `sha1` / `sha256` / `sha512` / `sha224` / `sha384`
# message-digest constructor value contract + the documented
# `new` factory + the documented `digest_size` instance
# attribute + the documented hashlib module hasattr surface),
# `hmac` (the documented `new` constructor + `hexdigest`
# instance method + `compare_digest` module-level helper),
# `secrets` (the documented `token_hex` / `token_bytes` length
# contract + the documented `randbelow` / `choice` module-
# level helper), `random` (the documented `random` / `randint`
# / `choice` / `sample` / `shuffle` / `uniform` module-level
# helper shape contract + the documented `random` module
# hasattr surface), `uuid` (the documented `UUID(str)`
# constructor str round-trip contract + the documented `uuid`
# module hasattr surface), and `math` (the full documented
# `pi` / `e` / `inf` / `nan` / `sqrt` / `pow` / `log` / `log2`
# / `log10` / `exp` / `floor` / `ceil` / `trunc` / `gcd` /
# `lcm` / `factorial` / `comb` / `perm` / `fabs` / `isinf` /
# `isnan` / `isfinite` / `modf` / `fmod` / `copysign` value
# contract + the documented math module hasattr surface).
#
# The matching subset between mamba and CPython is the full
# `hashlib` md5 / sha1 / sha256 / sha512 / sha224 / sha384
# hexdigest layer + new factory + digest_size attribute +
# module hasattr surface (md5 / sha1 / sha256 / sha512 /
# sha224 / sha384 / blake2s / new / algorithms_available /
# algorithms_guaranteed — blake2b hexdigest exact-value layer
# DIVERGES), the full `hmac` layer (new + hexdigest + compare_
# digest + HMAC + hasattr surface), the full `secrets` token-
# length / randbelow / choice / hasattr surface, the `random`
# shape contract layer (random in [0,1) / randint in range /
# choice picks from sequence / sample produces k elements
# from population / shuffle returns same sorted list /
# uniform in range) + partial random hasattr surface
# (random / seed / randint / choice / sample / shuffle /
# uniform / Random / gauss — `SystemRandom` DIVERGES + the
# seeded-reproducibility value layer DIVERGES), the `uuid.
# UUID(str)` constructor str round-trip layer + uuid module
# hasattr surface (uuid1 / uuid3 / uuid4 / uuid5 / UUID /
# NAMESPACE_DNS / NAMESPACE_URL — UUID instance .int / .version
# layer + uuid4 instance-class identity DIVERGES), and the
# full `math` value layer (constants + trig + log/exp /
# floor/ceil/trunc / gcd/lcm/factorial / comb/perm / modf /
# fmod / copysign + full math hasattr surface).
#
# Surface in this fixture:
#   • hashlib.md5 / sha1 / sha256 / sha512 / sha224 / sha384
#     — hexdigest exact-value contract on b"hello";
#   • hashlib.new("sha256", ...) — factory equivalence to
#     sha256;
#   • hashlib.sha256().digest_size — 32-byte digest contract;
#   • hashlib — module hasattr surface (md5 / sha1 / sha256 /
#     sha512 / sha224 / sha384 / blake2s / new /
#     algorithms_available / algorithms_guaranteed);
#   • hmac.new + .hexdigest — keyed MAC value contract on
#     (b"key", b"msg", sha256);
#   • hmac.compare_digest — constant-time string compare;
#   • hmac — module hasattr surface (new / compare_digest /
#     digest / HMAC);
#   • secrets.token_hex(n) / token_bytes(n) — length contract;
#   • secrets.randbelow(n) — in-range integer contract;
#   • secrets.choice(seq) — sequence-element contract;
#   • secrets — module hasattr surface (token_hex /
#     token_bytes / token_urlsafe / randbelow / choice /
#     compare_digest / SystemRandom);
#   • random.random() / random.uniform(a, b) — float-range
#     shape contract;
#   • random.randint(a, b) — int-range shape contract;
#   • random.choice(seq) — sequence-element shape contract;
#   • random.sample(pop, k) — k-element shape contract;
#   • random.shuffle(lst) — in-place permutation shape
#     contract (sorted-equality);
#   • random — partial module hasattr surface (random / seed
#     / randint / choice / sample / shuffle / uniform /
#     Random / gauss — `SystemRandom` DIVERGES);
#   • uuid.UUID(str) — str round-trip contract;
#   • uuid — module hasattr surface (uuid1 / uuid3 / uuid4 /
#     uuid5 / UUID / NAMESPACE_DNS / NAMESPACE_URL);
#   • math — pi / e / inf / nan constants + isinf / isnan /
#     sqrt / pow / log / log2 / log10 / exp / floor / ceil /
#     trunc / gcd / lcm / factorial / comb / perm / fabs /
#     copysign / modf / fmod value contract;
#   • math — module hasattr surface (pi / e / inf / nan / tau
#     / sqrt / pow / log / log2 / log10 / exp / floor / ceil
#     / trunc / gcd / lcm / factorial / comb / perm / fabs /
#     isinf / isnan / isfinite / modf / fmod / copysign / sin
#     / cos / tan / asin / acos / atan / atan2 / degrees /
#     radians).
#
# Behavioral edges that DIVERGE on mamba (hashlib.blake2b
# returns a different hexdigest than CPython — algorithm
# implementation diverges, type(uuid.uuid4()).__name__ ==
# "int" not "UUID" — uuid4 produces a bare int not the
# documented UUID instance, uuid.UUID(s).int / .version
# diverge — instance attribute surface broken,
# random.seed(42); random.random() produces a different float
# than CPython — PRNG reproducibility contract broken,
# hasattr(random, "SystemRandom") is False — documented class
# identifier missing) are covered in the matching spec
# fixture `lang_blake2b_uuid_randomseed_silent`.
import hashlib
import hmac
import secrets
import random
import uuid
import math


_ledger: list[int] = []

# 1) hashlib — md5 / sha1 / sha256 / sha512 / sha224 / sha384
#    hexdigest on b"hello"
assert hashlib.md5(b"hello").hexdigest() == "5d41402abc4b2a76b9719d911017c592"; _ledger.append(1)
assert hashlib.sha1(b"hello").hexdigest() == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"; _ledger.append(1)
assert hashlib.sha256(b"hello").hexdigest() == "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"; _ledger.append(1)
assert hashlib.sha512(b"hello").hexdigest()[:16] == "9b71d224bd62f378"; _ledger.append(1)
assert hashlib.sha224(b"hello").hexdigest()[:16] == "ea09ae9cc6768c50"; _ledger.append(1)
assert hashlib.sha384(b"hello").hexdigest()[:16] == "59e1748777448c69"; _ledger.append(1)

# 2) hashlib.new + digest_size
assert hashlib.new("sha256", b"hello").hexdigest()[:16] == "2cf24dba5fb0a30e"; _ledger.append(1)
assert hashlib.sha256().digest_size == 32; _ledger.append(1)

# 3) hashlib — module hasattr surface
assert hasattr(hashlib, "md5") == True; _ledger.append(1)
assert hasattr(hashlib, "sha1") == True; _ledger.append(1)
assert hasattr(hashlib, "sha256") == True; _ledger.append(1)
assert hasattr(hashlib, "sha512") == True; _ledger.append(1)
assert hasattr(hashlib, "sha224") == True; _ledger.append(1)
assert hasattr(hashlib, "sha384") == True; _ledger.append(1)
assert hasattr(hashlib, "blake2s") == True; _ledger.append(1)
assert hasattr(hashlib, "new") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_available") == True; _ledger.append(1)
assert hasattr(hashlib, "algorithms_guaranteed") == True; _ledger.append(1)

# 4) hmac.new + .hexdigest — keyed MAC
_h = hmac.new(b"key", b"msg", hashlib.sha256)
assert _h.hexdigest()[:16] == "2d93cbc1be167bcb"; _ledger.append(1)

# 5) hmac.compare_digest — constant-time compare
assert hmac.compare_digest("abc", "abc") == True; _ledger.append(1)
assert hmac.compare_digest("abc", "xyz") == False; _ledger.append(1)

# 6) hmac — module hasattr surface
assert hasattr(hmac, "new") == True; _ledger.append(1)
assert hasattr(hmac, "compare_digest") == True; _ledger.append(1)
assert hasattr(hmac, "HMAC") == True; _ledger.append(1)

# 7) secrets.token_hex / token_bytes — length contract
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)

# 8) secrets.randbelow — in-range integer
_r = secrets.randbelow(100)
assert _r >= 0; _ledger.append(1)
assert _r < 100; _ledger.append(1)

# 9) secrets.choice — sequence-element
assert secrets.choice([1, 2, 3]) in [1, 2, 3]; _ledger.append(1)

# 10) secrets — module hasattr surface
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)
assert hasattr(secrets, "SystemRandom") == True; _ledger.append(1)

# 11) random.random / uniform — float-range shape contract
_f = random.random()
assert _f >= 0.0; _ledger.append(1)
assert _f < 1.0; _ledger.append(1)
_u = random.uniform(0, 1)
assert _u >= 0.0; _ledger.append(1)
assert _u <= 1.0; _ledger.append(1)

# 12) random.randint — int-range shape contract
_i = random.randint(1, 10)
assert _i >= 1; _ledger.append(1)
assert _i <= 10; _ledger.append(1)

# 13) random.choice / sample / shuffle — shape contracts
assert random.choice([1, 2, 3]) in [1, 2, 3]; _ledger.append(1)
_s = random.sample([1, 2, 3, 4, 5], 2)
assert len(_s) == 2; _ledger.append(1)
_lst = [1, 2, 3, 4, 5]
random.shuffle(_lst)
assert sorted(_lst) == [1, 2, 3, 4, 5]; _ledger.append(1)
assert len(_lst) == 5; _ledger.append(1)

# 14) random — partial module hasattr surface
#     (SystemRandom DIVERGES — moved to spec fixture)
assert hasattr(random, "random") == True; _ledger.append(1)
assert hasattr(random, "seed") == True; _ledger.append(1)
assert hasattr(random, "randint") == True; _ledger.append(1)
assert hasattr(random, "choice") == True; _ledger.append(1)
assert hasattr(random, "sample") == True; _ledger.append(1)
assert hasattr(random, "shuffle") == True; _ledger.append(1)
assert hasattr(random, "uniform") == True; _ledger.append(1)
assert hasattr(random, "Random") == True; _ledger.append(1)
assert hasattr(random, "gauss") == True; _ledger.append(1)

# 15) uuid.UUID — str round-trip contract
_u1 = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert str(_u1) == "12345678-1234-5678-1234-567812345678"; _ledger.append(1)
assert _u1.hex == "12345678123456781234567812345678"; _ledger.append(1)

# 16) uuid — module hasattr surface
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_URL") == True; _ledger.append(1)

# 17) math — pi / e / inf / nan constants
assert math.pi > 3.14; _ledger.append(1)
assert math.pi < 3.15; _ledger.append(1)
assert math.e > 2.71; _ledger.append(1)
assert math.e < 2.72; _ledger.append(1)
assert math.isinf(math.inf) == True; _ledger.append(1)
assert math.isnan(math.nan) == True; _ledger.append(1)
assert math.isfinite(1.0) == True; _ledger.append(1)
assert math.isfinite(math.inf) == False; _ledger.append(1)

# 18) math — sqrt / pow / log / log2 / log10 / exp
assert math.sqrt(16) == 4.0; _ledger.append(1)
assert math.sqrt(2) > 1.41; _ledger.append(1)
assert math.pow(2, 10) == 1024.0; _ledger.append(1)
assert math.log(math.e) == 1.0; _ledger.append(1)
assert math.log2(8) == 3.0; _ledger.append(1)
assert math.log10(100) == 2.0; _ledger.append(1)
assert math.exp(0) == 1.0; _ledger.append(1)

# 19) math — floor / ceil / trunc
assert math.floor(1.7) == 1; _ledger.append(1)
assert math.floor(-1.2) == -2; _ledger.append(1)
assert math.ceil(1.2) == 2; _ledger.append(1)
assert math.ceil(-1.7) == -1; _ledger.append(1)
assert math.trunc(1.7) == 1; _ledger.append(1)
assert math.trunc(-1.7) == -1; _ledger.append(1)

# 20) math — gcd / lcm / factorial / comb / perm
assert math.gcd(12, 8) == 4; _ledger.append(1)
assert math.gcd(17, 13) == 1; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.factorial(0) == 1; _ledger.append(1)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)

# 21) math — fabs / copysign / modf / fmod
assert math.fabs(-3.5) == 3.5; _ledger.append(1)
assert math.fabs(3.5) == 3.5; _ledger.append(1)
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.copysign(-3, 1) == 3.0; _ledger.append(1)
assert math.modf(3.5) == (0.5, 3.0); _ledger.append(1)
assert math.fmod(10, 3) == 1.0; _ledger.append(1)

# 22) math — module hasattr surface
assert hasattr(math, "pi") == True; _ledger.append(1)
assert hasattr(math, "e") == True; _ledger.append(1)
assert hasattr(math, "inf") == True; _ledger.append(1)
assert hasattr(math, "nan") == True; _ledger.append(1)
assert hasattr(math, "tau") == True; _ledger.append(1)
assert hasattr(math, "sqrt") == True; _ledger.append(1)
assert hasattr(math, "pow") == True; _ledger.append(1)
assert hasattr(math, "log") == True; _ledger.append(1)
assert hasattr(math, "log2") == True; _ledger.append(1)
assert hasattr(math, "log10") == True; _ledger.append(1)
assert hasattr(math, "exp") == True; _ledger.append(1)
assert hasattr(math, "floor") == True; _ledger.append(1)
assert hasattr(math, "ceil") == True; _ledger.append(1)
assert hasattr(math, "trunc") == True; _ledger.append(1)
assert hasattr(math, "gcd") == True; _ledger.append(1)
assert hasattr(math, "lcm") == True; _ledger.append(1)
assert hasattr(math, "factorial") == True; _ledger.append(1)
assert hasattr(math, "comb") == True; _ledger.append(1)
assert hasattr(math, "perm") == True; _ledger.append(1)
assert hasattr(math, "fabs") == True; _ledger.append(1)
assert hasattr(math, "isinf") == True; _ledger.append(1)
assert hasattr(math, "isnan") == True; _ledger.append(1)
assert hasattr(math, "isfinite") == True; _ledger.append(1)
assert hasattr(math, "modf") == True; _ledger.append(1)
assert hasattr(math, "fmod") == True; _ledger.append(1)
assert hasattr(math, "copysign") == True; _ledger.append(1)
assert hasattr(math, "sin") == True; _ledger.append(1)
assert hasattr(math, "cos") == True; _ledger.append(1)
assert hasattr(math, "tan") == True; _ledger.append(1)
assert hasattr(math, "asin") == True; _ledger.append(1)
assert hasattr(math, "acos") == True; _ledger.append(1)
assert hasattr(math, "atan") == True; _ledger.append(1)
assert hasattr(math, "atan2") == True; _ledger.append(1)
assert hasattr(math, "degrees") == True; _ledger.append(1)
assert hasattr(math, "radians") == True; _ledger.append(1)

# NB: hashlib.blake2b hexdigest diverges on mamba vs cpython,
# type(uuid.uuid4()).__name__ == "int" not "UUID", uuid.UUID(s)
# .int / .version surface broken on mamba, random.seed(42);
# random() produces different float on mamba (PRNG state diverges),
# hasattr(random, "SystemRandom") is False on mamba — all
# DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_hashlib_hmac_secrets_math_value_ops {sum(_ledger)} asserts")
