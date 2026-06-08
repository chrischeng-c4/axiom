# Set membership and mutation via Python-semantic equality
# Tests for str / tuple / bool-int cross members.

# String members
s1 = {"alpha", "beta", "gamma"}
print("alpha" in s1)
print("delta" in s1)
s1.discard("alpha")
print("alpha" in s1)
print(len(s1))

# Remove raises KeyError if absent
try:
    s1.remove("zeta")
except KeyError as e:
    print("KE:", e)

# Tuple members
s2 = {(1, 2), (3, 4)}
print((1, 2) in s2)
print((5, 6) in s2)
s2.remove((1, 2))
print((1, 2) in s2)
print(len(s2))

# Bool/int cross — 1 equals True
s3 = {1, 2, 3}
print(True in s3)
print(False in s3)

# Adding True when 1 is present should be a no-op
s4 = {1, 2, 3}
s4.add(True)
print(len(s4))

# set-from-list dedupes across equal values
s5 = set([1, 1, 2, 2, 3])
print(len(s5))

# Union of mixed-type with equivalent elements
s6 = {"a", "b"} | {"b", "c"}
print(sorted(s6))

# intersection
s7 = {1, 2, 3} & {2, 3, 4}
print(sorted(s7))

# difference with equal-but-different-identity strings
a = "hello"
b = "hel" + "lo"
print(a == b)  # True, but different allocations
s8 = {a}
print(b in s8)  # True via mb_eq
s8.discard(b)
print(len(s8))

# Dict int-key membership
d1 = {1: "one", 2: "two"}
print(1 in d1)
print(3 in d1)
print(d1[1])
