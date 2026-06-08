# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/iterator_protocol: operator.length_hint on live list iterators.

length_hint() reports how many items an iterator believes remain. For
list iterators the estimate tracks consumption and reacts to in-place
mutation of the underlying list.
"""

from operator import length_hint

n = 10

# Forward list iterator: consuming items lowers the hint.
data = list(range(n))
it = iter(data)
next(it)
next(it)
assert length_hint(it) == n - 2
print("[length_hint] forward after 2 consumed:", length_hint(it))

# Appending to the list grows the forward iterator's remaining count.
data.append(n)
assert length_hint(it) == n - 1
print("[length_hint] forward after append:", length_hint(it))

# Truncating the list past the cursor collapses the hint to zero and
# the iterator yields nothing more.
data[1:] = []
assert length_hint(it) == 0
assert list(it) == []
print("[length_hint] forward after truncate:", length_hint(it))

# Extending an already-exhausted iterator's list does not revive it.
data.extend(range(20))
assert length_hint(it) == 0
print("[length_hint] forward stays exhausted:", length_hint(it))


# Reverse iterator: append does NOT change its remaining count because
# it walks toward the front, but truncation past its cursor empties it.
rdata = list(range(n))
rit = reversed(rdata)
next(rit)
next(rit)
assert length_hint(rit) == n - 2
print("[length_hint] reversed after 2 consumed:", length_hint(rit))

rdata.append(n)
assert length_hint(rit) == n - 2
print("[length_hint] reversed unaffected by append:", length_hint(rit))

rdata[1:] = []
assert length_hint(rit) == 0
assert list(rit) == []
print("[length_hint] reversed after truncate:", length_hint(rit))

print("length_hint OK")
