# RUN: parse
# CPython-derived: type annotations and type alias (PEP 695)

# --- basic type annotations ---
x: int = 0
y: float = 0.0
z: str = ""
w: bool = True

# --- optional type (Mamba shorthand) ---
a: int? = None

# --- generic types ---
items: list[int] = []
mapping: dict[str, int] = {}
pair: tuple[int, str] = (1, "a")

# --- union type (PEP 604 syntax) ---
value: int | str = 0
result: int | float | None = None

# --- nested generic ---
nested: list[list[int]] = []
complex_map: dict[str, list[int]] = {}

# --- user-defined generic type ---
container: MyType[int] = None

# --- function type annotation ---
callback: (int, str) -> bool = None

# --- type alias (PEP 695) ---
type Num = int
type Point = tuple[int, int]
type Result = int | str
type Mapping = dict[str, int]

# --- type alias with type params ---
type Pair[T] = tuple[T, T]
type Container[T, U] = dict[T, list[U]]

# --- function with generic type params (PEP 695) ---
def first[T](items: list[int]) -> int:
    return 0

def pair[T, U](a: int, b: int) -> int:
    return 0

# --- class with type params ---
class Box[T]:
    value: int = 0

class Pair[K, V]:
    key: int = 0
    value: int = 0

# --- None as type ---
def returns_none(x: int) -> None:
    pass
