# RUN: parse
# CPython-derived: async/await, yield, generators

# --- async function ---
async def fetch(url: str) -> str:
    return url

# --- async function with await ---
async def get_data(url: str) -> str:
    result: str = await fetch(url)
    return result

# --- async for ---
async def collect(source: int) -> int:
    async for item in source:
        pass
    return 0

# --- async with ---
async def managed(path: str) -> int:
    async with open(path) as f:
        pass
    return 0

# --- decorated async function ---
@decorator
async def decorated_async() -> int:
    return 0

# --- yield ---
def gen() -> int:
    yield 1
    yield 2
    yield 3
    return 0

# --- yield expression ---
def gen_expr() -> int:
    x = yield 42
    return 0

# --- yield from ---
def delegating() -> int:
    yield from other_gen()
    return 0

# --- bare yield ---
def bare() -> int:
    yield
    return 0
