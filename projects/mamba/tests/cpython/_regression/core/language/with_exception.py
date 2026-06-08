# with statement: __exit__ called even on exception

class SafeResource:
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        print(f"open {self.name}")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print(f"close {self.name}")
        return False  # don't suppress exception

# Normal exit
with SafeResource("file1") as r:
    print(f"using {r.name}")

# Exception exit — __exit__ still called
try:
    with SafeResource("file2") as r:
        print(f"using {r.name}")
        raise ValueError("error in body")
except ValueError as e:
    print(f"caught: {e}")
