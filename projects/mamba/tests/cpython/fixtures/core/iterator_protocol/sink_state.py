# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/iterator_protocol: iterators are one-shot ("sink state").

Once an iterator from any builtin iterable is fully consumed it stays
empty; re-listing the SAME iterator yields nothing. This is distinct
from re-calling iter() on the container, which makes a fresh iterator.
"""


def drains_once(label, iterable, expected):
    it = iter(iterable)
    first = list(it)
    second = list(it)
    assert first == expected, (label, first)
    assert second == [], (label, second)
    print("[sink]", label, "->", first, "then", second)


drains_once("list", [0, 1, 2, 3, 4], [0, 1, 2, 3, 4])
drains_once("tuple", (0, 1, 2, 3, 4), [0, 1, 2, 3, 4])
drains_once("str", "abcde", ["a", "b", "c", "d", "e"])
drains_once("range", range(5), [0, 1, 2, 3, 4])

# dict iterates its keys; views over the same dict each drain once.
d = {1: 10, 2: 20, 3: 30}
drains_once("dict", d, [1, 2, 3])
drains_once("dict.keys", d.keys(), [1, 2, 3])
drains_once("dict.values", d.values(), [10, 20, 30])
drains_once("dict.items", d.items(), [(1, 10), (2, 20), (3, 30)])

# enumerate is itself an iterator and exhausts.
drains_once("enumerate", enumerate("xy"), [(0, "x"), (1, "y")])


# A generator object is an iterator: once run to completion it is spent.
def counter():
    for i in range(5):
        yield i


drains_once("generator", counter(), [0, 1, 2, 3, 4])

print("sink_state OK")
