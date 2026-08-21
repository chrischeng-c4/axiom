# RUN: parse
# Extracted from CPython Lib/test/test_dictcomps.py — dict and set comprehension syntax constructs only.


# --- Basic dict comprehension ---

{k: v for k, v in [("a", 1), ("b", 2), ("c", 3)]}
{x: x for x in range(5)}
{x: x * x for x in range(10)}
{k: v for k, v in {"a": 1, "b": 2}.items()}
{k: None for k in "abc"}
{i: c for i, c in enumerate("hello")}


# --- Dict comprehension with condition ---

{k: v for k, v in [("a", 1), ("b", 2), ("c", 3)] if v > 1}
{x: x * x for x in range(20) if x % 2 == 0}
{k: v for k, v in enumerate("hello") if v != "l"}
{x: x for x in range(100) if x % 3 == 0 and x % 5 == 0}
{k: v for k, v in {"a": 1, "b": 2, "c": 3}.items() if k != "b"}
{x: x for x in range(50) if x > 10 if x < 30}


# --- Nested dict comprehensions ---

{k: {x: x * x for x in range(v)} for k, v in [("a", 3), ("b", 2)]}
{i: {j: i * j for j in range(3)} for i in range(3)}

# Flattening with dict comp
{f"{k1}_{k2}": v
 for k1, inner in [("a", {"x": 1}), ("b", {"y": 2})]
 for k2, v in inner.items()}


# --- Complex key expressions ---

{str(x): x for x in range(5)}
{(x, y): x + y for x in range(3) for y in range(3)}
{frozenset([x]): x for x in range(5)}
{x + y: x * y for x in range(3) for y in range(3)}
{f"key_{i}": i for i in range(5)}
{chr(65 + i): i for i in range(26)}


# --- Complex value expressions ---

{x: [i for i in range(x)] for x in range(5)}
{x: (x, x * 2, x * 3) for x in range(5)}
{x: {"square": x * x, "cube": x ** 3} for x in range(5)}
{x: x if x > 0 else -x for x in range(-3, 4)}
{x: abs(x) for x in range(-5, 6)}
{k: v.upper() for k, v in [("a", "hello"), ("b", "world")]}
{k: len(v) for k, v in {"a": [1, 2], "b": [3, 4, 5]}.items()}


# --- Dict comprehension with enumerate ---

{i: v for i, v in enumerate(["a", "b", "c"])}
{v: i for i, v in enumerate(["a", "b", "c"])}
{i: v.upper() for i, v in enumerate("hello")}
{i * 10: v for i, v in enumerate(range(5))}


# --- Dict comprehension with zip ---

{k: v for k, v in zip("abc", range(3))}
{k: v for k, v in zip(range(5), range(5, 10))}
{k: v for k, v in zip(["name", "age"], ["Alice", 30])}


# --- Dict comprehension with walrus operator ---

{k: doubled for k in range(10) if (doubled := k * 2) > 10}
{s: n for s in ["abc", "de", "f", "ghij"] if (n := len(s)) > 1}


# --- Dict comprehension with unpacking ---

pairs = [("a", 1), ("b", 2), ("c", 3)]
{k: v for k, v in pairs}
# NOTE: parenthesized for-loop target in dict comp not supported
# {k: v for (k, v) in pairs}
{k: v for k, v in pairs}

triples = [("a", 1, "x"), ("b", 2, "y")]
{k: (v, extra) for k, v, extra in triples}


# --- Dict comprehension in various contexts ---

len({x: x for x in range(10)})
list({x: x for x in range(5)}.keys())
list({x: x for x in range(5)}.values())
list({x: x * x for x in range(5)}.items())

result = {x: x for x in range(5)}
# NOTE: ** dict unpacking in dict literal not supported
# merged = {**{x: x for x in range(3)}, **{x: x * 10 for x in range(3, 6)}}


# --- Empty dict comprehension ---

{k: v for k, v in []}
{x: x for x in range(0)}


# --- Dict comprehension with method calls ---

words = ["Hello", "World", "Python"]
{w.lower(): w.upper() for w in words}
{w: w[::-1] for w in words}
{w[0]: w for w in words}
{w: len(w) for w in words}
{i: w.split() for i, w in enumerate(["a b", "c d"])}


# --- Dict comprehension scope ---

x = "outer"
result = {x: x for x in range(3)}
x  # still "outer"


# ===== Set comprehensions =====


# --- Basic set comprehension ---

{x for x in range(10)}
{x for x in [1, 2, 3, 4, 5]}
{x for x in "hello"}
{x for x in (1, 2, 2, 3, 3, 3)}


# --- Set comprehension with condition ---

{x for x in range(20) if x % 2 == 0}
{x for x in range(100) if x % 3 == 0}
{x for x in [-3, -2, -1, 0, 1, 2, 3] if x > 0}
{c for c in "hello world" if c != " "}
{x for x in range(50) if x > 10 if x < 30}
{x for x in range(100) if x % 2 == 0 if x % 5 == 0}


# --- Set comprehension with function application ---

{abs(x) for x in [-3, -2, -1, 0, 1, 2, 3]}
{len(s) for s in ["hello", "world", "hi", "hey"]}
{s.upper() for s in ["hello", "world"]}
{type(x) for x in [1, "a", 2.0, None]}
{x * x for x in range(10)}
{chr(x) for x in range(65, 91)}


# --- Nested set comprehension ---

{x for xs in [[1, 2], [2, 3], [3, 4]] for x in xs}
{(x, y) for x in range(3) for y in range(3)}
{x + y for x in range(3) for y in range(3)}
{frozenset([x, y]) for x in range(3) for y in range(x + 1, 3)}


# --- Set comprehension with walrus ---

{y for x in range(10) if (y := x * x) > 20}
{n for s in ["abc", "de", "f"] if (n := len(s)) > 1}


# --- Set comprehension with tuple unpacking ---

{a + b for a, b in [(1, 2), (3, 4), (5, 6)]}
{k for k, v in {"a": 1, "b": 2, "c": 3}.items()}
{v for k, v in {"a": 1, "b": 2, "c": 3}.items() if v > 1}


# --- Set comprehension with complex expressions ---

{x if x > 0 else -x for x in [-3, -1, 0, 2, 4]}
{(x, x * x) for x in range(5)}
{frozenset([x]) for x in range(5)}


# --- Set comprehension in various contexts ---

len({x for x in range(10)})
sorted({x for x in [3, 1, 4, 1, 5]})
list({x for x in range(5)})
frozenset({x for x in range(5)})
bool({x for x in range(0)})

result = {x * 2 for x in range(5)}
combined = {x for x in range(5)} | {x for x in range(3, 8)}
intersected = {x for x in range(10)} & {x for x in range(5, 15)}


# --- Empty set comprehension ---

{x for x in []}
{x for x in range(0)}
{x for x in range(10) if False}


# --- Set comprehension scope ---

x = "outer"
result = {x for x in range(5)}
x  # still "outer"
