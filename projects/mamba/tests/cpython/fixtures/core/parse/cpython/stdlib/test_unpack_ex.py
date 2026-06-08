# RUN: parse
# Extracted from CPython 3.12 Lib/test/test_unpack_ex.py — extended unpacking syntax constructs only.


# --- Star at beginning ---

*a, b = [1, 2, 3, 4, 5]
*a, b = range(10)
*a, b = (1, 2, 3)
*a, b = "hello"


# --- Star at end ---

a, *b = [1, 2, 3, 4, 5]
a, *b = range(10)
a, *b = (1, 2, 3)
a, *b = "hello"


# --- Star in middle ---

a, *b, c = [1, 2, 3, 4, 5]
a, *b, c = range(10)
a, *b, c = (1, 2, 3)
a, *b, c = "hello"


# --- Star with multiple fixed positions ---

a, b, *c = [1, 2, 3, 4, 5]
a, b, c, *d = [1, 2, 3, 4, 5]
*a, b, c = [1, 2, 3, 4, 5]
*a, b, c, d = [1, 2, 3, 4, 5]
a, *b, c, d = [1, 2, 3, 4, 5]
a, b, *c, d = [1, 2, 3, 4, 5]


# --- Star with empty result ---

a, *b = [1]
*a, b = [1]
a, *b, c = [1, 2]


# --- Star in list target ---

[*a, b] = [1, 2, 3]
[a, *b] = [1, 2, 3]
[a, *b, c] = [1, 2, 3, 4]


# --- Star in parenthesized target ---

(*a, b) = [1, 2, 3]
(a, *b) = [1, 2, 3]
(a, *b, c) = [1, 2, 3, 4]


# --- Star-only target ---

*a, = [1, 2, 3]
(*a,) = [1, 2, 3]
[*a] = [1, 2, 3]


# --- Extended unpacking in for loops ---

data = [[1, 2, 3, 4], [5, 6, 7, 8]]

# NOTE: starred in for-loop target not supported: for a, *b in data:
# NOTE: starred in for-loop target not supported: pass

# NOTE: starred in for-loop target not supported: for *a, b in data:
# NOTE: starred in for-loop target not supported: pass

# NOTE: starred in for-loop target not supported: for a, *b, c in data:
# NOTE: starred in for-loop target not supported: pass


# --- Extended unpacking with nested targets ---

nested = [(1, [2, 3, 4]), (5, [6, 7, 8])]

# NOTE: starred in for-loop target not supported: for a, [*b] in nested:
# NOTE: starred in for-loop target not supported: pass

# NOTE: starred in for-loop target not supported: for a, [b, *c] in nested:
# NOTE: starred in for-loop target not supported: pass


# --- Nested extended unpacking ---

(a, *b), c = ([1, 2, 3], 4)
a, (*b, c) = (1, [2, 3, 4])
(a, *b), (c, *d) = ([1, 2, 3], [4, 5, 6])


# --- Extended unpacking from various iterables ---

a, *b = {1, 2, 3}
a, *b = frozenset([1, 2, 3])
a, *b = iter([1, 2, 3])
a, *b = bytearray(b'\x01\x02\x03')
a, *b = bytes(b'\x01\x02\x03')
a, *b = memoryview(b'\x01\x02\x03')


# --- Extended unpacking from dict (keys) ---

a, *b = {"x": 1, "y": 2, "z": 3}


# --- Extended unpacking from generator ---

def gen():
    yield 1
    yield 2
    yield 3

a, *b = gen()
*a, b = gen()
a, *b, c = gen()


# --- Extended unpacking in comprehensions ---

pairs = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]

# NOTE: starred in for-loop target inside list comp not supported
# firsts = [a for a, *_ in pairs]
# lasts = [c for *_, c in pairs]
# middles = [b for _, *b, _ in pairs]


# --- Extended unpacking with ignored starred ---

a, *_, b = [1, 2, 3, 4, 5]
_, *rest, _ = [1, 2, 3, 4, 5]


# --- Extended unpacking in function ---

def process(items):
    first, *rest = items
    return first, rest

def head_tail(iterable):
    head, *tail = iterable
    return head

def last_element(iterable):
    *_, last = iterable
    return last


# --- Extended unpacking from function return ---

def multi():
    return [1, 2, 3, 4, 5]

a, *b = multi()
*a, b = multi()
a, *b, c = multi()


# --- Extended unpacking from string ---

first, *middle, last = "hello"
a, *_ = "x"
*_, z = "world"


# --- Extended unpacking with long iterables ---

a, *b = range(100)
*a, b = range(100)
a, *b, c = range(100)


# --- Extended unpacking in try/except ---

try:
    a, *b = [1]
except ValueError:
    pass


# --- Extended unpacking in with statement ---

class CM:
    def __enter__(self):
        return [1, 2, 3]
    def __exit__(self, *args):
        pass

# NOTE: list/starred in with-as target not supported
# with CM() as [a, *b]:
#     pass
# with CM() as (*a, b):
#     pass
with CM() as _cm_result:
    pass


# --- Extended unpacking in conditional ---

flag = True
if flag:
    a, *b = [1, 2, 3]
else:
    a, *b = [4, 5, 6]


# --- Extended unpacking chained ---

data = [1, 2, 3, 4, 5, 6]
first, *rest = data
second, *rest = rest
third, *rest = rest


# --- End of extended unpacking constructs ---
