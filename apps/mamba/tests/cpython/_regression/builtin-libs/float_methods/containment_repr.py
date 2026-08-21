# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Float identity in containers, short-repr round-trips, memoryview parsing."""

INF = float("inf")
NAN = float("nan")

# Even though NaN != NaN, the *same* object is found in containers by identity.
floats = (INF, -INF, 0.0, 1.0, NAN)
for f in floats:
    assert f in [f]
    assert f in (f,)
    assert f in {f}
    assert f in {f: None}
    assert [f].count(f) == 1
    assert f in floats

# Container equality also holds when built from the same NaN object.
for f in floats:
    assert [f] == [f]
    assert {f} == {f}
    assert {f: None} == {f: None}

# Two *distinct* NaN objects break == but a fresh-built tuple still differs.
assert NAN != float("nan")

# repr is the shortest string that round-trips back to the same float.
short = ["0.0", "1.0", "0.01", "1.23456789", "10.0", "100.0", "0.0001",
        "1e-05", "1e+16", "1000000000000000.0", "9999999999999990.0"]
for s in short:
    assert repr(float(s)) == s
    assert repr(float("-" + s)) == "-" + s
    assert repr(float(s)) == str(float(s))

# float() reads from a buffer slice via memoryview, stopping at junk/space/NUL.
assert float(memoryview(b"12.3")[1:4]) == 2.3
assert float(memoryview(b"12.3\x00")[1:4]) == 2.3
assert float(memoryview(b"12.3 ")[1:4]) == 2.3
assert float(memoryview(b"12.34")[1:4]) == 2.3

print("containment_repr OK")
