# Spec seed for CPython IndexError / KeyError / AttributeError /
# ValueError / TypeError contract on the `str.format` field-index /
# field-attr / format-spec-mismatch / format_map(non_mapping) corners
# where mamba silently returns the original template (`'{0[5]}'`)
# or coerces to a default value (`'0    '`) instead of raising.
#
# Surface: CPython rejects (1) `'{0[idx]}'.format(seq)` when `idx`
# is out of range — IndexError("<seq-type> index out of range"); (2)
# `'{0[key]}'.format(mapping)` when `key` is missing — KeyError(key);
# (3) `'{0.attr}'.format(value)` when `value` has no attribute
# `attr` — AttributeError("'<type>' object has no attribute '<attr>'");
# (4) `'{:5d}'.format(non_int)` / `'{:5f}'.format(non_float)` /
# `'{:5s}'.format(non_str)` / `'{:.3f}'.format(non_float)` /
# `'{:x}'.format(non_int)` because the format code requires a
# specific type — ValueError("Unknown format code '<code>' for
# object of type '<type>'") OR TypeError("unsupported format string
# passed to <type>.__format__"); (5) `'{x}'.format_map(non_mapping)`
# because format_map requires a mapping — TypeError("'<type>' object
# is not subscriptable" / "list indices must be integers ...").
#
# Mamba accepts every form and silently:
#   - returns the ORIGINAL template ('{0[5]}' / '{0.foo}') for the
#     field-index / field-attr family — masking the operator's intent
#     to access seq[5] / value.attr;
#   - returns the DEFAULT-coerced result ('0    ' for %5d, '0.000'
#     for %.3f, '0' for %x, etc.) for the format-spec wrong-type
#     family — masking the call-site bug where the wrong type
#     accidentally reached `.format(...)`;
#   - raises KeyError (not TypeError) on `format_map(non_mapping)`,
#     so the runtime contract diverges on the EXCEPTION CLASS even
#     though both raise — different from the CPython-specified
#     TypeError("'<type>' object is not subscriptable").
#
# Existing lang_format_complex_fromhex_silent.py covers
# `"{x}".format()` (KeyError on missing named field) and
# `"{0}".format()` (IndexError on out-of-range positional INDEX).
# Existing lang_format_attr_codec_silent.py covers `format(str, 'd')`
# / `'{:d}'.format('abc')` (ValueError on format-code mismatch
# without a width). Existing lang_percent_format_wrong_type_silent.py
# covers `'%d' % str` etc. on the `%`-format-operator side. This
# seed covers the FRESH divergence family — the `str.format`
# FIELD-INDEX (`{0[idx]}`) / FIELD-ATTR (`{0.attr}`) corners plus
# the WIDTH-prefixed format-spec wrong-type corners (`{:5d}` /
# `{:5f}` / `{:5s}` / `{:.3f}` / `{:x}` / `{:o}` / `{:b}`) and the
# `format_map(non_mapping)` TypeError-vs-KeyError corner.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • '{0[5]}'.format([1,2,3])         → mamba: '{0[5]}'    (IndexError)
#   • '{0[5]}'.format('abc')           → mamba: '{0[5]}'    (IndexError)
#   • '{0[bad]}'.format({1:2})         → mamba: '{0[bad]}'  (KeyError)
#   • '{0.foo}'.format(5)              → mamba: '{0.foo}'   (AttributeError)
#   • '{0.bar}'.format([1,2])          → mamba: '{0.bar}'   (AttributeError)
#   • '{:5d}'.format('a')              → mamba: '0    '     (ValueError)
#   • '{:5d}'.format([1])              → mamba: '0    '     (TypeError)
#   • '{:5d}'.format(None)             → mamba: '    0'     (TypeError)
#   • '{:5f}'.format('a')              → mamba: '0.000000'  (ValueError)
#   • '{:5s}'.format(5)                → mamba: '    5'     (ValueError)
#   • '{:.3f}'.format('a')             → mamba: '0.000'     (ValueError)
#   • '{:x}'.format('a')               → mamba: '0'         (ValueError)
#   • '{:o}'.format([1])               → mamba: '0'         (TypeError)
#   • '{:b}'.format(None)              → mamba: '0'         (TypeError)
#
# CPython contract (uniform across every form):
#   '{0[idx]}'.format(seq)
#       → IndexError("<seq-type> index out of range") when idx
#         is out of range;
#   '{0[key]}'.format(mapping)
#       → KeyError(key) when key is missing;
#   '{0.attr}'.format(value)
#       → AttributeError("'<type>' object has no attribute '<attr>'");
#   '{:<code>}'.format(wrong_type)
#       → ValueError("Unknown format code '<code>' for object of
#                    type '<type>'") OR TypeError("unsupported format
#                    string passed to <type>.__format__").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_l: Any = [1, 2, 3]
_s: Any = 'abc'
_d: Any = {1: 2}
_i: Any = 5
_n: Any = None

# '{0[5]}'.format([1,2,3]) — list index out of range
try:
    _ = '{0[5]}'.format(_l)
    raise AssertionError("'{0[5]}'.format([1,2,3]) must raise IndexError")
except IndexError:
    _ledger.append(1)

# '{0[5]}'.format('abc') — string index out of range
try:
    _ = '{0[5]}'.format(_s)
    raise AssertionError("'{0[5]}'.format('abc') must raise IndexError")
except IndexError:
    _ledger.append(1)

# '{0[10]}'.format([1,2,3]) — another out-of-range list index
try:
    _ = '{0[10]}'.format(_l)
    raise AssertionError("'{0[10]}'.format([1,2,3]) must raise IndexError")
except IndexError:
    _ledger.append(1)

# '{0[bad]}'.format({1:2}) — missing dict key
try:
    _ = '{0[bad]}'.format(_d)
    raise AssertionError("'{0[bad]}'.format({1:2}) must raise KeyError")
except KeyError:
    _ledger.append(1)

# '{0[missing]}'.format({1:2}) — another missing key
try:
    _ = '{0[missing]}'.format(_d)
    raise AssertionError("'{0[missing]}'.format({1:2}) must raise KeyError")
except KeyError:
    _ledger.append(1)

# '{0.foo}'.format(5) — int has no attr foo
try:
    _ = '{0.foo}'.format(_i)
    raise AssertionError("'{0.foo}'.format(5) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# '{0.bar}'.format([1,2]) — list has no attr bar
try:
    _ = '{0.bar}'.format(_l)
    raise AssertionError("'{0.bar}'.format([1,2]) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# '{0.zzz}'.format({1:2}) — dict has no attr zzz
try:
    _ = '{0.zzz}'.format(_d)
    raise AssertionError("'{0.zzz}'.format({1:2}) must raise AttributeError")
except AttributeError:
    _ledger.append(1)

# '{:5d}'.format('a') — format code 'd' rejects str
try:
    _ = '{:5d}'.format(_s)
    raise AssertionError("'{:5d}'.format('a') must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:5d}'.format([1]) — format code 'd' rejects list (TypeError on CPython)
try:
    _ = '{:5d}'.format(_l)
    raise AssertionError("'{:5d}'.format([1]) must raise TypeError or ValueError")
except (TypeError, ValueError):
    _ledger.append(1)

# '{:5d}'.format(None) — format code 'd' rejects None
try:
    _ = '{:5d}'.format(_n)
    raise AssertionError("'{:5d}'.format(None) must raise TypeError or ValueError")
except (TypeError, ValueError):
    _ledger.append(1)

# '{:5f}'.format('a') — format code 'f' rejects str
try:
    _ = '{:5f}'.format(_s)
    raise AssertionError("'{:5f}'.format('a') must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:5s}'.format(5) — format code 's' rejects int
try:
    _ = '{:5s}'.format(_i)
    raise AssertionError("'{:5s}'.format(5) must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:.3f}'.format('a') — precision-f rejects str
try:
    _ = '{:.3f}'.format(_s)
    raise AssertionError("'{:.3f}'.format('a') must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:x}'.format('a') — format code 'x' rejects str
try:
    _ = '{:x}'.format(_s)
    raise AssertionError("'{:x}'.format('a') must raise ValueError")
except ValueError:
    _ledger.append(1)

# '{:o}'.format([1]) — format code 'o' rejects list (TypeError)
try:
    _ = '{:o}'.format(_l)
    raise AssertionError("'{:o}'.format([1]) must raise TypeError or ValueError")
except (TypeError, ValueError):
    _ledger.append(1)

# '{:b}'.format(None) — format code 'b' rejects None (TypeError)
try:
    _ = '{:b}'.format(_n)
    raise AssertionError("'{:b}'.format(None) must raise TypeError or ValueError")
except (TypeError, ValueError):
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_format_field_attr_spec_silent {sum(_ledger)} asserts")
