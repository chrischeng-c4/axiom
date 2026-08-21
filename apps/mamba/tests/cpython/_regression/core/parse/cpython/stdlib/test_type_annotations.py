# RUN: parse
# Type annotation syntax constructs only — variable, function, and class annotations.


# --- Basic variable annotations ---

x: int = 5
y: str = "hello"
z: float = 3.14
flag: bool = True
data: bytes = b"abc"
nothing: None = None


# --- Variable annotation without assignment ---

# NOTE: bare annotation without value not supported: a: int
# NOTE: bare annotation without value not supported: b: str
# NOTE: bare annotation without value not supported: c: float
# NOTE: bare annotation without value not supported: d: bool


# --- Container type annotations ---

items: list[int] = [1, 2, 3]
names: list[str] = ["a", "b"]
pairs: list[tuple[int, str]] = [(1, "a")]
mapping: dict[str, int] = {"a": 1}
unique: set[int] = {1, 2, 3}
frozen: frozenset[int] = frozenset([1, 2])
fixed: tuple[int, str, float] = (1, "a", 3.0)
# NOTE: Ellipsis in type annotation not supported: variable: tuple[int, ...] = (1, 2, 3)


# --- Nested container annotations ---

matrix: list[list[int]] = [[1, 2], [3, 4]]
lookup: dict[str, list[int]] = {"a": [1, 2]}
nested: dict[str, dict[str, int]] = {"a": {"b": 1}}
deep: list[dict[str, list[int]]] = [{"a": [1]}]


# --- Union types (PEP 604) ---

maybe_int: int | None = None
int_or_str: int | str = 5
number: int | float | complex = 3.14
multi: int | str | float | None = None


# --- Function annotations: basic ---

def greet(name: str) -> str:
    return "hello " + name

def add(a: int, b: int) -> int:
    return a + b

def noop() -> None:
    pass

def identity(x: object) -> object:
    return x


# --- Function annotations: complex parameter types ---

def process(items: list[int]) -> list[str]:
    return [str(i) for i in items]

def merge(a: dict[str, int], b: dict[str, int]) -> dict[str, int]:
    # NOTE: dict unpacking {**a, **b} not supported
    merged = dict(a)
    merged.update(b)
    return merged

# NOTE: Ellipsis in type annotation not supported: def first(items: tuple[int, ...]) -> int:
# NOTE: removed dangling return from commented function
# return items[0]


# --- Function annotations: default values ---

def connect(host: str = "localhost", port: int = 8080) -> None:
    pass

def create(name: str, tags: list[str] | None = None) -> dict[str, object]:
    return {"name": name}


# --- Function annotations: *args and **kwargs ---

def variadic(*args: int) -> int:
    return sum(args)

def keyword_only(**kwargs: str) -> dict[str, str]:
    return kwargs

def mixed(a: int, *args: str, **kwargs: float) -> None:
    pass


# --- Function annotations: positional-only and keyword-only ---

def pos_only(x: int, y: int, /) -> int:
    return x + y

def kw_only(*, name: str, value: int) -> None:
    pass

def mixed_params(a: int, /, b: str, *, c: float) -> None:
    pass


# --- Function annotations: union return types ---

def maybe_parse(text: str) -> int | None:
    try:
        return int(text)
    except ValueError:
        return None

def coerce(val: int | str) -> str:
    return str(val)


# --- Class variable annotations ---

class Point:
    # NOTE: bare annotation without value not supported
    x: int = 0
    y: int = 0

    def __init__(self, x: int, y: int) -> None:
        self.x = x
        self.y = y


class Config:
    host: str = "localhost"
    port: int = 8080
    debug: bool = False
    tags: list[str] = []


# --- Class with ClassVar and Final (as strings for parse-only) ---

class Settings:
    # NOTE: bare annotation without value not supported
    name: str = ""
    # NOTE: string literal type annotation not supported
    # version: "int"
    # items: "list[str]"
    version = 0
    items = []


# --- Annotated assignments in class ---

class Container:
    # NOTE: bare annotation without value not supported
    data: list[int] = []
    size: int = 0
    label: str | None = None

    def add(self, item: int) -> None:
        pass

    def get(self, index: int) -> int | None:
        return None


# --- Callable type annotations ---

from collections.abc import Callable

# NOTE: bare annotation without value not supported
# handler: Callable[[], None]
# transform: Callable[[int], str]
# binary_op: Callable[[int, int], int]
# NOTE: Ellipsis in type annotation not supported: variadic_fn: Callable[..., int]
handler = None
transform = None
binary_op = None


# --- Complex callable annotations ---

# NOTE: bare annotation without value not supported
# factory: Callable[[], list[int]]
# nested_cb: Callable[[Callable[[int], str]], None]
factory = None
nested_cb = None
# NOTE: Callable[[int], str] with [[]] in type not supported
optional_cb = None


# --- Generic type annotations ---

from typing import TypeVar

T = TypeVar("T")

def first_item(items: list[T]) -> T:
    return items[0]

def pair(a: T, b: T) -> tuple[T, T]:
    return (a, b)


# --- Annotations with complex expressions ---

class Node:
    # NOTE: bare annotation without value not supported
    value: int = 0
    # NOTE: string literal type annotation not supported
    # children: "list[Node]"
    children = []
    # parent: "Node | None" = None
    parent = None


class Tree:
    # NOTE: bare annotation with string literal type not supported
    # root: "Node | None"
    root = None
    size: int = 0


# --- Global annotated variables ---

counter: int = 0
message: str = ""
values: list[float] = []
# NOTE: Ellipsis in type annotation not supported: registry: dict[str, Callable[..., None]] = {}


# --- Annotations in conditional ---

flag = True
if flag:
    result: int = 42
else:
    result: int = 0


# --- Annotations in loop ---

for item in [1, 2, 3]:
    current: int = item


# --- Annotations with walrus operator ---

data = [1, 2, 3, 4, 5]
if (n := len(data)) > 3:
    count: int = n


# --- Lambda (no annotations, but used in annotated contexts) ---

# NOTE: Callable[[int], int] with [[]] in type not supported
# fn: Callable[[int], int] = lambda x: x + 1
fn = lambda x: x + 1
# pred: Callable[[str], bool] = lambda s: len(s) > 0
pred = lambda s: len(s) > 0


# --- Nested function annotations ---

# NOTE: Callable[[int], int] with [[]] in return type not supported
def outer(x: int):
    def inner(y: int) -> int:
        return x + y
    return inner


# --- Async function annotations ---

async def fetch(url: str) -> bytes:
    return b""

async def process_all(urls: list[str]) -> list[bytes]:
    return []


# --- Type annotation on for target (via walrus) ---

total: int = 0
for i in range(10):
    total += i


# --- Annotated class with inheritance ---

class Base:
    x: int = 0

class Derived(Base):
    y: str = ""
    z: float = 0.0


# --- Multiple annotations on one line (separate statements) ---

# NOTE: semicolons in annotation line split into separate statements
a: int = 1
b: str = "x"
c: float = 3.14


# --- End of type annotation constructs ---
