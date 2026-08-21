"""Define a pydantic model and exercise both the success and error paths.

End-user scenario: a downstream API service defines a `BaseModel` with
typed fields and validates inbound payloads through `model_validate`.
The fixture proves (1) a well-formed payload parses to the expected
typed values and (2) a malformed payload raises `ValidationError` —
both legs are required because a pydantic that silently dropped
validation would still pass a success-only check.

DoD: this script must exit 0 under both CPython and mamba.
"""

from pydantic import BaseModel, ValidationError


class UserCreate(BaseModel):
    username: str
    age: int


# Success path — strings/ints come through unchanged.
user = UserCreate.model_validate({"username": "ada", "age": 36})
assert user.username == "ada", f"unexpected username: {user.username!r}"
assert user.age == 36, f"unexpected age: {user.age!r}"
assert isinstance(user.age, int), f"age coerced to non-int: {type(user.age).__name__}"

# Error path — an obviously wrong type must raise ValidationError, not
# silently coerce or no-op. Catching the exception class (rather than
# bare Exception) ensures pydantic's error machinery actually fired.
try:
    UserCreate.model_validate({"username": "ada", "age": "not-an-int"})
except ValidationError as err:
    rendered = str(err)
    assert "age" in rendered, f"error message missing field name: {rendered!r}"
else:
    raise AssertionError("ValidationError not raised for malformed payload")

print("ok:", user.username, user.age)
