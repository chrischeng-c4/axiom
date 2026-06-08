# Dunder conformance: __str__ vs __repr__ dispatch.
class MyClass:
    def __str__(self):
        return 'MyClass str'

    def __repr__(self):
        return 'MyClass repr'

obj = MyClass()
print(str(obj))
print(repr(obj))

class NoStr:
    def __repr__(self):
        return 'NoStr repr'

obj2 = NoStr()
print(str(obj2))
print(repr(obj2))
