"""Surface contract for third-party marshmallow package.

# type-regime: monomorphic

Probes: marshmallow.Schema, marshmallow.fields, marshmallow.validate,
marshmallow.ValidationError, marshmallow.EXCLUDE/INCLUDE/RAISE.
CPython 3.12 is the oracle.
"""

import marshmallow  # type: ignore[import]
import marshmallow.fields as fields  # type: ignore[import]
import marshmallow.validate as validate  # type: ignore[import]

# Core API
assert hasattr(marshmallow, "Schema"), "Schema"
assert hasattr(marshmallow, "fields"), "fields"
assert hasattr(marshmallow, "validate"), "validate"
assert hasattr(marshmallow, "ValidationError"), "ValidationError"
assert hasattr(marshmallow, "EXCLUDE"), "EXCLUDE"
assert hasattr(marshmallow, "INCLUDE"), "INCLUDE"
assert hasattr(marshmallow, "RAISE"), "RAISE"
assert hasattr(marshmallow, "pre_load"), "pre_load"
assert hasattr(marshmallow, "post_load"), "post_load"
assert hasattr(marshmallow, "pre_dump"), "pre_dump"
assert hasattr(marshmallow, "post_dump"), "post_dump"

# Schema is callable
assert callable(marshmallow.Schema), "Schema callable"

# ValidationError is an exception
assert issubclass(marshmallow.ValidationError, Exception), \
    "ValidationError < Exception"

# fields module
assert hasattr(fields, "String"), "fields.String"
assert hasattr(fields, "Integer"), "fields.Integer"
assert hasattr(fields, "Float"), "fields.Float"
assert hasattr(fields, "Boolean"), "fields.Boolean"
assert hasattr(fields, "List"), "fields.List"
assert hasattr(fields, "Dict"), "fields.Dict"
assert hasattr(fields, "Email"), "fields.Email"
assert hasattr(fields, "Nested"), "fields.Nested"
assert hasattr(fields, "DateTime"), "fields.DateTime"

# validate module
assert hasattr(validate, "Length"), "validate.Length"
assert hasattr(validate, "Range"), "validate.Range"
assert hasattr(validate, "OneOf"), "validate.OneOf"
assert hasattr(validate, "Regexp"), "validate.Regexp"

# Schema with declared fields
class _PersonSchema(marshmallow.Schema):
    name = fields.String(required=True)
    age = fields.Integer()

_s = _PersonSchema()
assert hasattr(_s, "load"), "schema.load"
assert hasattr(_s, "dump"), "schema.dump"
assert hasattr(_s, "validate"), "schema.validate"
assert hasattr(_s, "fields"), "schema.fields"

# Module attributes stable
_schema_ref = marshmallow.Schema
assert marshmallow.Schema is _schema_ref, "Schema stable"
_fields_ref = marshmallow.fields
assert marshmallow.fields is _fields_ref, "fields stable"
_validate_ref = marshmallow.validate
assert marshmallow.validate is _validate_ref, "validate stable"
_ve_ref = marshmallow.ValidationError
assert marshmallow.ValidationError is _ve_ref, "ValidationError stable"

print("surface OK")
