# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Keyword-only parameters mixed with *args, **kwargs, and defaults.
#
# After a bare `*` (or a `*args` collector), every following parameter
# is keyword-only. Keyword-only params may have defaults or be required,
# in any order, and they coexist with **kwargs which absorbs the rest.
#
# Clauses:
#   1. `*, k1=0, k2` — defaulted kwonly may precede a required kwonly.
#   2. `*, k1, k2` — both required, must be passed by keyword.
#   3. `*, k1, k2, **kw` — extra keywords spill into **kw.
#   4. `a, b=0, *arg, k1, k2=0` — positional + varargs + kwonly mix.
#   5. `a, b=0, *arg, k1, k2=0, **kw` — full grand-slam signature.
#   6. `*nums, reverse=False` — varargs then a defaulted kwonly flag.
#
# Each print line is tagged `[kwonly-mixed]`.


def kw_default_then_required(*, k1=0, k2):
    return k1 + k2


print("[kwonly-mixed] clause-1:", kw_default_then_required(k2=5))
print("[kwonly-mixed] clause-1 both:", kw_default_then_required(k1=3, k2=5))


def kw_both_required(*, k1, k2):
    return k1 + k2


print("[kwonly-mixed] clause-2:", kw_both_required(k1=2, k2=4))


def kw_and_kwargs(*, k1, k2, **kw):
    return k1 + k2 + sum(kw.values())


print("[kwonly-mixed] clause-3:", kw_and_kwargs(k1=1, k2=2, extra=10, more=20))


def pos_varargs_kw(a, b=0, *arg, k1, k2=0):
    return a + b + k1 + k2 + sum(arg)


print("[kwonly-mixed] clause-4:", pos_varargs_kw(1, 2, 3, 4, k1=5))


def grand_slam(a, b=0, *arg, k1, k2=0, **kw):
    return a + b + k1 + k2 + sum(arg) + sum(kw.values())


print("[kwonly-mixed] clause-5:", grand_slam(1, 2, 3, k1=4, k2=5, z=6))


def sortnum(*nums, reverse=False):
    return sorted(nums, reverse=reverse)


print("[kwonly-mixed] clause-6:", sortnum(3, 1, 2))
print("[kwonly-mixed] clause-6 reverse:", sortnum(3, 1, 2, reverse=True))
