# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
d = {x: x**2 for x in range(5)}
print(sorted(d.items()))
d2 = {k: v for k, v in d.items() if v > 4}
print(sorted(d2.items()))
