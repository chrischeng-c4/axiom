# RUN: parse
# Python 3.12 type alias syntax (PEP 695) — type statement constructs only.


# --- Basic type aliases ---

type IntAlias = int
type StrAlias = str
type FloatAlias = float
type BoolAlias = bool
type BytesAlias = bytes
type NoneAlias = None


# --- Container type aliases ---

type IntList = list[int]
type StrList = list[str]
type IntSet = set[int]
type IntFrozenSet = frozenset[int]
# NOTE: Ellipsis in type annotation not supported: type IntTuple = tuple[int, ...]
type FixedTuple = tuple[int, str, float]
type IntDict = dict[str, int]


# --- Compound type aliases ---

type Point = tuple[int, int]
type Point3D = tuple[int, int, int]
type Pair = tuple[int, str]
type Matrix = list[list[int]]
type Grid = list[list[list[float]]]
type Lookup = dict[str, list[int]]
type NestedDict = dict[str, dict[str, int]]


# --- Union type aliases (PEP 604) ---

type MaybeInt = int | None
type MaybeStr = str | None
type IntOrStr = int | str
type Number = int | float | complex
type MaybeNumber = int | float | None


# --- Generic type aliases ---

type Vector[T] = list[T]
type Mapping[K, V] = dict[K, V]
type Pair2[T, U] = tuple[T, U]
type Container[T] = list[T] | set[T]
type Optional[T] = T | None


# --- Generic with multiple parameters ---

type BiMap[K, V] = dict[K, V]
type Triple[A, B, C] = tuple[A, B, C]
type Transformer[In, Out] = dict[In, Out]


# --- Nested generic type aliases ---

type ListOfLists[T] = list[list[T]]
type DictOfLists[K, V] = dict[K, list[V]]
type MatrixT[T] = list[list[T]]
type NestedOpt[T] = list[T | None]


# --- Bounded type parameters ---

# NOTE: bounded type param (T: constraint) not supported: type IntLike[T: int] = list[T]
# NOTE: bounded type param (T: constraint) not supported: type Stringable[T: str] = set[T]


# --- Constrained type parameters ---

# NOTE: bounded type param (T: constraint) not supported: type NumericList[T: (int, float)] = list[T]
# NOTE: bounded type param (T: constraint) not supported: type TextLike[T: (str, bytes)] = list[T]


# --- Complex nested types ---

type Callback = tuple[str, list[int]]
type Handler = dict[str, tuple[int, str]]
type Registry = dict[str, list[tuple[int, str]]]


# --- Type alias with callable ---

# NOTE: type[X] in type alias RHS conflicts with soft keyword
# type SimpleFunc = type[int]
# type Factory[T] = type[T]


# --- Type aliases with string forward references ---

# NOTE: string literal type annotations in type alias not supported
# type TreeNode = tuple[int, "TreeNode", "TreeNode"] | None
# NOTE: string literal type annotations in type alias not supported
# type LinkedList = tuple[int, "LinkedList"] | None


# --- Type alias in class scope ---

class MyClass:
    type InnerType = int
    type InnerList[T] = list[T]


# --- Type alias in function scope ---

def my_function():
    type LocalType = str
    type LocalList[T] = list[T]
    x: LocalType = "hello"


# --- Type alias in conditional ---

flag = True
if flag:
    type CurrentType = int
else:
    type CurrentType = str


# --- Type alias with complex union ---

type JSONPrimitive = str | int | float | bool | None
# NOTE: string literal type annotations in type alias not supported
# type JSONArray = list["JSONValue"]
# NOTE: string literal type annotations in type alias not supported
# type JSONObject = dict[str, "JSONValue"]
type JSONValue = JSONPrimitive | JSONArray | JSONObject


# --- Multiple type aliases in sequence ---

type X = int
type Y = str
type Z = float
type XY = tuple[X, Y]
type XYZ = tuple[X, Y, Z]


# --- End of type alias constructs ---
