# Operational AssertionPass seed for builtin functions beyond
# test_iter_builtins_ops.
# Surface: iter(callable, sentinel) two-arg form; next(it, default);
# reversed on tuple; range with negative step; sorted with key+reverse
# combined; min/max with key=; min/max with default= on an empty input;
# divmod; three-arg pow modulo; abs on float; round with ndigits.
_ledger: list[int] = []


# iter(callable, sentinel) calls the zero-arg callable until the
# returned value equals sentinel
class _Lines:
    def __init__(self):
        self.n = 0
    def __call__(self):
        self.n += 1
        if self.n > 3:
            return "STOP"
        return f"line{self.n}"

src = _Lines()
assert list(iter(src, "STOP")) == ["line1", "line2", "line3"]; _ledger.append(1)

# next(it, default) returns default when the iterator is exhausted
# instead of raising StopIteration
it_ex = iter([1, 2])
assert next(it_ex) == 1; _ledger.append(1)
assert next(it_ex) == 2; _ledger.append(1)
assert next(it_ex, "end") == "end"; _ledger.append(1)

# reversed works on any indexable sequence, tuples included
assert list(reversed((1, 2, 3))) == [3, 2, 1]; _ledger.append(1)

# range with a negative step counts down (stop is exclusive)
assert list(range(10, 0, -2)) == [10, 8, 6, 4, 2]; _ledger.append(1)

# sorted with key= and reverse= combined
assert sorted(["bb", "a", "ccc"], key=len, reverse=True) == ["ccc", "bb", "a"]; _ledger.append(1)

# max/min with key= picks the element with the largest/smallest key value
assert max(["bb", "a", "ccc"], key=len) == "ccc"; _ledger.append(1)
assert min(["bb", "a", "ccc"], key=len) == "a"; _ledger.append(1)

# max/min with default= return the default on an empty input rather
# than raising ValueError
assert max([], default="none") == "none"; _ledger.append(1)
assert min([], default=42) == 42; _ledger.append(1)

# divmod(a, b) returns (a // b, a % b) as a single tuple
assert divmod(17, 5) == (3, 2); _ledger.append(1)

# Three-arg pow(base, exp, mod) computes base**exp % mod efficiently
assert pow(2, 10, 1000) == 24; _ledger.append(1)

# abs on float preserves the float type / sign-stripped magnitude
assert abs(-3.5) == 3.5; _ledger.append(1)

# round with ndigits returns a value rounded to that many fractional digits
assert round(3.14159, 2) == 3.14; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_builtin_extras_ops {sum(_ledger)} asserts")
