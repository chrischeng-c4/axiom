# RUN: parse
# CPython 3.12 test_decorators: function decorators

def log(func):
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

def repeat(n):
    def decorator(func):
        def wrapper(*args, **kwargs):
            for _ in range(n):
                func(*args, **kwargs)
        return wrapper
    return decorator

# Simple decorator
@log
def say_hello():
    pass

# Parameterized decorator
@repeat(3)
def greet():
    pass

# Stacked decorators
@log
@repeat(2)
def multi():
    pass

# Lambda as decorator target
identity = lambda f: f

@identity
def plain():
    pass
