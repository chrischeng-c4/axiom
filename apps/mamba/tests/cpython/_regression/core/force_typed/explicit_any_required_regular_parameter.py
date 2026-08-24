from typing import Any


def echo(value: Any):
    print("BODY", value)
    return value


print("RETURN", echo("dynamic"))
