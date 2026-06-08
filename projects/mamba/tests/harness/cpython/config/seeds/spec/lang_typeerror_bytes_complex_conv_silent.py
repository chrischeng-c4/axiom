# Spec seed for CPython TypeError / ValueError contract on the
# str↔bytes encoding-required and complex↔numeric narrowing-disallowed
# conversion corners. Surface: CPython 3 requires an explicit
# `encoding=` argument on `bytes(str)` / `bytearray(str)` (no implicit
# Latin-1 / ASCII fallback), and rejects `int(complex)` / `float(complex)`
# entirely (no real-part extraction even when `imag == 0`). It also
# parses `complex(str)` strictly: invalid literals raise ValueError,
# not silently returning `None`. Mamba 0.3.60 silently coerces / returns
# `0` / `0.0` / `b'abc'` / `None` instead of dispatching the
# `__bytes__`/`__int__`/`__complex__` protocol → TypeError-or-ValueError
# fallback. Existing lang_typeerror_conversion_unary seed covers
# int(list/dict/tuple/set), float(list/dict), complex(list/None), but
# skips the bytes-without-encoding and complex-numeric-narrowing
# corners.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • bytes('abc')                   → mamba: b'abc'         (TypeError)
#   • bytearray('abc')               → mamba: bytearray(...) (TypeError)
#   • bytes('héllo')                 → mamba: b'héllo'-ish   (TypeError)
#   • bytes(None)                    → mamba: silent         (TypeError)
#   • int(complex(3, 4))             → mamba: 0              (TypeError)
#   • int(complex(5, 0))             → mamba: 0              (TypeError)
#   • float(complex(3, 4))           → mamba: 0.0            (TypeError)
#   • float(complex(0, 0))           → mamba: 0.0            (TypeError)
#   • complex('hello')               → mamba: None           (ValueError)
#   • complex('1+2')                 → mamba: None           (ValueError)
#   • complex('')                    → mamba: None           (ValueError)
#   • range('abc')                   → mamba: []             (TypeError)
#
# CPython contract:
#   bytes(str_no_encoding)   → TypeError("string argument without
#                                  an encoding");
#   bytearray(str_no_enc)    → TypeError(same);
#   int(complex_value)       → TypeError("int() argument must be a
#                                  string, a bytes-like object or a
#                                  real number, not 'complex'");
#   float(complex_value)     → TypeError("float() argument must be a
#                                  string or a real number, not
#                                  'complex'");
#   complex(bad_str)         → ValueError("complex() arg is a
#                                  malformed string");
#   range(non_int_iter)      → TypeError("'<typename>' object cannot
#                                  be interpreted as an integer").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so the
# runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_str_a: Any = "abc"
_str_unicode: Any = "héllo"
_c_full: Any = complex(3, 4)
_c_real: Any = complex(5, 0)
_c_zero: Any = complex(0, 0)

# bytes('abc') — encoding is required for str source
try:
    _ = bytes(_str_a)
    raise AssertionError("bytes(str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytes('héllo') — unicode str also needs encoding
try:
    _ = bytes(_str_unicode)
    raise AssertionError("bytes('héllo') must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytearray('abc') — same rule applies to bytearray
try:
    _ = bytearray(_str_a)
    raise AssertionError("bytearray(str) must raise TypeError")
except TypeError:
    _ledger.append(1)

# bytearray('héllo') — unicode bytearray also needs encoding
try:
    _ = bytearray(_str_unicode)
    raise AssertionError("bytearray('héllo') must raise TypeError")
except TypeError:
    _ledger.append(1)

# int(complex(3, 4)) — complex has no narrowing to int even when imag>0
try:
    _ = int(_c_full)
    raise AssertionError("int(complex(3, 4)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# int(complex(5, 0)) — even with imag=0 CPython refuses (no implicit
# real-part extraction)
try:
    _ = int(_c_real)
    raise AssertionError("int(complex(5, 0)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float(complex(3, 4)) — complex has no narrowing to float
try:
    _ = float(_c_full)
    raise AssertionError("float(complex(3, 4)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# float(complex(0, 0)) — zero complex also refuses narrowing
try:
    _ = float(_c_zero)
    raise AssertionError("float(complex(0, 0)) must raise TypeError")
except TypeError:
    _ledger.append(1)

# complex('hello') — non-numeric str is a malformed complex literal
try:
    _ = complex("hello")
    raise AssertionError("complex('hello') must raise ValueError")
except ValueError:
    _ledger.append(1)

# complex('1+2') — missing trailing 'j' is a malformed complex literal
try:
    _ = complex("1+2")
    raise AssertionError("complex('1+2') must raise ValueError")
except ValueError:
    _ledger.append(1)

# complex('') — empty string is a malformed complex literal
try:
    _ = complex("")
    raise AssertionError("complex('') must raise ValueError")
except ValueError:
    _ledger.append(1)

# range('abc') — range needs int-like, not str
try:
    _ = list(range(_str_a))
    raise AssertionError("range('abc') must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_typeerror_bytes_complex_conv_silent {sum(_ledger)} asserts")
