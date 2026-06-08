# RUN: parse
# CPython 3.12 test_grammar: global and nonlocal statements

# Global declaration
x = 0

def modify_global():
    global x
    x = 42

# Multiple globals
def multi_global():
    global x, y, z
    x = 1
    y = 2
    z = 3

# Nonlocal declaration
def outer():
    count = 0
    def increment():
        nonlocal count
        count += 1
    increment()
    return count

# Nested nonlocal
def level1():
    a = 1
    def level2():
        b = 2
        def level3():
            nonlocal a, b
            a = 10
            b = 20
        level3()
    level2()

# Nonlocal with closure
def make_counter():
    n = 0
    def counter():
        nonlocal n
        n += 1
        return n
    return counter
