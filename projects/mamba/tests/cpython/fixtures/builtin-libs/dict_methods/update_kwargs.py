# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: dict.update(k1=v1, k2=v2) kwargs form must add those pairs.
# The method dispatcher only read positional arg 0, so kwargs were dropped
# and d.update(b=2, c=3) silently left d unchanged.

d = {"a": 1}
d.update(b=2, c=3)
print(d)

# Mixed: dict arg + kwargs
e = {"x": 10}
e.update({"y": 20}, z=30)
print(e)

# kwargs override positional dict entries
f = {}
f.update({"a": 1}, a=100)
print(f)

# kwargs overwrite existing entries
g = {"a": 1, "b": 2}
g.update(b=20, c=30)
print(g)
