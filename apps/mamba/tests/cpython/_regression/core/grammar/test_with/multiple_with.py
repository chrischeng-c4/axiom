# RUN: parse
# CPython 3.12 test_with: parenthesized multiple context managers (PEP 617)

class Resource:
    def __enter__(self):
        return self
    def __exit__(self, *args):
        return False

# Parenthesized with (PEP 617, Python 3.10+)
with (Resource() as a, Resource() as b):
    pass

# Parenthesized with multi-line
with (
    Resource() as a,
    Resource() as b,
    Resource() as c,
):
    pass
