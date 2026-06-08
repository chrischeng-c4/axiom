# Operational AssertionPass seed for function call surfaces involving
# keyword arguments, defaults, *args, and **kwargs.
# Surface: positional calls; explicit-keyword calls; mixed
# positional+keyword; default arguments fall back when omitted;
# *args collects extra positionals into a tuple; **kwargs collects
# extra keywords into a dict; keyword-only arguments declared after
# `*` work when supplied by name; a default value on a keyword-only
# argument is used when not supplied; *args and **kwargs can be
# combined in a single signature.
_ledger: list[int] = []

# Plain positional call
def _f1(a, b, c):
    return f"{a},{b},{c}"
assert _f1(1, 2, 3) == "1,2,3"; _ledger.append(1)
# All keyword
assert _f1(a=1, b=2, c=3) == "1,2,3"; _ledger.append(1)
# Mixed positional + keyword (keyword used out of order)
assert _f1(1, c=3, b=2) == "1,2,3"; _ledger.append(1)

# Default arguments fill in when an arg is omitted
def _f2(a, b=10):
    return f"{a},{b}"
assert _f2(1) == "1,10"; _ledger.append(1)
assert _f2(1, 2) == "1,2"; _ledger.append(1)
# Default can also be overridden via keyword
assert _f2(1, b=99) == "1,99"; _ledger.append(1)

# *args collects extra positional arguments into a tuple
def _f3(*args):
    return list(args)
assert _f3() == []; _ledger.append(1)
assert _f3(1, 2, 3) == [1, 2, 3]; _ledger.append(1)
assert _f3("a", "b") == ["a", "b"]; _ledger.append(1)

# **kwargs collects extra keyword arguments into a dict
def _f4(**kwargs):
    return sorted(kwargs.items())
assert _f4(x=1, y=2) == [("x", 1), ("y", 2)]; _ledger.append(1)
assert _f4() == []; _ledger.append(1)

# Keyword-only after *: callable when supplied by name
def _f5(a, *, b):
    return f"{a},{b}"
assert _f5(1, b=2) == "1,2"; _ledger.append(1)
assert _f5(a=1, b=2) == "1,2"; _ledger.append(1)

# Keyword-only with default falls back when omitted
def _f6(a, *, b=99):
    return f"{a},{b}"
assert _f6(1) == "1,99"; _ledger.append(1)
assert _f6(1, b=2) == "1,2"; _ledger.append(1)

# *args + **kwargs combined: each collects its own kind of extra arg
def _f7(*args, **kwargs):
    return (list(args), sorted(kwargs.items()))
result = _f7(1, 2, x=10, y=20)
assert result[0] == [1, 2]; _ledger.append(1)
assert result[1] == [("x", 10), ("y", 20)]; _ledger.append(1)

# Function with positional + *args + **kwargs
def _f8(first, *args, **kwargs):
    return (first, list(args), sorted(kwargs.items()))
res2 = _f8("head", 1, 2, k=3)
assert res2[0] == "head"; _ledger.append(1)
assert res2[1] == [1, 2]; _ledger.append(1)
assert res2[2] == [("k", 3)]; _ledger.append(1)

# Unpack a list into *args at the call site
def _f9(a, b, c):
    return [a, b, c]
nums = [1, 2, 3]
assert _f9(*nums) == [1, 2, 3]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_function_kwargs_defaults {sum(_ledger)} asserts")
