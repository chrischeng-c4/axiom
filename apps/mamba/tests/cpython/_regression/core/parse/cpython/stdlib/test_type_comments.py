# RUN: parse
# Legacy type comment syntax — these are just comments, so they always parse.


# --- Variable type comments ---

x = []  # type: List[int]
y = {}  # type: Dict[str, int]
z = ()  # type: Tuple[int, ...]
name = ""  # type: str
count = 0  # type: int
flag = True  # type: bool
value = None  # type: Optional[int]


# --- Function type comments ---

def add(x, y):  # type: (int, int) -> int
    return x + y

def greet(name):  # type: (str) -> str
    return "hello " + name

def noop():  # type: () -> None
    pass

def process(items):  # type: (List[int]) -> List[str]
    return [str(i) for i in items]


# --- Complex type comments ---

matrix = []  # type: List[List[int]]
lookup = {}  # type: Dict[str, List[int]]
callback = None  # type: Optional[Callable[[int], str]]


# --- Type comments with Union ---

result = None  # type: Union[int, str, None]
data = []  # type: List[Union[int, str]]


# --- Type comments on assignments ---

a, b = 1, 2  # type: int, int
x, y, z = 1, 2, 3  # type: int, int, int


# --- Type ignore comments ---

x = "not an int"  # type: ignore
y = []  # type: ignore[assignment]
z = {}  # type: ignore[arg-type]


# --- End of type comment constructs ---
