# RUN: parse
# Extracted from CPython Lib/test/test_enumerate.py — syntax constructs only.
import operator
import sys


class G:
    'Sequence using __getitem__'
    def __init__(self, seqn):
        self.seqn = seqn
    def __getitem__(self, i):
        return self.seqn[i]

class I:
    'Sequence using iterator protocol'
    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= len(self.seqn): raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    'Sequence using iterator protocol defined with a generator'
    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0
    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    'Missing __getitem__ and __iter__'
    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0
    def __next__(self):
        if self.i >= len(self.seqn): raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    'Test propagation of exceptions'
    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        3 // 0

class N:
    'Iterator missing __next__()'
    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0
    def __iter__(self):
        return self


# Enumerate subclass
class MyEnum(enumerate):
    pass

# Basic usage
seq, res = 'abc', [(0, 'a'), (1, 'b'), (2, 'c')]
type(enumerate(seq))
e = enumerate(seq)
iter(e)
list(enumerate(seq))

# With various iterable types
list(enumerate(G(seq)))
list(enumerate(I(seq)))
list(enumerate(Ig(seq)))

# Keyword arguments
list(enumerate(iterable=Ig(seq)))
list(enumerate(iterable=Ig(seq), start=0))
list(enumerate(start=0, iterable=Ig(seq)))

# Large range
big_seq = range(10, 20000, 2)
big_res = list(zip(range(20000), big_seq))

# Reversed protocol
class SeqForReversed:
    def __getitem__(self, i):
        if i < 5:
            return str(i)
        raise StopIteration
    def __len__(self):
        return 5

for data in ('abc', range(5), tuple(enumerate('abc')), SeqForReversed(),
             range(1, 17, 5), dict.fromkeys('abcde')):
    list(data)[::-1]
    list(reversed(data))

# Range optimization
x = range(1)
type(reversed(x))
type(iter(x))

# Length hint
for s in ('hello', tuple('hello'), list('hello'), range(5)):
    operator.length_hint(reversed(s))
    r = reversed(s)
    list(r)
    operator.length_hint(r)

class SeqWithWeirdLen:
    called = False
    def __len__(self):
        if not self.called:
            self.called = True
            return 10
        raise ZeroDivisionError
    def __getitem__(self, index):
        return index

# GC interaction
class SeqForGC:
    def __len__(self):
        return 10
    def __getitem__(self, index):
        return index

s = SeqForGC()
r = reversed(s)
s.r = r

# Enumerate with start offset
def enum_with_start(iterable, start=11):
    return enumerate(iterable, start=start)

enum_with_start('abc')

# Long start value
def enum_long_start(iterable, start=sys.maxsize + 1):
    return enumerate(iterable, start=start)

enum_long_start('abc')
