# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `__import__(name, globals=None, locals=None, fromlist=(), level=0)`
# (#1256 long-tail tracker, sub-priority 2). Public hook into the
# import machinery. Mamba routes through the existing
# `mb_import(name)` runtime; trailing args are silently dropped at
# the lower-pass level — package-context threading is a separate
# follow-up.

# 1-arg form: returns the imported module namespace.
math = __import__("math")
print(math.pi)                  # 3.141592653589793
print(math.sqrt(16))            # 4.0
print(math.floor(3.7))          # 3

# Trailing CPython-shape args are accepted and ignored.
m2 = __import__("math", None, None, (), 0)
print(m2.pi)                    # 3.141592653589793
print(m2.e)                     # 2.718281828459045

# Lookup of a non-existent module raises ImportError under CPython;
# Mamba's import path returns None — that divergence is tracked
# separately, so the fixture only covers the success path.

# json: stdlib already wired through mb_import.
js = __import__("json")
print(js.dumps([1, 2, 3]))      # [1, 2, 3]
