# RUN: parse
# Extracted from CPython Lib/test/test_operator.py — syntax constructs only.


class Seq1:
    def __init__(self, lst):
        self.lst = lst
    def __len__(self):
        return len(self.lst)
    def __getitem__(self, i):
        return self.lst[i]
    def __add__(self, other):
        return self.lst + other.lst
    def __mul__(self, other):
        return self.lst * other
    def __rmul__(self, other):
        return other * self.lst

class Seq2(object):
    def __init__(self, lst):
        self.lst = lst
    def __len__(self):
        return len(self.lst)
    def __getitem__(self, i):
        return self.lst[i]
    def __add__(self, other):
        return self.lst + other.lst
    def __mul__(self, other):
        return self.lst * other
    def __rmul__(self, other):
        return other * self.lst

class BadIterable:
    def __iter__(self):
        raise ZeroDivisionError


# Nested class with __eq__ override
class C(object):
    def __eq__(self, other):
        raise SyntaxError

# Matmul operator
class M:
    def __matmul__(self, other):
        return other - 1

M() @ 42

# Inplace dunder methods
class InplaceOps(object):
    def __iadd__(self, other): return "iadd"
    def __iand__(self, other): return "iand"
    def __ifloordiv__(self, other): return "ifloordiv"
    def __ilshift__(self, other): return "ilshift"
    def __imod__(self, other): return "imod"
    def __imul__(self, other): return "imul"
    def __imatmul__(self, other): return "imatmul"
    def __ior__(self, other): return "ior"
    def __ipow__(self, other): return "ipow"
    def __irshift__(self, other): return "irshift"
    def __isub__(self, other): return "isub"
    def __itruediv__(self, other): return "itruediv"
    def __ixor__(self, other): return "ixor"
    def __getitem__(self, other): return 5

# __length_hint__ protocol
class LengthHint(object):
    def __init__(self, value):
        self.value = value
    def __length_hint__(self):
        if type(self.value) is type:
            raise self.value
        else:
            return self.value

# Attrgetter patterns — recursive attribute access
class A:
    pass

a = A()
a.name = 'arthur'
a.child = A()
a.child.name = 'thomas'
a.child.child = A()
a.child.child.name = 'johnson'

# __getattr__ override
class AttrOverride(object):
    def __getattr__(self, name):
        raise SyntaxError

# Tuple subclass
class T(tuple):
    'Tuple subclass'
    pass

# Itemgetter patterns
a_str = 'ABCDE'
t = tuple('abcde')
t[-1]
t[2:4]

# Variadic function
def func(*args, **kwargs):
    return args, kwargs

func(0, 1, a=2, obj=3)

# Method caller patterns
class MethodHost:
    def foo(self, *args, **kwds):
        return args[0] + args[1]
    def bar(self, f=42):
        return f
    def baz(*args, **kwds):
        return kwds['name'], kwds['self']

# Operator expressions
nan = float("nan")
d = dict(key='val')
inventory = [('apple', 3), ('banana', 2), ('pear', 5), ('orange', 1)]
data = list(map(str, range(20)))

# is / is not
a_ref = b_ref = 'xyzpdq'
c_ref = a_ref[:3] + b_ref[3:]
a_ref is b_ref
a_ref is not c_ref

# Sequence operations
it = iter('leave the iterator at exactly the position after the match')
next(it)

# __ne__ override
class NeOverride(object):
    def __ne__(self, other):
        raise SyntaxError

# __bool__ override
class BoolOverride(object):
    def __bool__(self):
        raise SyntaxError

# __getitem__ override
class GetItemOverride(object):
    def __getitem__(self, name):
        raise SyntaxError
