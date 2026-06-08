# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_random_enum_hmac_class_silent"
# subject = "cpython321.lang_random_enum_hmac_class_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_random_enum_hmac_class_silent.py"
# status = "filled"
# ///
"""cpython321.lang_random_enum_hmac_class_silent: execute CPython 3.12 seed lang_random_enum_hmac_class_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across the
# crypto / random / enum quartet pinned by atomic 145: `hashlib`
# (the documented `set` container type of `algorithms_guaranteed`
# — mamba returns a `list`), `hmac` (the HMAC class identity),
# `random` (the Random / SystemRandom bare class identity plus
# the documented seed-deterministic output for random / randint /
# uniform), and `enum` (the Enum / IntEnum / Flag / IntFlag bare
# class identity, the Enum-subclass `.name` / `.value` member
# surface, and the `enum.auto` helper).
#
# The matching subset (keyword.iskeyword / kwlist / softkwlist,
# hashlib md5 / sha1 / sha256 / sha512 byte-exact hexdigest +
# digest_size + block_size + name, hmac.new hexdigest + digest_size
# + block_size + name, hmac.compare_digest, secrets.token_bytes /
# token_hex / token_urlsafe / randbelow / choice surface) is
# covered by `test_keyword_hashlib_hmac_secrets_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • type(hashlib.algorithms_guaranteed).__name__ == "set" —
#     documented as a `set` of guaranteed algorithm names
#     (mamba: returns "list");
#   • hmac.HMAC.__name__ == "HMAC" — RFC 2104 class identity
#     (mamba: returns None);
#   • random.random() with seed(42) == 0.6394267984578837 —
#     Mersenne Twister deterministic output (mamba: returns
#     0.37454, a DIFFERENT deterministic value because mamba
#     uses a non-Mersenne-Twister RNG);
#   • random.randint(1, 100) with seed(42) == 82 (mamba: 100);
#   • random.uniform(0, 1) with seed(42) ==
#     0.6394267984578837 (mamba: 0.37454);
#   • random.Random.__name__ == "Random" — bare class identity
#     (mamba: returns None);
#   • random.SystemRandom.__name__ == "SystemRandom" (mamba:
#     None);
#   • enum.Enum.__name__ == "Enum" — bare class identity (mamba:
#     None);
#   • enum.IntEnum.__name__ == "IntEnum" (mamba: None);
#   • enum.Flag.__name__ == "Flag" (mamba: None);
#   • enum.IntFlag.__name__ == "IntFlag" (mamba: None);
#   • Color.RED.name == "RED" — Enum-subclass member surface
#     (mamba: returns None);
#   • Color.RED.value == 1 (mamba: None);
#   • type(enum.auto()).__name__ == "auto" — auto sentinel
#     constructor (mamba: AttributeError on `enum.auto`).
import hashlib as _hashlib_mod
import hmac as _hmac_mod
import random as _random_mod
import enum as _enum_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constants / class identifiers / instance members that mamba's
# bundled type stubs do not surface accurately.
hashlib: Any = _hashlib_mod
hmac: Any = _hmac_mod
random: Any = _random_mod
enum: Any = _enum_mod


# Enum subclass kept at module scope to dodge the documented
# mamba quirk where a `class` defined inside a `try:` block isn't
# visible to the next statement.
class Color(_enum_mod.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3


_ledger: list[int] = []

# 1) hashlib.algorithms_guaranteed — documented `set` container
assert type(hashlib.algorithms_guaranteed).__name__ == "set"; _ledger.append(1)

# 2) hmac.HMAC — class identity
assert hmac.HMAC.__name__ == "HMAC"; _ledger.append(1)

# 3) random.random — Mersenne Twister deterministic output
random.seed(42)
assert random.random() == 0.6394267984578837; _ledger.append(1)

# 4) random.randint — deterministic int in [1, 100]
random.seed(42)
assert random.randint(1, 100) == 82; _ledger.append(1)

# 5) random.uniform — deterministic float in [0, 1]
random.seed(42)
assert random.uniform(0, 1) == 0.6394267984578837; _ledger.append(1)

# 6) random — bare class identity
assert random.Random.__name__ == "Random"; _ledger.append(1)
assert random.SystemRandom.__name__ == "SystemRandom"; _ledger.append(1)

# 7) enum — bare class identity
assert enum.Enum.__name__ == "Enum"; _ledger.append(1)
assert enum.IntEnum.__name__ == "IntEnum"; _ledger.append(1)
assert enum.Flag.__name__ == "Flag"; _ledger.append(1)
assert enum.IntFlag.__name__ == "IntFlag"; _ledger.append(1)

# 8) Enum-subclass member surface
assert Color.RED.name == "RED"; _ledger.append(1)
assert Color.RED.value == 1; _ledger.append(1)
assert Color.GREEN.name == "GREEN"; _ledger.append(1)
assert Color.GREEN.value == 2; _ledger.append(1)
assert Color.BLUE.name == "BLUE"; _ledger.append(1)
assert Color.BLUE.value == 3; _ledger.append(1)

# 9) enum.auto — sentinel constructor
assert type(enum.auto()).__name__ == "auto"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_random_enum_hmac_class_silent {sum(_ledger)} asserts")
