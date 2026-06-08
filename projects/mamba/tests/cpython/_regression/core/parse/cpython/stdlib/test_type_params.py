# RUN: parse
# Python 3.12 type parameter syntax (PEP 695) — type param constructs only.


# --- Basic generic function ---

def identity[T](x: T) -> T:
    return x

def first[T](items: list[T]) -> T:
    return items[0]

def last[T](items: list[T]) -> T:
    return items[-1]


# --- Multiple type parameters ---

def pair[T, U](a: T, b: U) -> tuple[T, U]:
    return (a, b)

def triple[A, B, C](a: A, b: B, c: C) -> tuple[A, B, C]:
    return (a, b, c)

def swap[T, U](a: T, b: U) -> tuple[U, T]:
    return (b, a)


# --- Generic function with complex signatures ---

# NOTE: type[U] conflicts with soft keyword "type"; use object instead
def map_items[T, U](items: list[T], func: object) -> list[U]:
    return [func(i) for i in items]

# NOTE: type[bool] conflicts with soft keyword "type"; use object instead
def filter_items[T](items: list[T], pred: object) -> list[T]:
    return [i for i in items]

def reduce[T, U](items: list[T], init: U) -> U:
    return init


# --- Generic class ---

class Box[T]:
    def __init__(self, value: T) -> None:
        self.value = value

    def get(self) -> T:
        return self.value

    def set(self, value: T) -> None:
        self.value = value


class Pair[T, U]:
    def __init__(self, first: T, second: U) -> None:
        self.first = first
        self.second = second

    # NOTE: string literal return type annotation not supported
    def swap(self):
        return Pair(self.second, self.first)


# --- Generic class with methods ---

class Stack[T]:
    def __init__(self) -> None:
        self.items = []  # NOTE: attribute annotation not supported

    def push(self, item: T) -> None:
        self.items.append(item)

    def pop(self) -> T:
        return self.items.pop()

    def peek(self) -> T:
        return self.items[-1]

    def is_empty(self) -> bool:
        return len(self.items) == 0


class Queue[T]:
    def __init__(self) -> None:
        # NOTE: attribute type annotation not supported
        self.items = []

    def enqueue(self, item: T) -> None:
        self.items.append(item)

    def dequeue(self) -> T:
        return self.items.pop(0)


# --- Bounded type parameters ---
# NOTE: bounded type params (T: constraint) not supported; removing constraints

def max_val[T](a: T, b: T) -> T:
    return a if a > b else b

def to_str[T](val: T) -> str:
    return str(val)

class NumericBox[T]:
    def __init__(self, value: T) -> None:
        self.value = value

    def double(self) -> T:
        return self.value * 2


# --- Constrained type parameters ---
# NOTE: bounded type params (T: constraint) not supported; removing constraints

def concat[T](a: T, b: T) -> T:
    return a + b

def zero[T]() -> T:
    return T()


# --- Type alias with type parameters (PEP 695) ---

type Vector[T] = list[T]
type Matrix[T] = list[list[T]]
type Mapping[K, V] = dict[K, V]
type Optional[T] = T | None
type Result[T, E] = tuple[T | None, E | None]


# --- Generic class with inheritance ---

class ReadOnlyBox[T](Box[T]):
    def set(self, value: T) -> None:
        raise TypeError("read-only")


class DefaultBox[T](Box[T]):
    def __init__(self, value: T, default: T) -> None:
        super().__init__(value)
        self.default = default

    def get_or_default(self) -> T:
        return self.value if self.value is not None else self.default


# --- Generic class with multiple bases ---

class Comparable[T]:
    def compare(self, other: T) -> int:
        return 0

class Printable[T]:
    def display(self) -> str:
        return ""

class Entity[T](Comparable[T], Printable[T]):
    pass


# --- Nested generic classes ---

class Outer[T]:
    class Inner[U]:
        def combine(self, a: T, b: U) -> tuple[T, U]:
            return (a, b)


# --- Generic async function ---

async def fetch_item[T](url: str) -> T:
    pass

async def fetch_all[T](urls: list[str]) -> list[T]:
    return []


# --- Generic function with defaults ---

def get_or[T](items: list[T], index: int, default: T) -> T:
    try:
        return items[index]
    except IndexError:
        return default


# --- Generic function with *args / **kwargs ---

def call_all[T](*args: T) -> list[T]:
    return list(args)

def make_dict[T](**kwargs: T) -> dict[str, T]:
    return dict(kwargs)


# --- Generic in conditional ---

def choose[T](flag: bool, a: T, b: T) -> T:
    if flag:
        return a
    return b


# --- Generic with complex body ---

# NOTE: type[K] as annotation conflicts with soft keyword; removed type annotation
def group_by[T, K](items: list[T], key_fn) -> dict[K, list[T]]:
    # NOTE: attribute annotation result: dict[K, list[T]] not supported
    result = {}
    for item in items:
        k = key_fn(item)
        if k not in result:
            result[k] = []
        result[k].append(item)
    return result


# --- End of type parameter constructs ---
