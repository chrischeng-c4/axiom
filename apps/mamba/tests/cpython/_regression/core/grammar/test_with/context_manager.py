# RUN: parse
# CPython 3.12 test_with: context manager statements

# Basic with statement
class Resource:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        return False

with Resource() as r:
    pass

# Without as clause
with Resource():
    pass

# Nested with
with Resource() as a:
    with Resource() as b:
        pass

# Multiple context managers (single line)
with Resource() as x, Resource() as y:
    pass

