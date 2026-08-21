# Spec seed for CPython TypeError contract on the codec / numeric-
# conversion argument-type-routing corners where mamba silently
# coerces the wrong-typed encoding / base / target to a default
# (`b'abc'` / `'abc'` / `b''` / `None` / `10` / `0` / `0.0`)
# instead of raising the canonical TypeError.
#
# Surface: CPython rejects (1) `str.encode(non_str_encoding)` /
# `bytes.decode(non_str_encoding)` because the encoding parameter
# must be a real `str` — TypeError("encode()/decode() argument
# 'encoding' must be str, not <type>"); (2) `str.encode('utf-8',
# non_str_errors)` because the `errors` parameter must also be `str`
# — TypeError("encode() argument 'errors' must be str, not <type>");
# (3) `bytes.fromhex(non_str)` because the argument must be `str`
# — TypeError("fromhex() argument must be str, not <type>"); (4)
# `int(s, non_int_base)` because the base parameter routes through
# `__index__` and floats / None / list don't implement it (and `str`
# base is covered by lang_typeerror_str_bytes_method_silent.py) —
# TypeError("'<type>' object cannot be interpreted as an integer");
# (5) `bin(non_int)` / `hex(non_int)` / `oct(non_int)` because the
# argument routes through `__index__` and float / str don't
# implement it — TypeError("'<type>' object cannot be interpreted
# as an integer"); (6) `float(non_str_non_num)` because the
# argument must be a string or a real number — TypeError("float()
# argument must be a string or a real number, not '<type>'").
#
# Mamba accepts every form and silently:
#   - returns the SOURCE string/bytes for `encode(non_str_enc)` /
#     `decode(non_str_enc)` / `encode(_, non_str_errors)` — masking
#     the operator's intent to control encoding;
#   - returns empty bytes `b''` for `bytes.fromhex(non_str)`,
#     making `bytes.fromhex(maybe_int_payload)` silently return an
#     empty buffer instead of the canonical "argument must be str";
#   - returns the parsed-base-10 result for `int(s, non_int_base)`
#     (so `int('10', None) == 10`), masking the operator's intent
#     to interpret in a non-decimal base;
#   - returns `None` for `bin/hex/oct(non_int)`, propagating
#     downstream as a "NoneType is not subscriptable" error rather
#     than the canonical "object cannot be interpreted as integer";
#   - returns `0` / `0.0` for `float(non_str_non_num)`, masking
#     a wrong-payload upstream bug as a clean zero.
#
# Existing lang_encode_decode_replace_silent.py covers
# `str.encode(VALID_str_enc)` with invalid characters and
# `bytes.decode(VALID_str_enc)` with invalid bytes — the CODEC
# error paths. Existing lang_typeerror_str_bytes_method_silent.py
# covers `int(str, str_base)` — the BASE-IS-STR variant. Existing
# lang_format_complex_fromhex_silent.py covers `bytes.fromhex(non_hex_str)`
# — the VALUE-error path. Existing lang_typeerror_bytes_complex_conv_silent.py
# covers `int(complex)` / `float(complex)`. This seed covers the
# FRESH divergence family — codec encoding-arg / errors-arg / base-arg /
# bin/hex/oct-arg / float-arg WRONG-TYPE checks (where the argument
# type is the violation, not the value content).
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • 'abc'.encode(5)              → mamba: b'abc'  (TypeError)
#   • 'abc'.encode(None)           → mamba: b'abc'  (TypeError)
#   • 'abc'.encode([1,2])          → mamba: b'abc'  (TypeError)
#   • b'abc'.decode(5)             → mamba: 'abc'   (TypeError)
#   • b'abc'.decode(None)          → mamba: 'abc'   (TypeError)
#   • b'abc'.decode([1,2])         → mamba: 'abc'   (TypeError)
#   • 'abc'.encode('utf-8', 5)     → mamba: b'abc'  (TypeError)
#   • 'abc'.encode('utf-8', None)  → mamba: b'abc'  (TypeError)
#   • bytes.fromhex(5)             → mamba: b''     (TypeError)
#   • bytes.fromhex(None)          → mamba: b''     (TypeError)
#   • bytes.fromhex([1,2])         → mamba: b''     (TypeError)
#   • int('10', 5.0)               → mamba: 10      (TypeError)
#   • int('10', None)              → mamba: 10      (TypeError)
#   • int('10', [1,2])             → mamba: 10      (TypeError)
#   • bin(3.14)                    → mamba: None    (TypeError)
#   • hex(3.14)                    → mamba: None    (TypeError)
#   • oct(3.14)                    → mamba: None    (TypeError)
#   • bin('abc')                   → mamba: None    (TypeError)
#   • float([1,2])                 → mamba: 0       (TypeError)
#   • float({1:2})                 → mamba: 0       (TypeError)
#
# CPython contract (uniform across every form):
#   str.encode(non_str_enc) / bytes.decode(non_str_enc)
#       → TypeError("encode()/decode() argument 'encoding' must be
#                    str, not <type>");
#   str.encode('utf-8', non_str_errors)
#       → TypeError("encode() argument 'errors' must be str,
#                    not <type>");
#   bytes.fromhex(non_str)
#       → TypeError("fromhex() argument must be str, not <type>");
#   int(s, non_int_base) / bin/hex/oct(non_int)
#       → TypeError("'<type>' object cannot be interpreted as an
#                    integer");
#   float(non_str_non_num)
#       → TypeError("float() argument must be a string or a real
#                    number, not '<type>'").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_n: Any = None
_i: Any = 5
_f: Any = 3.14
_l: Any = [1, 2]
_d: Any = {1: 2}
_s: Any = 'abc'

# 'abc'.encode(5) — encoding arg must be str
try:
    _ = 'abc'.encode(_i)
    raise AssertionError("'abc'.encode(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.encode(None)
try:
    _ = 'abc'.encode(_n)
    raise AssertionError("'abc'.encode(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.encode([1,2])
try:
    _ = 'abc'.encode(_l)
    raise AssertionError("'abc'.encode([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b'abc'.decode(5)
try:
    _ = b'abc'.decode(_i)
    raise AssertionError("b'abc'.decode(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b'abc'.decode(None)
try:
    _ = b'abc'.decode(_n)
    raise AssertionError("b'abc'.decode(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# b'abc'.decode([1,2])
try:
    _ = b'abc'.decode(_l)
    raise AssertionError("b'abc'.decode([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.encode('utf-8', 5) — errors arg must be str
try:
    _ = 'abc'.encode('utf-8', _i)
    raise AssertionError("'abc'.encode('utf-8', 5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# 'abc'.encode('utf-8', None)
try:
    _ = 'abc'.encode('utf-8', _n)
    raise AssertionError("'abc'.encode('utf-8', None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes.fromhex(5)
try:
    _ = bytes.fromhex(_i)
    raise AssertionError("bytes.fromhex(5) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes.fromhex(None)
try:
    _ = bytes.fromhex(_n)
    raise AssertionError("bytes.fromhex(None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes.fromhex([1,2])
try:
    _ = bytes.fromhex(_l)
    raise AssertionError("bytes.fromhex([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int('10', 5.0) — base arg must be int (via __index__)
try:
    _ = int('10', _f)
    raise AssertionError("int('10', 5.0) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int('10', None)
try:
    _ = int('10', _n)
    raise AssertionError("int('10', None) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int('10', [1,2])
try:
    _ = int('10', _l)
    raise AssertionError("int('10', [1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin(3.14)
try:
    _ = bin(_f)
    raise AssertionError("bin(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# hex(3.14)
try:
    _ = hex(_f)
    raise AssertionError("hex(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# oct(3.14)
try:
    _ = oct(_f)
    raise AssertionError("oct(3.14) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bin('abc') — str cannot be interpreted as int
try:
    _ = bin(_s)
    raise AssertionError("bin('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

# float([1,2])
try:
    _ = float(_l)
    raise AssertionError("float([1,2]) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float({1:2})
try:
    _ = float(_d)
    raise AssertionError("float({1:2}) must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_codec_conv_arg_type_silent {sum(_ledger)} asserts")
