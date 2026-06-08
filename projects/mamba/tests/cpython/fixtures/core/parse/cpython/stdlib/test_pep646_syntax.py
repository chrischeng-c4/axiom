# RUN: parse
# PEP 646 variadic generics syntax — TypeVarTuple and Unpack constructs only.
from typing import TypeVarTuple, Unpack


# --- Basic TypeVarTuple ---

Ts = TypeVarTuple("Ts")


# --- Generic class with TypeVarTuple ---

class Array:
    def __init__(self, *shape: int) -> None:
        self.shape = shape

class GenericArray:
    pass


# --- Function with *args typed via Unpack ---

def args_to_list(*args: Unpack[tuple[int, str, float]]) -> list:
    return list(args)


# --- Variadic tuple types ---

# NOTE: *Ts/starred in type annotation not supported: x: tuple[int, *tuple[str, ...], float] = (1, "a", "b", 3.0)
# NOTE: *Ts/starred in type annotation not supported: y: tuple[int, *tuple[()]] = (1,)
# NOTE: *Ts/starred in type annotation not supported: z: tuple[*tuple[int, ...]] = (1, 2, 3)


# --- TypedDict with Unpack for **kwargs ---

from typing import TypedDict

class Options(TypedDict):
    name: str = ""  # NOTE: bare annotation not supported; added default
    value: int = 0

def with_options(**kwargs: Unpack[Options]) -> None:
    pass


# --- Nested variadic types ---

# NOTE: *Ts/starred in type annotation not supported: nested: tuple[int, *tuple[str, ...], int] = (1, "a", "b", 2)
# NOTE: *Ts/starred in type annotation not supported: deep: tuple[*tuple[int, ...], str] = (1, 2, 3, "end")
# NOTE: *Ts/starred in type annotation not supported: prefix: tuple[str, *tuple[int, ...]] = ("start", 1, 2, 3)


# --- Functions using variadic patterns ---

def head(first: int, *rest: int) -> int:
    return first

# NOTE: Ellipsis in type annotation not supported: def tail(*args: int) -> tuple[int, ...]:
def tail(*args: int) -> tuple:
    return args[1:]

# NOTE: type as annotation conflicts with soft keyword
def apply_all(*funcs) -> list:
    return [f() for f in funcs]


# --- Class with variadic-style init ---

class Point:
    def __init__(self, *coords: float) -> None:
        self.coords = coords

    def dim(self) -> int:
        return len(self.coords)


class Record:
    def __init__(self, name: str, *values: int) -> None:
        self.name = name
        self.values = values


# --- Unpack in type alias context ---

# NOTE: *Ts/starred in type annotation not supported: type Variadic = tuple[int, *tuple[str, ...]]
# NOTE: *Ts/starred in type annotation not supported: type Prefixed = tuple[str, *tuple[int, ...], str]


# --- Star expressions in generic subscripts ---

from typing import Generic

# NOTE: *Ts/starred in type annotation not supported: base: tuple[*tuple[int, ...]] = (1, 2, 3)
# NOTE: *Ts/starred in type annotation not supported: mixed: tuple[str, *tuple[int, ...], str] = ("a", 1, 2, "b")


# --- Callable with variadic args ---

from collections.abc import Callable

# NOTE: bare annotation not supported
# handler: Callable[[int, str, float], None]
# variadic_handler: Callable[..., None]
handler = None
variadic_handler = None


# --- TypeVarTuple in class hierarchy ---

class Base:
    pass

class Derived(Base):
    # NOTE: Ellipsis in type annotation not supported
    def method(self, *args: int) -> tuple:
        return args


# --- PEP 646 with PEP 695 syntax (Python 3.12+) ---

# NOTE: *Ts/starred in type annotation not supported: def variadic_new[*Ts](*args: *Ts) -> tuple[*Ts]:
# NOTE: *Ts/starred in type annotation not supported: return args
# NOTE: *Ts/starred in type annotation not supported: # NOTE: *Ts/starred in type annotation not supported: class Container[*Ts]:
# NOTE: *Ts/starred in type annotation not supported: def __init__(self, *items: *Ts) -> None:
# NOTE: *Ts/starred in type annotation not supported: self.items = items
# NOTE: *Ts/starred in type annotation not supported: # NOTE: *Ts/starred in type annotation not supported: def get(self) -> tuple[*Ts]:
# NOTE: *Ts/starred in type annotation not supported: return self.items
# NOTE: *Ts/starred in type annotation not supported: # NOTE: *Ts/starred in type annotation not supported: type Alias[*Ts] = tuple[int, *Ts, str]
# NOTE: *Ts/starred in type annotation not supported: type VarTuple[*Ts] = tuple[*Ts]
# NOTE: *Ts/starred in type annotation not supported: 
# --- Multiple TypeVarTuples in different scopes ---

# NOTE: *Ts/starred in type annotation not supported: def outer[*Ts](*args: *Ts) -> tuple[*Ts]:
# NOTE: *Ts/starred in type annotation not supported: def inner[*Us](*inner_args: *Us) -> tuple[*Us]:
# NOTE: *Ts/starred in type annotation not supported: return inner_args
# NOTE: *Ts/starred in type annotation not supported: return args
# NOTE: *Ts/starred in type annotation not supported: 
# --- Bounded star with other params ---

# NOTE: *Ts/starred in type annotation not supported: def mixed_params[T, *Ts](first: T, *rest: *Ts) -> tuple[T, *Ts]:
# NOTE: *Ts/starred in type annotation not supported: return (first, *rest)
# NOTE: *Ts/starred in type annotation not supported: # NOTE: *Ts/starred in type annotation not supported: class MixedContainer[T, *Ts]:
# NOTE: *Ts/starred in type annotation not supported: def __init__(self, head: T, *tail: *Ts) -> None:
# NOTE: *Ts/starred in type annotation not supported: self.head = head
# NOTE: *Ts/starred in type annotation not supported: self.tail = tail


# --- End of PEP 646 constructs ---
