# Spec seed for CPython index-out-of-range (IndexError), missing-key
# (KeyError), invalid-ord/chr-arity (TypeError), out-of-range chr
# (ValueError), negative-shift (ValueError), non-int slice-index
# (TypeError), and non-encodable / invalid-decode codec errors
# (UnicodeEncodeError / UnicodeDecodeError). Surface: CPython rejects
# every form below with the indicated subclass of Exception; mamba
# 0.3.60 silently returns `None`, `0.0`, the original list, a `'?'`,
# or a replacement-char string. Existing lang_typeerror_* /
# lang_valueerror_* / lang_overflowerror_* / lang_format_attr_codec_*
# seeds cover binary arithmetic / call-arity / iter-required /
# numeric-conversion / format-code / attr-name / unhashable / unknown
# codec angles; this seed adds the
# index-out-of-range / del-missing-key / ord-arity / chr-out-of-range
# / negative-shift / non-int-slice / unencodable-char / invalid-decode
# corners.
#
# Probes (CPython raises the indicated exception; mamba silently
# returns a wrong-shape value):
#   • ()[0]                            → mamba: None  (IndexError)
#   • (1, 2, 3)[5]                     → mamba: None  (IndexError)
#   • ''[0]                            → mamba: None  (IndexError)
#   • b''[0]                           → mamba: None  (IndexError)
#   • del {}['missing']                → mamba: silent (KeyError)
#   • ord('')                          → mamba: None  (TypeError)
#   • ord('ab')                        → mamba: None  (TypeError)
#   • chr(1.5)                         → mamba: None  (TypeError)
#   • chr(0x110000)                    → mamba: None  (ValueError)
#   • 1 << -1                          → mamba: -0.0  (ValueError)
#   • 1 >> -1                          → mamba: 0.0   (ValueError)
#   • [1, 2, 3][::'a']                 → mamba: list  (TypeError)
#   • [1, 2, 3]['a':]                  → mamba: list  (TypeError)
#   • 'é'.encode('ascii')              → mamba: b'?'  (UnicodeEncodeError)
#   • '€'.encode('latin-1')            → mamba: b'?'  (UnicodeEncodeError)
#   • b'\x80'.decode('ascii')          → mamba: '�' (UnicodeDecodeError)
#   • b'\x80'.decode('utf-8')          → mamba: '�' (UnicodeDecodeError)
#   • b'\xc0\xc0'.decode('utf-8')      → mamba: '��' (UnicodeDecodeError)
#
# CPython contract:
#   seq[OOR]               → IndexError("tuple/string/bytes index out
#                                  of range");
#   del d[missing]         → KeyError(missing);
#   ord(non-1-char-str)    → TypeError("ord() expected a character,
#                                  but string of length N found");
#   chr(non_int)           → TypeError("'<typename>' object cannot
#                                  be interpreted as an integer");
#   chr(>= 0x110000)       → ValueError("chr() arg not in
#                                  range(0x110000)");
#   int << neg / int >> neg→ ValueError("negative shift count");
#   seq[non_int_slice]     → TypeError("slice indices must be
#                                  integers or None or have an
#                                  __index__ method");
#   str.encode(bad_enc)    → UnicodeEncodeError("'<codec>' codec
#                                  can't encode character ...");
#   bytes.decode(bad_dec)  → UnicodeDecodeError("'<codec>' codec
#                                  can't decode byte ...").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_empty_tup: Any = ()
_tup3: Any = (1, 2, 3)
_empty_str: Any = ""
_empty_bytes: Any = b""
_idx_big: Any = 5
_empty_dict: Any = {}
_flt: Any = 1.5
_lst3: Any = [1, 2, 3]
_step_str: Any = "a"
_neg_one: Any = -1

# ()[0] — empty tuple index
try:
    _ = _empty_tup[0]
    raise AssertionError("()[0] must raise IndexError")
except IndexError:
    _ledger.append(1)

# (1, 2, 3)[5] — tuple index out of range
try:
    _ = _tup3[_idx_big]
    raise AssertionError("(1, 2, 3)[5] must raise IndexError")
except IndexError:
    _ledger.append(1)

# ''[0] — empty string index
try:
    _ = _empty_str[0]
    raise AssertionError("''[0] must raise IndexError")
except IndexError:
    _ledger.append(1)

# b''[0] — empty bytes index
try:
    _ = _empty_bytes[0]
    raise AssertionError("b''[0] must raise IndexError")
except IndexError:
    _ledger.append(1)

# del {}['missing'] — KeyError on missing key delete
try:
    del _empty_dict["missing"]
    raise AssertionError("del {}['missing'] must raise KeyError")
except KeyError:
    _ledger.append(1)

# ord('') — empty string is not a single char
try:
    _ = ord(_empty_str)
    raise AssertionError("ord('') must raise TypeError")
except TypeError:
    _ledger.append(1)

# ord('ab') — 2-char string is not a single char
_two_char: Any = "ab"
try:
    _ = ord(_two_char)
    raise AssertionError("ord('ab') must raise TypeError")
except TypeError:
    _ledger.append(1)

# chr(1.5) — chr requires int, not float
try:
    _ = chr(_flt)
    raise AssertionError("chr(1.5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# chr(0x110000) — Unicode codepoint out of range
_above_unicode: Any = 0x110000
try:
    _ = chr(_above_unicode)
    raise AssertionError("chr(0x110000) must raise ValueError")
except ValueError:
    _ledger.append(1)

# 1 << -1 — negative shift count
try:
    _ = 1 << _neg_one
    raise AssertionError("1 << -1 must raise ValueError")
except ValueError:
    _ledger.append(1)

# 1 >> -1 — negative shift count
try:
    _ = 1 >> _neg_one
    raise AssertionError("1 >> -1 must raise ValueError")
except ValueError:
    _ledger.append(1)

# [1, 2, 3][::'a'] — slice step must be int / None
try:
    _ = _lst3[::_step_str]
    raise AssertionError("[1, 2, 3][::'a'] must raise TypeError")
except TypeError:
    _ledger.append(1)

# [1, 2, 3]['a':] — slice start must be int / None
try:
    _ = _lst3[_step_str:]
    raise AssertionError("[1, 2, 3]['a':] must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'é'.encode('ascii') — char beyond ASCII range
try:
    _ = "é".encode("ascii")
    raise AssertionError("'é'.encode('ascii') must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# '€'.encode('latin-1') — char beyond Latin-1 range
try:
    _ = "€".encode("latin-1")
    raise AssertionError("'€'.encode('latin-1') must raise UnicodeEncodeError")
except UnicodeEncodeError:
    _ledger.append(1)

# b'\x80'.decode('ascii') — byte beyond ASCII range
try:
    _ = b"\x80".decode("ascii")
    raise AssertionError("b'\\x80'.decode('ascii') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# b'\x80'.decode('utf-8') — continuation byte without lead byte
try:
    _ = b"\x80".decode("utf-8")
    raise AssertionError("b'\\x80'.decode('utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

# b'\xc0\xc0'.decode('utf-8') — invalid lead byte sequence
try:
    _ = b"\xc0\xc0".decode("utf-8")
    raise AssertionError("b'\\xc0\\xc0'.decode('utf-8') must raise UnicodeDecodeError")
except UnicodeDecodeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_index_codec_chr_silent {sum(_ledger)} asserts")
