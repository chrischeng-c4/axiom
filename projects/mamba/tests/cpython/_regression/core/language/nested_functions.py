# Nested function definitions with basic capture

# Inner function calling outer definition
def outer1():
    def inner():
        return "inner"
    return "outer-" + inner()

print(outer1())

# Multiple nested levels, all local
def level1():
    def level2():
        def level3():
            return "deep"
        return "mid-" + level3()
    return "top-" + level2()

print(level1())

# Sibling inner functions
def siblings():
    def a():
        return "A"
    def b():
        return "B"
    return a() + b()

print(siblings())

# Nested function returning constant
def fixed_value():
    def five():
        return 5
    return five() + five() + five()

print(fixed_value())

# Nested function called multiple times
def repeat_inner():
    def greet():
        return "hi"
    r = []
    r.append(greet())
    r.append(greet())
    r.append(greet())
    return r

print(repeat_inner())
