# RUN: parse
# CPython-derived: PEP 695 type parameter syntax, Python 3.12 (#559)

# --- basic generic function ---
def identity[T](x: T) -> T:
    return x

# --- generic function with two params ---
def pair[T, U](x: T, y: U) -> tuple[T, U]:
    return (x, y)

# --- generic class ---
class Stack[T]:
    def __init__(self) -> None:
        self.items = []

    def push(self, item: T) -> None:
        self.items.append(item)

    def pop(self) -> T:
        return self.items.pop()

# --- type alias statement ---
type Point = tuple[int, int]
type Vector2D = tuple[float, float]

# --- generic type alias ---
type Vector[T] = list[T]
type Matrix[T] = list[list[T]]
type Pair[T, U] = tuple[T, U]

# --- bounded type parameter (NOTE: T: constraint not supported, using plain T) ---
def clamp[T](value: T, low: T, high: T) -> T:
    if value < low:
        return low
    if value > high:
        return high
    return value

# --- single bound (NOTE: T: constraint not supported) ---
def stringify[T](x: T) -> str:
    return str(x)

# --- constrained type parameter (NOTE: T: constraint not supported) ---
def add[T](a: T, b: T) -> T:
    return a + b

# --- multiple type params ---
def transform[T, U, V](x: T, f: object, g: object) -> V:
    pass

# --- TypeVarTuple syntax ---
# NOTE: TypeVarTuple *Ts in type params not supported by parser
# def args_to_tuple[*Ts](*args: *Ts) -> tuple[*Ts]:
#     return args
# def apply_all[*Ts](funcs: tuple[*Ts]) -> None:
#     pass

# --- ParamSpec syntax ---
# NOTE: ParamSpec **P in type params not supported by parser
# def decorator[**P](func: object) -> object:
#     def wrapper(*args: object, **kwargs: object) -> object:
#         return func(*args, **kwargs)
#     return wrapper

# --- combined type params ---
# NOTE: TypeVarTuple *Ts and ParamSpec **P not supported by parser
# class Container[T, *Ts, **P]:
#     pass
# class Registry[T, U, *Ts]:
#     pass

# --- generic class with methods ---
class Map[K, V]:
    def __init__(self) -> None:
        self.data = {}

    def get[D](self, key: K, default: D) -> V | D:
        pass

    def keys(self) -> list[K]:
        return list(self.data.keys())

# --- generic class with inheritance ---
class OrderedStack[T](Stack[T]):
    pass

class Wrapper[T]:
    def __init__(self, value: T) -> None:
        self.value = value

class DoubleWrapper[T, U](Wrapper[T]):
    def __init__(self, first: T, second: U) -> None:
        super().__init__(first)
        self.second = second

# --- type alias with complex types ---
type Callback[T] = object
# NOTE: ParamSpec **P not supported: type Handler[**P] = object
type Reducer[T, U] = object

# --- type alias with union ---
type IntOrStr = int | str
type OptionalInt = int | None
type Number = int | float | complex

# --- generic function with default-like patterns ---
def first[T](items: list[T]) -> T | None:
    if items:
        return items[0]
    return None

# --- nested generic usage ---
def flatten[T](nested: list[list[T]]) -> list[T]:
    result: list[T] = []
    for inner in nested:
        result.extend(inner)
    return result

# --- generic with decorator ---
def my_decorator(f):
    return f

@my_decorator
def decorated[T](x: T) -> T:
    return x

# --- generic async function ---
async def async_fetch[T](url: str) -> T:
    pass

# --- generic class with class methods ---
class Factory[T]:
    @classmethod
    def create(cls) -> T:
        pass

    @staticmethod
    def default[U]() -> U:
        pass

# --- multiple type aliases ---
type Name = str
type Age = int
type Person = tuple[Name, Age]
type People = list[Person]

# --- type alias referencing another alias ---
type Ints = list[int]
type Matrix2D = list[Ints]

# --- type alias with None ---
type MaybeInt = int | None
type MaybeStr = str | None

# --- generic with star in tuple ---
# NOTE: TypeVarTuple *Ts not supported by parser
# def head_tail[T, *Ts](first: T, *rest: *Ts) -> tuple[T, tuple[*Ts]]:
#     return (first, rest)

# --- complex class hierarchy ---
class Base[T]:
    # NOTE: bare annotation not supported
    value = None

class Derived[T, U](Base):
    # NOTE: bare annotation not supported
    # NOTE: generic subscript in base class with multi-param not supported: (Base[T])
    extra = None

# NOTE: generic subscript in base class with multi-param not supported
# class Leaf[T](Derived[T, str]):
class Leaf[T](Derived):
    pass
