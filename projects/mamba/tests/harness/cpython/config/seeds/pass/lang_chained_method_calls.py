# Operational AssertionPass seed for chained method calls and
# attribute access.
# Surface: string-method chains like .strip().lower().split() compose
# left-to-right; the result of a method call can itself be subscripted,
# called, or have another method invoked; .format() on a literal
# string returns a string that supports further method calls;
# comprehensions over a method-chain result work; chained dict-method
# results compose with sorted/list/sum.
_ledger: list[int] = []

# .strip().lower().split() — classic three-step normalization
assert "  HELLO  ".strip().lower().split() == ["hello"]; _ledger.append(1)
# .strip + .split with explicit separator
assert "a,b,c".strip().split(",") == ["a", "b", "c"]; _ledger.append(1)

# chained string transforms — .lower() then .replace() then .upper()
assert "ABC".lower().replace("a", "X").upper() == "XBC"; _ledger.append(1)

# .format() on a literal supports further method calls on its result
assert "Hello, {}!".format("world").upper() == "HELLO, WORLD!"; _ledger.append(1)
assert "{} + {} = {}".format(1, 2, 3).replace("+", "plus") == "1 plus 2 = 3"; _ledger.append(1)

# The result of a function call can be subscripted directly
def _first_item():
    return [10, 20, 30]

assert _first_item()[0] == 10; _ledger.append(1)
assert _first_item()[-1] == 30; _ledger.append(1)
# And sliced
assert _first_item()[:2] == [10, 20]; _ledger.append(1)

# The result of a function call can be passed to sorted directly
def _scrambled():
    return [3, 1, 4, 1, 5, 9, 2, 6]

assert sorted(_scrambled()) == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)
# Same source through max / min
assert max(_scrambled()) == 9; _ledger.append(1)
assert min(_scrambled()) == 1; _ledger.append(1)

# .keys() / .values() return views that compose with list / sorted
words = {"apple": 1, "banana": 2, "cherry": 3}
assert sorted(words.keys()) == ["apple", "banana", "cherry"]; _ledger.append(1)
assert sorted(words.values()) == [1, 2, 3]; _ledger.append(1)

# Comprehension on a method-call result
assert [k.upper() for k in words.keys()] == ["APPLE", "BANANA", "CHERRY"]; _ledger.append(1)

# sum over a comprehension over a .split() chain
assert sum([int(x) for x in "1,2,3,4".split(",")]) == 10; _ledger.append(1)

# .split().count() — chain on a string split
assert "a b a c a".split().count("a") == 3; _ledger.append(1)

# .join + sorted + map composition
assert "-".join(sorted(["banana", "apple", "cherry"])) == "apple-banana-cherry"; _ledger.append(1)

# Repeated chained .replace()
assert "aaa".replace("a", "b").replace("b", "c") == "ccc"; _ledger.append(1)

# Chain on a number — str(int).zfill(n)
assert str(42).zfill(5) == "00042"; _ledger.append(1)
# Chain on a number — str().split() of f-string
nums = [1, 2, 3]
assert " ".join(str(n) for n in nums).split() == ["1", "2", "3"]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_chained_method_calls {sum(_ledger)} asserts")
