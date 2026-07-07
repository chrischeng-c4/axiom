# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# #942: `int(x, base=...)` — the variadic Ty::Fn keyword-binding path mis-
# packed the `base` keyword argument.
#
# A call carrying a keyword arg (`int("ff", base=16)`) is routed dynamically:
# the compiler emits `mb_builtin_type_obj("int")` for the callee, builds a
# positional list and a kwargs dict, and calls
# `mb_call_spread_kwargs(callee, pos_list, kwargs_dict)`. Since `int` isn't a
# plain native function pointer (it's a type-object value used for
# `isinstance`/`type() is int` too), `resolve_callable` can't find a native
# address for it, so `mb_call_spread_kwargs` falls through to its generic
# "append kwargs dict as a trailing positional" convention and re-enters
# `mb_call_spread`. There, the `class_name == "type"` constructor-dispatch
# match arm for `"int"` blindly read `items[1]` as the base value — but for
# a keyword call `items[1]` is the *kwargs dict itself* (`{"base": 16}`), not
# an integer. `resolve_index_value` on a dict fails, raising the spurious
# `TypeError: int() base must be an integer` instead of converting.
#
# Fix (runtime/builtins.rs, `mb_call_spread`'s `"int"` constructor arm): when
# the 2nd item is a dict, pull `base` back out by key instead of handing the
# dict itself to `mb_int_base`.

# Headline repro from the issue.
print(int("ff", base=16))          # 255
print(int("10", base=2))           # 2

# Positional twin must stay unchanged (already worked; regression guard).
print(int("ff", 16))               # 255
print(int("10", 2))                # 2

# Other bases via keyword.
print(int("10", base=8))           # 8
print(int("z", base=36))           # 35
print(int("-ff", base=16))         # -255
print(int("0x1a", base=0))         # 26 (prefix auto-detect)

# No base at all — unaffected single-arg / zero-arg forms.
print(int("42"))                   # 42
print(int())                       # 0
