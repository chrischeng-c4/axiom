# Operational AssertionPass seed for builtin introspection and
# higher-order helpers. Surface: `callable(obj)` reports whether the
# object is callable, including instances of classes defining
# `__call__`. `isinstance(x, (T1, T2))` and `issubclass(C, (T1, T2))`
# accept a tuple of types and return True if any matches; bool is a
# subclass of int. `pow(base, exp, mod)` is modular exponentiation
# returning a single int. `divmod(a, b)` returns `(a // b, a % b)`,
# following floor-division on negatives. `True + True == 2` because
# bool inherits from int. `sum(iter, start)` adds start to the running
# total. `zip` truncates to the shortest iterable; `enumerate(iter,
# start)` indexes from start. `sorted(iter, reverse=True)` and
# `sorted(iter, key=fn)` are the standard sort modifiers.
class C:
    def __call__(self): return 1
_ledger: list[int] = []

# callable on builtin / lambda / class / instance
assert callable(print) == True; _ledger.append(1)
assert callable(42) == False; _ledger.append(1)
assert callable(lambda: 1) == True; _ledger.append(1)
assert callable(C()) == True; _ledger.append(1)
assert callable(C) == True; _ledger.append(1)

# isinstance with tuple of types
assert isinstance(1, (int, str)) == True; _ledger.append(1)
assert isinstance("a", (int, str)) == True; _ledger.append(1)
assert isinstance(1.5, (int, str)) == False; _ledger.append(1)

# issubclass — bool <: int <: object; tuple form
assert issubclass(bool, int) == True; _ledger.append(1)
assert issubclass(int, object) == True; _ledger.append(1)
assert issubclass(str, (int, str)) == True; _ledger.append(1)

# divmod returns (q, r); follows floor convention on negatives
assert divmod(17, 5) == (3, 2); _ledger.append(1)
assert divmod(-17, 5) == (-4, 3); _ledger.append(1)

# pow(b, e) and pow(b, e, mod) modular exponent
assert pow(2, 10) == 1024; _ledger.append(1)
assert pow(2, 10, 1000) == 24; _ledger.append(1)

# bool inherits from int — arithmetic on True/False
assert True + True == 2; _ledger.append(1)
assert True * 5 == 5; _ledger.append(1)
assert False + 1 == 1; _ledger.append(1)

# sum with explicit start
assert sum([1, 2, 3]) == 6; _ledger.append(1)
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)
assert sum([]) == 0; _ledger.append(1)

# zip truncates to shortest input
assert list(zip([1, 2, 3], ["a", "b", "c"])) == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
assert list(zip([1, 2], [3, 4, 5])) == [(1, 3), (2, 4)]; _ledger.append(1)

# enumerate with explicit start
assert list(enumerate(["a", "b"])) == [(0, "a"), (1, "b")]; _ledger.append(1)
assert list(enumerate(["a", "b"], 5)) == [(5, "a"), (6, "b")]; _ledger.append(1)

# reversed on list and string
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)

# range as iterable
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(range(2, 8, 2)) == [2, 4, 6]; _ledger.append(1)

# map / filter return iterators
assert list(map(lambda x: x * 2, [1, 2, 3])) == [2, 4, 6]; _ledger.append(1)
assert list(filter(lambda x: x > 1, [0, 1, 2, 3])) == [2, 3]; _ledger.append(1)

# sorted with reverse= and key=
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]; _ledger.append(1)
assert sorted(["bb", "a", "ccc"], key=len) == ["a", "bb", "ccc"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_builtin_introspection_ops {sum(_ledger)} asserts")
