# Class used as decorator via __init__ + __call__

class CountCalls:
    def __init__(self, func):
        self.func = func
        self.count = 0

    def __call__(self, *args, **kwargs):
        self.count = self.count + 1
        print(f"call #{self.count}")
        return self.func(*args, **kwargs)

@CountCalls
def say_hello(name):
    print(f"Hello, {name}!")

say_hello("Alice")
say_hello("Bob")
say_hello("Charlie")
