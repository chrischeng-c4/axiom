# Operational AssertionPass divergence-spec fixture for the
# silent value-contract divergence of the
# `textwrap` / `operator` / `statistics` / `uuid` / `base64` /
# `zlib` / `binascii` / `hashlib` / `shlex` / `string` / `sys`
# / `os` ten-pack pinned to atomic 225:
# `textwrap` (the documented `textwrap.wrap("a b c d", width=3)
# == ["a b", "c d"]` / `textwrap.fill("a b c d", width=3) ==
# "a b\nc d"` / `textwrap.shorten("a b c d e f", width=8) ==
# "a [...]"` value contract — mamba silently returns the input
# unchanged), `operator` (the documented `operator.itemgetter(0)
# ([1,2,3]) == 1` / `operator.attrgetter("real")(1+2j) == 1.0`
# / `operator.methodcaller("upper")("abc") == "ABC"` value
# contract — mamba silently returns None), `statistics` (the
# documented integer-input integer-return contract:
# `statistics.mean([1,2,3]) == 2` and `statistics.variance(
# [1,2,3]) == 1` — mamba silently widens to 2.0 / 1.0),
# `uuid` (the documented `uuid.UUID(int=0).version is None`
# contract — mamba silently returns 0), `zlib` (the
# documented `hasattr(zlib, "compressobj") /
# "decompressobj" / "DEFLATED" / "MAX_WBITS" /
# "Z_DEFAULT_COMPRESSION" / "Z_BEST_COMPRESSION" /
# "Z_BEST_SPEED" / "Z_NO_COMPRESSION" / "Z_FINISH" /
# "Z_FULL_FLUSH" / "Z_PARTIAL_FLUSH" / "Z_SYNC_FLUSH" /
# "Z_NO_FLUSH" / "ZLIB_VERSION" / "ZLIB_RUNTIME_VERSION"
# == True` extended hasattr surface), `binascii` (the
# documented `hasattr(binascii, "b2a_uu") / "a2b_uu" /
# "b2a_qp" / "a2b_qp" / "crc32" / "Error" / "Incomplete" ==
# True` extended hasattr surface), `hashlib` (the documented
# `hasattr(hashlib, "shake_128") / "shake_256" /
# "pbkdf2_hmac" / "scrypt" / "file_digest" == True` extended
# hasattr surface), `shlex` / `string` / `sys` / `os` (small
# attribute-surface gaps: `hasattr(shlex, "shlex")` /
# `hasattr(string, "printable")` / `hasattr(sys, "maxunicode")`
# / `hasattr(os, "fstat")` / `hasattr(textwrap, "TextWrapper")`
# == True).
#
# Behavioral edges that CONFORM on mamba (hashlib sha256/sha1
# /md5 hexdigest, hmac.new/digest/compare_digest, base64
# b64encode/decode + b32encode/decode + b16encode/decode,
# binascii hexlify/unhexlify/b2a_hex/a2b_hex/b2a_base64, zlib
# compress/decompress/adler32/crc32, secrets token sizes,
# uuid hex/uuid4 version/from-str hex/bytes, statistics
# median/median_low/median_high/mode/stdev/pstdev/pvariance/
# harmonic_mean/geometric_mean/fmean, functools reduce/partial
# /lru_cache/cache, operator add/sub/mul/eq/lt/gt/neg/contains
# /getitem, math gcd/lcm/factorial/comb/perm/isqrt/prod/floor
# /ceil/trunc/isnan/isinf/isfinite, struct calcsize/pack hex)
# are covered in the matching pass fixture
# `test_hashlib_hmac_base64_binascii_zlib_value_ops`.
from typing import Any
import textwrap as _textwrap_mod
import operator as _operator_mod
import statistics as _statistics_mod
import uuid as _uuid_mod
import base64 as _base64_mod
import zlib as _zlib_mod
import binascii as _binascii_mod
import hashlib as _hashlib_mod
import shlex as _shlex_mod
import string as _string_mod
import sys as _sys_mod
import os as _os_mod

textwrap: Any = _textwrap_mod
operator: Any = _operator_mod
statistics: Any = _statistics_mod
uuid: Any = _uuid_mod
base64: Any = _base64_mod
zlib: Any = _zlib_mod
binascii: Any = _binascii_mod
hashlib: Any = _hashlib_mod
shlex: Any = _shlex_mod
string: Any = _string_mod
sys: Any = _sys_mod
os: Any = _os_mod


_ledger: list[int] = []

# 1) textwrap — value contract divergence
#    (mamba: wrap/fill/shorten silently return input unchanged)
assert textwrap.wrap("a b c d", width=3) == ["a b", "c d"]; _ledger.append(1)
assert textwrap.fill("a b c d", width=3) == "a b\nc d"; _ledger.append(1)
assert textwrap.shorten("a b c d e f", width=8) == "a [...]"; _ledger.append(1)

# 2) operator — itemgetter/attrgetter/methodcaller value contract
#    (mamba: all three silently return None)
assert operator.itemgetter(0)([1, 2, 3]) == 1; _ledger.append(1)
assert operator.attrgetter("real")(1 + 2j) == 1.0; _ledger.append(1)
assert operator.methodcaller("upper")("abc") == "ABC"; _ledger.append(1)

# 3) statistics — integer-input integer-return contract
#    (mamba: silently widens to float)
assert statistics.mean([1, 2, 3]) == 2; _ledger.append(1)
assert type(statistics.mean([1, 2, 3])).__name__ == "int"; _ledger.append(1)
assert statistics.variance([1, 2, 3]) == 1; _ledger.append(1)
assert type(statistics.variance([1, 2, 3])).__name__ == "int"; _ledger.append(1)

# 4) uuid — UUID(int=0).version is None contract
#    (mamba: silently returns 0)
_u0 = uuid.UUID(int=0)
assert _u0.version is None; _ledger.append(1)

# 5) zlib — extended module hasattr surface
#    (mamba: compressobj / decompressobj / DEFLATED /
#    MAX_WBITS / Z_DEFAULT_COMPRESSION / Z_BEST_COMPRESSION /
#    Z_BEST_SPEED / Z_NO_COMPRESSION / Z_FINISH /
#    Z_FULL_FLUSH / Z_PARTIAL_FLUSH / Z_SYNC_FLUSH /
#    Z_NO_FLUSH / ZLIB_VERSION / ZLIB_RUNTIME_VERSION
#    all False)
assert hasattr(zlib, "compressobj") == True; _ledger.append(1)
assert hasattr(zlib, "decompressobj") == True; _ledger.append(1)
assert hasattr(zlib, "DEFLATED") == True; _ledger.append(1)
assert hasattr(zlib, "MAX_WBITS") == True; _ledger.append(1)
assert hasattr(zlib, "Z_DEFAULT_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_BEST_SPEED") == True; _ledger.append(1)
assert hasattr(zlib, "Z_NO_COMPRESSION") == True; _ledger.append(1)
assert hasattr(zlib, "Z_FINISH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_FULL_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_PARTIAL_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_SYNC_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "Z_NO_FLUSH") == True; _ledger.append(1)
assert hasattr(zlib, "ZLIB_VERSION") == True; _ledger.append(1)
assert hasattr(zlib, "ZLIB_RUNTIME_VERSION") == True; _ledger.append(1)

# 6) binascii — extended module hasattr surface
#    (mamba: b2a_uu / a2b_uu / b2a_qp / a2b_qp / crc32 /
#    Error / Incomplete all False)
assert hasattr(binascii, "b2a_uu") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_uu") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_qp") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_qp") == True; _ledger.append(1)
assert hasattr(binascii, "crc32") == True; _ledger.append(1)
assert hasattr(binascii, "Error") == True; _ledger.append(1)
assert hasattr(binascii, "Incomplete") == True; _ledger.append(1)

# 7) hashlib — extended module hasattr surface
#    (mamba: shake_128 / shake_256 / pbkdf2_hmac / scrypt /
#    file_digest all False)
assert hasattr(hashlib, "shake_128") == True; _ledger.append(1)
assert hasattr(hashlib, "shake_256") == True; _ledger.append(1)
assert hasattr(hashlib, "pbkdf2_hmac") == True; _ledger.append(1)
assert hasattr(hashlib, "scrypt") == True; _ledger.append(1)
assert hasattr(hashlib, "file_digest") == True; _ledger.append(1)

# 8) shlex / string / sys / os / textwrap — small attribute-
#    surface gaps
assert hasattr(shlex, "shlex") == True; _ledger.append(1)
assert hasattr(string, "printable") == True; _ledger.append(1)
assert hasattr(sys, "maxunicode") == True; _ledger.append(1)
assert hasattr(os, "fstat") == True; _ledger.append(1)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_textwrap_operator_statistics_silent {sum(_ledger)} asserts")
