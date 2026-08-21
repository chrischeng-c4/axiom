# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: `str.lower("HELLO")` / `list.append(xs, v)` / `str.lower` as
# a key function. Types are stored as plain strings in Mamba, so these
# calls previously dispatched against the type-name string itself:
# `str.lower("HELLO")` → `"str".lower()` → `"str"`. Now recognize the
# receiver as a type name at attribute-access and method-call time.

# Direct calls
print(str.lower("HELLO"))
print(str.upper("hello"))
print(str.strip("   hi   "))

# As a key function
print(sorted(["BB", "aa", "CC"], key=str.lower))
print(sorted(["BB", "aa", "CC"], key=str.upper))

# Sorting with a method on each tuple element
pairs = [("BB", 1), ("aa", 2), ("CC", 3)]
print(sorted(pairs, key=lambda p: p[0].lower()))
