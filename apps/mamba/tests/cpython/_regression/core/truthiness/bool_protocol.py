# mamba-xfail: invalid __bool__ return does not raise TypeError on mamba (#2802 TypeError clause)
# Truthiness via __bool__ / __len__ protocol — #2802.
#
# Covers fallback order:
#   1. __bool__ wins if defined (must return bool).
#   2. __len__ used if __bool__ absent (0 -> False, nonzero -> True).
#   3. With neither, instances are unconditionally truthy.
# Plus invalid __bool__ return -> TypeError (probed without brittle message).

# 1. __bool__ defined -> wins.
class HasBoolTrue:
    def __bool__(self):
        return True

class HasBoolFalse:
    def __bool__(self):
        return False

print("bool(HasBoolTrue)=", bool(HasBoolTrue()))
print("bool(HasBoolFalse)=", bool(HasBoolFalse()))

if HasBoolTrue():
    print("if-branch: HasBoolTrue truthy [truthiness]")
else:
    print("if-branch: HasBoolTrue falsy [truthiness FAIL]")

if HasBoolFalse():
    print("if-branch: HasBoolFalse truthy [truthiness FAIL]")
else:
    print("if-branch: HasBoolFalse falsy [truthiness]")

# 2. __len__ only -> fallback used (0 -> False, nonzero -> True).
class LenZero:
    def __len__(self):
        return 0

class LenThree:
    def __len__(self):
        return 3

print("bool(LenZero)=", bool(LenZero()))
print("bool(LenThree)=", bool(LenThree()))

if LenZero():
    print("if-branch: LenZero truthy [truthiness FAIL]")
else:
    print("if-branch: LenZero falsy [truthiness]")

if LenThree():
    print("if-branch: LenThree truthy [truthiness]")
else:
    print("if-branch: LenThree falsy [truthiness FAIL]")

# 3. Both defined -> __bool__ wins, __len__ ignored.
class BoolWinsOverLen:
    def __bool__(self):
        return False
    def __len__(self):
        return 7  # non-zero, would be truthy if __len__ used

print("bool(BoolWinsOverLen)=", bool(BoolWinsOverLen()))
if BoolWinsOverLen():
    print("if-branch: BoolWinsOverLen truthy [truthiness FAIL: __len__ overrode __bool__]")
else:
    print("if-branch: BoolWinsOverLen falsy [truthiness: __bool__ won]")

# 4. Neither defined -> instance is truthy.
class Neither:
    pass

print("bool(Neither)=", bool(Neither()))
if Neither():
    print("if-branch: Neither truthy [truthiness]")
else:
    print("if-branch: Neither falsy [truthiness FAIL]")

# 5. Invalid __bool__ return -> TypeError. Probed without brittle message;
#    we only check that an exception is raised (any TypeError-shaped one).
class BadBool:
    def __bool__(self):
        return "not a bool"

raised = False
try:
    bool(BadBool())
except TypeError:
    raised = True
print("invalid __bool__ raises TypeError=", raised, "[truthiness]")
