# RUN: parse
# Extracted from CPython Lib/test/test_bool.py — syntax constructs only.
import os


# Subclass restriction
try:
    class C(bool):
        pass
except TypeError:
    pass

# Repr/str
repr(False)
repr(True)
str(False)
str(True)

# Type conversions
int(False)
int(True)
float(False)
float(True)
complex(False)
complex(True)

# Unary operators on bools
+False
-False
abs(False)
+True
-True
abs(True)

# Arithmetic with bools
False + 2
True + 2
2 + False
2 + True
False + False
False + True
True + False
True + True
True - True
False - False
True - False
False - True
True * 1
False * 1
True / 1
False / 1
True % 1
True % 2
False % 1

for b in False, True:
    for i in 0, 1, 2:
        b ** i
        int(b) ** i

for a in False, True:
    for b in False, True:
        a & b
        a | b
        a ^ b
        a & int(b)
        a | int(b)
        a ^ int(b)
        int(a) & b
        int(a) | b
        int(a) ^ b

# Comparison operators
1 == 1
1 == 0
0 < 1
1 < 0
0 <= 0
1 <= 0
1 > 0
1 > 1
1 >= 1
0 >= 1
0 != 1
0 != 0

# Identity and membership
x = [1]
x is x
x is not x
1 in x
0 in x
1 not in x
0 not in x

x = {1: 2}
x is x
x is not x
1 in x
0 in x
1 not in x
0 not in x
not True
not False

# bool() constructor
bool(10)
bool(1)
bool(-1)
bool(0)
bool("hello")
bool("")
bool()

# Format strings
"%d" % False
"%d" % True
"%x" % False
"%x" % True

# Built-in predicates
hasattr([], "append")
hasattr([], "wobble")
callable(len)
callable(1)
isinstance(True, bool)
isinstance(False, bool)
isinstance(True, int)
issubclass(bool, int)
issubclass(int, bool)
1 in {}
1 in {1: 1}

# String methods returning bools
"xyz".endswith("z")
"xyz0123".isalnum()
"xyz".isalpha()
"0123".isdigit()
"xyz".islower()
"0123".isdecimal()
"0123".isnumeric()
" ".isspace()
"\xa0".isspace()
"\u3000".isspace()
"X".istitle()
"XYZ".isupper()
"xyz".startswith("x")

# Boolean bitwise
True & 1
True & True
True | 1
True | True
True ^ 1
True ^ True

# __bool__ override classes
class Foo(object):
    def __bool__(self):
        return self

class Bar(object):
    def __bool__(self):
        return "Yes"

class Baz(int):
    def __bool__(self):
        return self

class Spam(int):
    def __bool__(self):
        return 1

class Eggs:
    def __len__(self):
        return -1

# SymbolicBool pattern — __bool__ raising TypeError
class SymbolicBool:
    def __bool__(self):
        raise TypeError

class Symbol:
    def __gt__(self, other):
        return SymbolicBool()

sym = Symbol()

# __len__ with bad values
for badval in ['illegal', -1, 1 << 32]:
    class BadLen:
        def __len__(self):
            return badval
    try:
        bool(BadLen())
    except Exception:
        pass

# __bool__ = None blocking
class Blocked:
    __bool__ = None

class BlockedWithLen:
    def __len__(self):
        return 10
    __bool__ = None

# Bool attributes
True.real
True.imag
type(True.real)
type(True.imag)
False.real
False.imag

# bool.__new__
bool.__new__(bool)
bool.__new__(bool, 1)
bool.__new__(bool, 0)
bool.__new__(bool, False)
bool.__new__(bool, True)

# from_bytes
bool.from_bytes(b'\x00' * 8, 'big')
bool.from_bytes(b'abcd', 'little')

# Conditional with custom __bool__
class Counter:
    def __init__(self):
        self.count = 0
    def __bool__(self):
        self.count += 1
        return True

def check(x):
    if x or True:
        pass

# Types are always true
for t in [bool, complex, dict, float, int, list, object,
          set, str, tuple, type]:
    bool(t)
