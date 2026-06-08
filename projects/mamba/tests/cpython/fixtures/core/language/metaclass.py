# Language conformance: metaclass= keyword (P2-R2).
# Tests metaclass association storage and __call__ routing.

# TC-2.1: Metaclass __call__ intercepts instantiation.
# Meta.__call__ receives the class name as its first argument (cls),
# creates the instance via runtime, sets a flag, and returns it.
class Meta(type):
    def __call__(cls):
        print("Meta.__call__ invoked")
        obj = super().__call__()
        obj.from_meta = True
        return obj

class Foo(metaclass=Meta):
    def __init__(self):
        self.x = 42

# Meta.__call__ is invoked and returns the created instance.
obj = Foo()
print(obj.x)
print(obj.from_meta)

# TC-2.2: Class without metaclass works normally.
class Bar:
    def __init__(self):
        self.y = 99

b = Bar()
print(b.y)

# TC-2.3: Metaclass __call__ that returns a custom value.
class Factory(type):
    def __call__(cls):
        print("Factory creating")
        return 100

class Widget(metaclass=Factory):
    pass

w = Widget()
print(w)
