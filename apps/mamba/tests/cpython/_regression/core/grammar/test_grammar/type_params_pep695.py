# RUN: parse

# PEP 695: Generic functions
def first[T](items: list[T]) -> T:
    return items[0]

def pair[T, U](a: T, b: U) -> tuple[T, U]:
    return (a, b)

# PEP 695: Generic classes
class Stack[T]:
    def __init__(self) -> None:
        self._items: list[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> T:
        return self._items.pop()

class Pair[T, U]:
    first: T
    second: U

# PEP 695: Type aliases with parameters
type Vector[T] = list[T]
type Matrix[T] = list[list[T]]
type Pair[T, U] = tuple[T, U]

# PEP 695: Bounded type parameters
type IntLike[T: int] = list[T]
type Sortable[T: (int, float, str)] = list[T]

# PEP 695: ParamSpec and TypeVarTuple
type Callback[**P] = Callable[P, None]
type Shape[*Ts] = tuple[*Ts]

# PEP 695: Generic class with bases and keywords
class MyList[T](list[T]):
    pass

class Concrete[T: int](list[T], metaclass=ABCMeta):
    pass
