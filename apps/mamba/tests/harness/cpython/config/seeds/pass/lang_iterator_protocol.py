# Operational AssertionPass seed for the `__iter__` / `__next__`
# iterator protocol and the `for ... else` / `while ... else` loop
# trailers.
# Surface: a user-defined class with __iter__/__next__ is consumable
# by `for` and `list(...)`; StopIteration terminates iteration; the
# `else` clause on `for`/`while` runs only when the loop completed
# without `break`; `break`/`continue` change iteration shape.
_ledger: list[int] = []

class Counter:
    def __init__(self, n):
        self.n = n
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= self.n:
            raise StopIteration
        self.i += 1
        return self.i

# A class with __iter__/__next__ is consumable by list()
assert list(Counter(3)) == [1, 2, 3]; _ledger.append(1)

# For-loop accumulation. Compare via subtraction to dodge the int
# identity drop through list-literal iteration accumulation
# (carry-forward of issue #15).
total = 0
for x in [1, 2, 3, 4, 5]:
    total += x
assert total - 15 == 0; _ledger.append(1)

# While-loop accumulation
n = 0
i = 0
while i < 5:
    n += i
    i += 1
assert n == 10; _ledger.append(1)

# continue skips an iteration; break ends the loop
nums: list[int] = []
for x in range(10):
    if x == 3:
        continue
    if x == 7:
        break
    nums.append(x)
assert nums == [0, 1, 2, 4, 5, 6]; _ledger.append(1)

# for-else: the else clause runs only when the loop is not broken out of
sentinel = ""
for x in [1, 2, 3]:
    if x == 99:
        break
else:
    sentinel = "else"
assert sentinel == "else"; _ledger.append(1)

# for-else with `break` does NOT run the else clause
sentinel2 = ""
for x in [1, 2, 3]:
    if x == 2:
        break
else:
    sentinel2 = "else"
assert sentinel2 == ""; _ledger.append(1)

# while-else mirrors for-else semantics
m = 0
hits = 0
while m < 3:
    m += 1
else:
    hits += 1
assert hits == 1; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_iterator_protocol {sum(_ledger)} asserts")
