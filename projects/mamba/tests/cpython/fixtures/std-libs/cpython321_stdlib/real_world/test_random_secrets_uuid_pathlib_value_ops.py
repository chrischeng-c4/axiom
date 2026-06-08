# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_random_secrets_uuid_pathlib_value_ops"
# subject = "cpython321.test_random_secrets_uuid_pathlib_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_random_secrets_uuid_pathlib_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_random_secrets_uuid_pathlib_value_ops: execute CPython 3.12 seed test_random_secrets_uuid_pathlib_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 257 pass conformance — random module (hasattr surface
# random/randint/choice/shuffle/sample/seed/randrange/uniform/gauss/
# Random/getrandbits/choices + seeded random() ∈ [0,1), randint
# bounded inclusive, choice picks from list, shuffle preserves
# multiset, sample length, randrange upper bound) + secrets module
# (hasattr surface token_bytes/token_hex/token_urlsafe/choice/
# randbelow/compare_digest + token_bytes(16) length 16, token_hex(8)
# length 16, randbelow upper bound, compare_digest True for equal /
# False for differing strings) + uuid module (hasattr surface
# uuid1/uuid3/uuid4/uuid5/UUID/NAMESPACE_DNS + uuid4().hex length 32,
# uuid4().hex contains only lowercase hex chars) + pathlib module
# (hasattr surface Path/PurePath/PurePosixPath/PureWindowsPath/
# PosixPath). All asserts match between CPython 3.12 and mamba.
import random
import secrets
import uuid
import pathlib


_ledger: list[int] = []

# 1) random — hasattr surface
assert hasattr(random, "random") == True; _ledger.append(1)
assert hasattr(random, "randint") == True; _ledger.append(1)
assert hasattr(random, "choice") == True; _ledger.append(1)
assert hasattr(random, "shuffle") == True; _ledger.append(1)
assert hasattr(random, "sample") == True; _ledger.append(1)
assert hasattr(random, "seed") == True; _ledger.append(1)
assert hasattr(random, "randrange") == True; _ledger.append(1)
assert hasattr(random, "uniform") == True; _ledger.append(1)
assert hasattr(random, "gauss") == True; _ledger.append(1)
assert hasattr(random, "Random") == True; _ledger.append(1)
assert hasattr(random, "getrandbits") == True; _ledger.append(1)
assert hasattr(random, "choices") == True; _ledger.append(1)

# 2) random — seeded random() ∈ [0, 1)
random.seed(42)
r = random.random()
assert (0.0 <= r) == True; _ledger.append(1)
assert (r < 1.0) == True; _ledger.append(1)

# 3) random — randint bounded inclusive
random.seed(42)
ri = random.randint(1, 10)
assert (1 <= ri) == True; _ledger.append(1)
assert (ri <= 10) == True; _ledger.append(1)

# 4) random — choice picks from list
random.seed(42)
assert random.choice([1, 2, 3, 4, 5]) in [1, 2, 3, 4, 5]; _ledger.append(1)

# 5) random — shuffle preserves multiset
random.seed(42)
def _shuffle_preserves() -> list:
    lst = [1, 2, 3, 4, 5]
    random.shuffle(lst)
    return sorted(lst)
assert _shuffle_preserves() == [1, 2, 3, 4, 5]; _ledger.append(1)

# 6) random — sample length
random.seed(42)
assert len(random.sample([1, 2, 3, 4, 5], 3)) == 3; _ledger.append(1)

# 7) random — randrange upper bound
random.seed(42)
rr = random.randrange(10)
assert (0 <= rr) == True; _ledger.append(1)
assert (rr < 10) == True; _ledger.append(1)

# 8) secrets — hasattr surface
assert hasattr(secrets, "token_bytes") == True; _ledger.append(1)
assert hasattr(secrets, "token_hex") == True; _ledger.append(1)
assert hasattr(secrets, "token_urlsafe") == True; _ledger.append(1)
assert hasattr(secrets, "choice") == True; _ledger.append(1)
assert hasattr(secrets, "randbelow") == True; _ledger.append(1)
assert hasattr(secrets, "compare_digest") == True; _ledger.append(1)

# 9) secrets — token byte/hex length
assert len(secrets.token_bytes(16)) == 16; _ledger.append(1)
assert len(secrets.token_hex(8)) == 16; _ledger.append(1)

# 10) secrets — randbelow upper bound
rb = secrets.randbelow(100)
assert (0 <= rb) == True; _ledger.append(1)
assert (rb < 100) == True; _ledger.append(1)

# 11) secrets — compare_digest equal / differ
assert secrets.compare_digest("abc", "abc") == True; _ledger.append(1)
assert secrets.compare_digest("abc", "xyz") == False; _ledger.append(1)

# 12) uuid — hasattr surface
assert hasattr(uuid, "uuid1") == True; _ledger.append(1)
assert hasattr(uuid, "uuid3") == True; _ledger.append(1)
assert hasattr(uuid, "uuid4") == True; _ledger.append(1)
assert hasattr(uuid, "uuid5") == True; _ledger.append(1)
assert hasattr(uuid, "UUID") == True; _ledger.append(1)
assert hasattr(uuid, "NAMESPACE_DNS") == True; _ledger.append(1)

# 13) uuid — uuid4().hex length and hex chars
u = uuid.uuid4()
assert len(u.hex) == 32; _ledger.append(1)
assert all(c in "0123456789abcdef" for c in u.hex) == True; _ledger.append(1)

# 14) pathlib — hasattr surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PosixPath") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_random_secrets_uuid_pathlib_value_ops {sum(_ledger)} asserts")
