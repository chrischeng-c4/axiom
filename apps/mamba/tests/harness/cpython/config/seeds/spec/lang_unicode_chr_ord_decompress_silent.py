# Spec seed for CPython TypeError / ValueError / UnicodeError /
# OSError / LZMAError contract on the unicode / chr-ord / compressed-
# blob corners that mamba silently coerces. Surface: CPython rejects
# (1) `ord(empty_or_multichar)` because `ord()` requires exactly one
# code point — TypeError, not silent `None`; (2) `chr(n)` for
# `n >= 0x110000` because that exceeds the Unicode max codepoint —
# ValueError, not silent `None`; (3) `s.encode("ascii")` /
# `b.decode("ascii")` / `b.decode("utf-8")` when bytes/codepoints lie
# outside the codec's range — UnicodeEncodeError / UnicodeDecodeError,
# not silent passthrough that drops or `?`-fills the offending
# character; (4) `bz2.decompress(non_bz2_blob)` and
# `lzma.decompress(non_lzma_blob)` because the magic bytes don't
# match the codec — OSError / LZMAError, not silent `b''`.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • ord("")                       → mamba: None     (TypeError)
#   • ord("ab")                     → mamba: None     (TypeError)
#   • ord("abc")                    → mamba: None     (TypeError)
#   • chr(0x110000)                 → mamba: None     (ValueError)
#   • chr(0x200000)                 → mamba: None     (ValueError)
#   • chr(0x7FFFFFFF)               → mamba: None     (ValueError)
#   • "héllo".encode("ascii")       → mamba: b'h?llo' (UnicodeEncodeError)
#   • "中文".encode("ascii")        → mamba: b'??'    (UnicodeEncodeError)
#   • b"\xff\xfe".decode("ascii")   → mamba: '�' (UnicodeDecodeError)
#   • b"\x80\x81".decode("utf-8")   → mamba: '�' (UnicodeDecodeError)
#   • bz2.decompress(b"not_bz2_at_all")
#                                   → mamba: b''      (OSError)
#   • lzma.decompress(b"not_lzma_at_all")
#                                   → mamba: b''      (LZMAError)
#
# CPython contract:
#   ord(empty)             → TypeError("ord() expected a character,
#                                  but string of length 0 found");
#   ord(multichar)         → TypeError("ord() expected a character,
#                                  but string of length N found");
#   chr(>=0x110000)        → ValueError("chr() arg not in range
#                                  (0x110000)");
#   str.encode("ascii")    → UnicodeEncodeError("'ascii' codec can't
#                                  encode character ... in position …");
#   bytes.decode("ascii")
#   bytes.decode("utf-8")  → UnicodeDecodeError("'ascii' / 'utf-8'
#                                  codec can't decode byte ... in
#                                  position …");
#   bz2.decompress(garbage)
#                          → OSError("Invalid data stream");
#   lzma.decompress(garbage)
#                          → LZMAError("Input format not supported by
#                                  decoder").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
import bz2
import lzma
_ledger: list[int] = []

_empty: Any = ""
_double: Any = "ab"
_triple: Any = "abc"
_over1: Any = 0x110000
_over2: Any = 0x200000
_over3: Any = 0x7FFFFFFF
_high_lat: Any = "héllo"
_high_cjk: Any = "中文"
_bad_ascii: Any = b"\xff\xfe"
_bad_utf8: Any = b"\x80\x81"
_garbage_bz2: Any = b"definitely_not_bz2_data_payload_here"
_garbage_lzma: Any = b"definitely_not_lzma_data_payload_here"

# ord("") — empty string is not a single code point
try:
    _ = ord(_empty)
    raise AssertionError("ord('') must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord("ab") — two-char string is not a single code point
try:
    _ = ord(_double)
    raise AssertionError("ord('ab') must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord("abc") — three-char string is not a single code point
try:
    _ = ord(_triple)
    raise AssertionError("ord('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# chr(0x110000) — exactly above the Unicode max
try:
    _ = chr(_over1)
    raise AssertionError("chr(0x110000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(0x200000) — well above max
try:
    _ = chr(_over2)
    raise AssertionError("chr(0x200000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# chr(0x7FFFFFFF) — near int32 max, far above unicode max
try:
    _ = chr(_over3)
    raise AssertionError("chr(0x7FFFFFFF) must raise ValueError")
except ValueError:
    _ledger.append(1)

# "héllo".encode("ascii") — Latin-1 codepoint outside ascii range
try:
    _ = _high_lat.encode("ascii")
    raise AssertionError("'héllo'.encode('ascii') must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# "中文".encode("ascii") — CJK codepoints outside ascii range
try:
    _ = _high_cjk.encode("ascii")
    raise AssertionError("'中文'.encode('ascii') must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# b"\xff\xfe".decode("ascii") — bytes above 0x7f aren't ascii
try:
    _ = _bad_ascii.decode("ascii")
    raise AssertionError("b'\\xff\\xfe'.decode('ascii') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# b"\x80\x81".decode("utf-8") — invalid utf-8 lead bytes
try:
    _ = _bad_utf8.decode("utf-8")
    raise AssertionError("b'\\x80\\x81'.decode('utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# bz2.decompress(garbage) — magic bytes don't match
try:
    _ = bz2.decompress(_garbage_bz2)
    raise AssertionError("bz2.decompress(garbage) must raise OSError")
except OSError:
    _ledger.append(1)

# lzma.decompress(garbage) — magic bytes don't match
try:
    _ = lzma.decompress(_garbage_lzma)
    raise AssertionError("lzma.decompress(garbage) must raise LZMAError")
except lzma.LZMAError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_unicode_chr_ord_decompress_silent {sum(_ledger)} asserts")
