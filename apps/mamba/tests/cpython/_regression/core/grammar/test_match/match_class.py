# RUN: parse
# CPython 3.12 test_match: class patterns

class Point:
    x: int
    y: int
    def __init__(self, x, y):
        self.x = x
        self.y = y

# Class pattern
match Point(1, 2):
    case Point(x=0, y=0):
        pass
    case Point(x=x, y=0):
        pass
    case Point(x=0, y=y):
        pass
    case Point(x=x, y=y):
        pass

# Built-in type patterns
match 42:
    case int(x):
        pass

match "hello":
    case str(s):
        pass

match [1, 2]:
    case list(items):
        pass

# Nested class patterns
class Line:
    def __init__(self, start, end):
        self.start = start
        self.end = end

match Line(Point(0, 0), Point(1, 1)):
    case Line(start=Point(x=0, y=0), end=end):
        pass
