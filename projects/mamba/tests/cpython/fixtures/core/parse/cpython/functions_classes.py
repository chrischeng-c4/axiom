# RUN: parse
# CPython-derived: functions, classes, decorators, parameters

# --- basic function with types ---
def greet(name: str) -> str:
    return name

# --- function with untyped params (defaults to Any) ---
def hello(name):
    return name

# --- function with default parameter ---
def add(a: int, b: int = 0) -> int:
    return a + b

# --- untyped with default ---
def inc(x, step=1):
    return x + step

# --- function with *args (typed) ---
def variadic(*args: int) -> int:
    return 0

# --- function with *args (untyped) ---
def variadic2(*args):
    return 0

# --- function with **kwargs (typed) ---
def keyword_fn(**kwargs: str) -> int:
    return 0

# --- function with **kwargs (untyped) ---
def keyword_fn2(**kwargs):
    return 0

# --- function with all param kinds (untyped) ---
def complex_fn(a, b=1, *args, **kwargs):
    return a

# --- function without return type ---
def no_return(x: int):
    pass

# --- untyped function, no return type ---
def bare(x):
    pass

# --- nested function ---
def outer(x: int) -> int:
    def inner(y: int) -> int:
        return y
    return inner(x)

# --- decorator on function ---
@decorator
def decorated() -> int:
    return 0

# --- decorator with arguments ---
@decorator(arg1, arg2)
def decorated_with_args() -> int:
    return 0

# --- multiple decorators ---
@first
@second
@third
def multi_decorated() -> int:
    return 0

# --- class definition ---
class Animal:
    name: str = ""

# --- class with base ---
class Dog(Animal):
    breed: str = ""

# --- class with methods (self is auto-typed) ---
class Calculator:
    value: int = 0
    def add(self, x: int) -> int:
        return x
    def reset(self) -> int:
        return 0

# --- class with untyped methods ---
class Simple:
    data: int = 0
    def process(self, x):
        return x

# --- class with decorator ---
@decorator
class DecoratedClass:
    pass

# --- class with type parameters (PEP 695) ---
class Stack[T]:
    items: int = 0
    def push(self, item: int) -> int:
        return 0
    def pop(self) -> int:
        return 0
