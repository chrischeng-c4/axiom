# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: type-introspection builtins (R5).
# type, isinstance, issubclass, callable

# type
print(type(42).__name__)
print(type(3.14).__name__)
print(type(True).__name__)
print(type(None).__name__)
print(type("hello").__name__)
print(type([1, 2]).__name__)
print(type((1,)).__name__)
print(type({1: 2}).__name__)
print(type({1, 2}).__name__)

# isinstance
print(isinstance(42, int))
print(isinstance(True, int))
print(isinstance(True, bool))
print(isinstance(3.14, float))
print(isinstance("hi", str))
print(isinstance([1, 2], list))
print(isinstance((1,), tuple))
print(isinstance({}, dict))
print(isinstance(42, (int, str)))
print(isinstance("hi", (int, str)))
print(isinstance(3.14, (int, str)))

# issubclass
print(issubclass(bool, int))
print(issubclass(int, float))
print(issubclass(list, object))
print(issubclass(str, str))

# callable
def my_func() -> None:
    pass

class MyCallable:
    def __call__(self) -> None:
        pass

print(callable(my_func))
print(callable(MyCallable()))
print(callable(42))
print(callable("hello"))
print(callable(len))
