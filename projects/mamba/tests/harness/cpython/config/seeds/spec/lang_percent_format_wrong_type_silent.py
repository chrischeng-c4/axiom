# Spec seed for CPython TypeError / OverflowError contract on
# the `%`-format-operator corners that mamba silently coerces
# the wrong type into a default value. Surface: CPython rejects
# `'%d' % str`, `'%d' % None`, `'%d' % list`, `'%s %s' % single`,
# `'%s' % oversize_tuple`, `'%f' % str`, `'%x' % str`, `'%c' %
# str`, `'%c' % 9999999` (range), and `'%(name)s' % non_mapping`
# with TypeError / OverflowError every time, because `%` delegates
# to per-format-code coercion (`%d` calls `int()`, `%f` calls
# `float()`, `%c` requires int or 1-char str in [0,0x10FFFF]).
# Mamba accepts every form and silently returns a default
# rendering — `'%d' % 'abc'` becomes `'0'`, `'%s' % (1, 2)` keeps
# only the first arg, `'%c' % 9999999` returns the literal int,
# `'%(name)s' % 42` returns `'42'`. Code that does `log.info(fmt %
# args)` where `args` was supposed to be a numeric tuple silently
# logs `'0'` / `'0.000000'` / truncated output rather than failing
# loud. This is one of the broadest silent-coercion classes in
# mamba because `%`-formatting is the still-widely-used legacy
# format string surface alongside `str.format` / f-strings.
#
# Probes (every form CPython rejects, mamba silently coerces):
#   • '%d' % 'abc'                  → mamba: '0'           (TypeError)
#   • '%d' % None                   → mamba: '0'           (TypeError)
#   • '%d' % [1, 2]                 → mamba: '0'           (TypeError)
#   • '%s %s' % 'single'            → mamba: 'single '     (TypeError)
#   • '%s %s' % (1,)                → mamba: '1 '          (TypeError)
#   • '%s' % (1, 2)                 → mamba: '1'           (TypeError)
#   • '%f' % 'abc'                  → mamba: '0.000000'    (TypeError)
#   • '%x' % 'abc'                  → mamba: '0'           (TypeError)
#   • '%c' % 'abc'                  → mamba: 'abc'         (TypeError)
#   • '%c' % 9999999                → mamba: '9999999'     (OverflowError)
#   • '%(name)s' % 42               → mamba: '42'          (TypeError)
#
# CPython contract (uniform across every format code):
#   '%d'/'%i'/'%x'/'%o' % non_int   → TypeError("%d format: …");
#   '%f'/'%e'/'%g' % non_float      → TypeError("must be real number, …");
#   '%c' % non_int_or_1char         → TypeError("%c requires int …");
#   '%c' % int_oor                  → OverflowError("%c arg not in …");
#   '%s'/'%r' % wrong_arity         → TypeError("not enough/ all arguments …");
#   '%(key)s' % non_mapping         → TypeError("format requires a mapping").
#
# `Any`-typed holders push the operand past static type-checkers
# (Pyright) and past mamba's compile-time argtype enforcement so
# the runtime divergence is what's exercised.
from typing import Any
_ledger: list[int] = []

_fmt_d: Any = "%d"
_fmt_two_s: Any = "%s %s"
_fmt_one_s: Any = "%s"
_fmt_f: Any = "%f"
_fmt_x: Any = "%x"
_fmt_c: Any = "%c"
_fmt_named: Any = "%(name)s"

_abc: Any = "abc"
_none: Any = None
_lst: Any = [1, 2]
_single_str: Any = "single"
_one_tup: Any = (1,)
_two_tup: Any = (1, 2)
_big_int: Any = 9999999
_non_mapping: Any = 42

# '%d' % str
try:
    _ = _fmt_d % _abc
    raise AssertionError("'%d' % 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%d' % None
try:
    _ = _fmt_d % _none
    raise AssertionError("'%d' % None must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%d' % list
try:
    _ = _fmt_d % _lst
    raise AssertionError("'%d' % [1,2] must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%s %s' % str (single value supplied, two slots needed)
try:
    _ = _fmt_two_s % _single_str
    raise AssertionError("'%s %s' % 'single' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%s %s' % (1,) — not enough arguments
try:
    _ = _fmt_two_s % _one_tup
    raise AssertionError("'%s %s' % (1,) must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%s' % (1, 2) — too many arguments
try:
    _ = _fmt_one_s % _two_tup
    raise AssertionError("'%s' % (1,2) must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%f' % str
try:
    _ = _fmt_f % _abc
    raise AssertionError("'%f' % 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%x' % str
try:
    _ = _fmt_x % _abc
    raise AssertionError("'%x' % 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%c' % multi-char str
try:
    _ = _fmt_c % _abc
    raise AssertionError("'%c' % 'abc' must raise TypeError")
except TypeError:
    _ledger.append(1)

# '%c' % int-out-of-range (OverflowError, not TypeError)
try:
    _ = _fmt_c % _big_int
    raise AssertionError("'%c' % 9999999 must raise OverflowError")
except OverflowError:
    _ledger.append(1)

# '%(name)s' % non-mapping (int, not dict)
try:
    _ = _fmt_named % _non_mapping
    raise AssertionError("'%(name)s' % 42 must raise TypeError")
except TypeError:
    _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_percent_format_wrong_type_silent {sum(_ledger)} asserts")
