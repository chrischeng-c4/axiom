# Context manager edge cases

# __exit__ returns True suppresses exception
class Suppress:
    def __enter__(self):
        return self
    def __exit__(self, exc_type, exc_val, exc_tb):
        return True

with Suppress():
    raise ValueError('suppressed')
print('after suppression')

# Multiple context managers — LIFO exit order
class CM:
    def __init__(self, n):
        self.n = n
    def __enter__(self):
        print(f'enter {self.n}')
        return self
    def __exit__(self, *a):
        print(f'exit {self.n}')
        return False

with CM(1), CM(2):
    print('body')

# Nested with blocks
with CM(3):
    with CM(4):
        print('nested body')
