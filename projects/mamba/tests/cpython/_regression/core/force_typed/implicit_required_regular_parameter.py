from typing import Any


def echo(value):
    print("BODY", value)
    return value


print("RETURN", echo("dynamic"))
