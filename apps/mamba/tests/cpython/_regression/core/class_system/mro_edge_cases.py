# Class system edge cases

# Diamond MRO
class A:
    pass

class B(A):
    pass

class C(A):
    pass

class D(B, C):
    pass

print([cls.__name__ for cls in D.__mro__])
