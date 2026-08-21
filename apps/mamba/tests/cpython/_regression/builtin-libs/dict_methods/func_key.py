# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Function-pointer values used as dict keys. Mamba previously
# represented them via `DictKey::Other(format!("{}", val.to_bits()))`,
# which (a) hashed by raw NaN-boxed bits (correct identity but lossy),
# and (b) printed the bit pattern as a giant integer when the dict was
# repr'd. CPython reprs each key via `repr(<function>)`. The fix adds
# a `DictKey::Func(addr)` variant that round-trips through
# `dict_key_to_mbvalue` → `MbValue::from_func(addr)` so `mb_repr` picks
# up the FUNC_NAMES-aware `<function NAME at 0xADDR>` shape.

def f(): pass
def g(): pass

# Identity-based hashing — same fn maps, different fns don't.
d = {f: 1, g: 2}
print(d[f])                                # 1
print(d[g])                                # 2
print(len(d))                              # 2

# repr should expose the function name, not the bit pattern.
r = repr(d)
print(r.startswith("{<function f at 0x"))  # True
print(", <function g at 0x" in r)          # True
print(r.endswith(": 2}"))                  # True

# Lookup by the same function value still hits.
d[f] = 100
print(d[f])                                # 100
