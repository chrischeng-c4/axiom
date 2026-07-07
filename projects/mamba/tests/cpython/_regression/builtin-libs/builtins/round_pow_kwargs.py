# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# #942: the same "variadic Ty::Fn keyword-binding drops named args" bug that
# broke `int(x, base=...)` also broke the native-function dispatch table in
# `runtime/stdlib/builtins_mod.rs`. A keyword call to an ordinary function
# (not a type constructor — `round`/`pow` resolve to a real native function
# pointer, so they route through `mb_call_spread_kwargs` -> `dispatch_round`
# / `dispatch_pow` rather than through `mb_call_spread`'s type-object arm)
# also reaches its dispatcher with the kwargs dict appended as a trailing
# positional. `dispatch_round`/`dispatch_pow` read that trailing slot
# directly (`args.get(1)`, `args[2]`) instead of unpacking it, so
# `round(x, ndigits=2)` and `pow(base, exp, mod=m)` fed the dict itself to
# `mb_round`/`mb_pow_mod` and mis-happened (float ndigits check failed /
# "pow() 3rd argument not allowed unless all arguments are integers").
#
# Fix: `dispatch_round` and `dispatch_pow` now split the trailing kwargs
# dict (`split_kwargs`) and bind `number`/`ndigits` and `base`/`exp`/`mod`
# by name (`arg_or_kw`) before falling back to positional reads — mirroring
# the fix already used by `dispatch_open` for `mode=`/`encoding=`/etc.

# round(number, ndigits=...) — headline repro.
print(round(2.675, ndigits=2))         # 2.67
print(round(number=2.675, ndigits=2))  # 2.67  (both keyword, either order)
print(round(ndigits=0, number=3.7))    # 4.0

# Positional twin unchanged.
print(round(2.675, 2))                 # 2.67
print(round(3.7))                      # 4

# pow(base, exp, mod=...) — headline repro.
print(pow(2, 10, mod=1000))            # 24
print(pow(base=2, exp=10, mod=1000))   # 24
print(pow(exp=10, base=2))             # 1024  (mod omitted, all keyword)

# Positional twin unchanged.
print(pow(2, 10, 1000))                # 24
print(pow(2, 10))                      # 1024
