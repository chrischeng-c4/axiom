# with...as: basic context manager protocol

class CM:
    def __init__(self, value):
        self.value = value

    def __enter__(self):
        print(f"enter {self.value}")
        return self.value

    def __exit__(self, exc_type, exc_val, exc_tb):
        print(f"exit {self.value}")
        return False

# Basic with...as
with CM(42) as x:
    print(f"body: {x}")

# Nested with (separate statements)
with CM("outer") as a:
    with CM("inner") as b:
        print(f"a={a}, b={b}")

# Without as
with CM("no-as"):
    print("no binding")
