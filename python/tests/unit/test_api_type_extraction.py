"""Tests for API type extraction system."""

import pytest
from typing import Annotated, List, Dict, Optional, Union, Literal, Tuple, Set
from dataclasses import dataclass
from datetime import datetime, date, time, timedelta
from uuid import UUID
from decimal import Decimal
from enum import Enum

from cclab.quasar.type_extraction import (
    extract_handler_meta,
    extract_type_schema,
    schema_to_rust_type_descriptor,
    extract_dataclass_schema,
)
from cclab.quasar.types import Path, Query, Body, Header, Depends


class TestBasicTypeExtraction:
    """Test basic type extraction."""

    def test_string_type(self):
        schema = extract_type_schema(str)
        assert schema == {"type": "string"}

    def test_int_type(self):
        schema = extract_type_schema(int)
        assert schema == {"type": "int"}

    def test_float_type(self):
        schema = extract_type_schema(float)
        assert schema == {"type": "float"}

    def test_bool_type(self):
        schema = extract_type_schema(bool)
        assert schema == {"type": "bool"}

    def test_bytes_type(self):
        schema = extract_type_schema(bytes)
        assert schema == {"type": "bytes"}

    def test_none_type(self):
        schema = extract_type_schema(None)
        assert schema == {"type": "null"}

    def test_none_type_class(self):
        schema = extract_type_schema(type(None))
        assert schema == {"type": "null"}


class TestSpecialTypeExtraction:
    """Test special type extraction."""

    def test_uuid_type(self):
        schema = extract_type_schema(UUID)
        assert schema == {"type": "uuid"}

    def test_datetime_type(self):
        schema = extract_type_schema(datetime)
        assert schema == {"type": "datetime"}

    def test_date_type(self):
        schema = extract_type_schema(date)
        assert schema == {"type": "date"}

    def test_time_type(self):
        schema = extract_type_schema(time)
        assert schema == {"type": "time"}

    def test_timedelta_type(self):
        schema = extract_type_schema(timedelta)
        assert schema == {"type": "timedelta"}

    def test_decimal_type(self):
        schema = extract_type_schema(Decimal)
        assert schema == {"type": "decimal"}


class TestComplexTypeExtraction:
    """Test complex type extraction."""

    def test_list_type(self):
        schema = extract_type_schema(List[str])
        assert schema == {"type": "list", "items": {"type": "string"}}

    def test_nested_list(self):
        schema = extract_type_schema(List[List[int]])
        assert schema == {
            "type": "list",
            "items": {"type": "list", "items": {"type": "int"}}
        }

    def test_dict_type(self):
        schema = extract_type_schema(Dict[str, int])
        assert schema["type"] == "object"
        assert schema["additional_properties"] == {"type": "int"}

    def test_dict_complex_value(self):
        schema = extract_type_schema(Dict[str, List[int]])
        assert schema["type"] == "object"
        assert schema["additional_properties"] == {
            "type": "list",
            "items": {"type": "int"}
        }

    def test_optional_type(self):
        schema = extract_type_schema(Optional[str])
        assert schema == {"type": "optional", "inner": {"type": "string"}}

    def test_optional_complex(self):
        schema = extract_type_schema(Optional[List[int]])
        assert schema["type"] == "optional"
        assert schema["inner"] == {"type": "list", "items": {"type": "int"}}

    def test_union_type(self):
        schema = extract_type_schema(Union[str, int])
        assert schema["type"] == "union"
        assert len(schema["variants"]) == 2
        assert {"type": "string"} in schema["variants"]
        assert {"type": "int"} in schema["variants"]
        assert schema["nullable"] is False

    def test_union_with_none(self):
        schema = extract_type_schema(Union[str, int, None])
        assert schema["type"] == "union"
        assert len(schema["variants"]) == 2
        assert schema["nullable"] is True

    def test_tuple_variable_length(self):
        schema = extract_type_schema(Tuple[int, ...])
        assert schema == {"type": "list", "items": {"type": "int"}}

    def test_tuple_fixed_length(self):
        schema = extract_type_schema(Tuple[str, int, bool])
        assert schema["type"] == "tuple"
        assert len(schema["items"]) == 3
        assert schema["items"][0] == {"type": "string"}
        assert schema["items"][1] == {"type": "int"}
        assert schema["items"][2] == {"type": "bool"}

    def test_set_type(self):
        schema = extract_type_schema(Set[str])
        assert schema == {"type": "set", "items": {"type": "string"}}

    def test_frozenset_type(self):
        schema = extract_type_schema(frozenset)
        assert schema["type"] == "set"

    def test_literal_type(self):
        schema = extract_type_schema(Literal["admin", "user", "guest"])
        assert schema["type"] == "literal"
        assert set(schema["values"]) == {"admin", "user", "guest"}


class TestEnumExtraction:
    """Test enum type extraction."""

    def test_enum_type(self):
        class Status(Enum):
            ACTIVE = "active"
            INACTIVE = "inactive"
            PENDING = "pending"

        schema = extract_type_schema(Status)
        assert schema["type"] == "enum"
        assert set(schema["values"]) == {"active", "inactive", "pending"}
        assert set(schema["names"]) == {"ACTIVE", "INACTIVE", "PENDING"}


class TestDataclassExtraction:
    """Test dataclass schema extraction."""

    def test_simple_dataclass(self):
        @dataclass
        class User:
            name: str
            age: int

        schema = extract_type_schema(User)
        assert schema["type"] == "object"
        assert schema["class_name"] == "User"
        assert len(schema["fields"]) == 2

        name_field = next(f for f in schema["fields"] if f["name"] == "name")
        assert name_field["type"] == {"type": "string"}
        assert name_field["required"] is True

        age_field = next(f for f in schema["fields"] if f["name"] == "age")
        assert age_field["type"] == {"type": "int"}
        assert age_field["required"] is True

    def test_dataclass_with_defaults(self):
        @dataclass
        class Config:
            host: str = "localhost"
            port: int = 8080

        schema = extract_type_schema(Config)
        assert schema["type"] == "object"

        for field in schema["fields"]:
            assert field["required"] is False
            assert "default" in field

    def test_dataclass_with_optional(self):
        @dataclass
        class UserProfile:
            username: str
            bio: Optional[str] = None

        schema = extract_type_schema(UserProfile)
        bio_field = next(f for f in schema["fields"] if f["name"] == "bio")
        assert bio_field["type"]["type"] == "optional"
        assert bio_field["type"]["inner"] == {"type": "string"}

    def test_nested_dataclass(self):
        @dataclass
        class Address:
            city: str
            country: str

        @dataclass
        class User:
            name: str
            address: Address

        schema = extract_type_schema(User)
        address_field = next(f for f in schema["fields"] if f["name"] == "address")
        assert address_field["type"]["type"] == "object"
        assert address_field["type"]["class_name"] == "Address"
        assert len(address_field["type"]["fields"]) == 2


class TestHandlerMetaExtraction:
    """Test handler metadata extraction."""

    def test_path_param(self):
        async def handler(user_id: Annotated[str, Path()]) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/users/{user_id}")
        assert len(meta["validator"]["path_params"]) == 1
        assert meta["validator"]["path_params"][0]["name"] == "user_id"
        assert meta["validator"]["path_params"][0]["location"] == "path"

    def test_query_param(self):
        async def handler(limit: Annotated[int, Query(default=10)]) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/items")
        assert len(meta["validator"]["query_params"]) == 1
        param = meta["validator"]["query_params"][0]
        assert param["name"] == "limit"
        assert param["location"] == "query"
        assert param["default"] == 10
        assert param["required"] is False

    def test_query_param_with_constraints(self):
        async def handler(
            limit: Annotated[int, Query(default=10, ge=1, le=100)]
        ) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/items")
        param = meta["validator"]["query_params"][0]
        assert param["name"] == "limit"
        assert param["default"] == 10
        assert param["type"]["minimum"] == 1
        assert param["type"]["maximum"] == 100

    def test_body_param(self):
        @dataclass
        class CreateUser:
            name: str
            email: str

        async def handler(user: Annotated[CreateUser, Body()]) -> dict:
            pass

        meta = extract_handler_meta(handler, "POST", "/users")
        body = meta["validator"]["body"]
        assert body["name"] == "user"
        assert body["location"] == "body"
        assert body["type"]["type"] == "object"
        assert body["type"]["class_name"] == "CreateUser"

    def test_header_param(self):
        async def handler(
            x_request_id: Annotated[str, Header()]
        ) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/test")
        assert len(meta["validator"]["header_params"]) == 1
        header = meta["validator"]["header_params"][0]
        assert header["name"] == "x_request_id"
        assert header["location"] == "header"
        assert "alias" in header

    def test_multiple_params(self):
        async def handler(
            user_id: Annotated[str, Path()],
            limit: Annotated[int, Query(default=10)],
            x_token: Annotated[str, Header()],
        ) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/users/{user_id}")
        assert len(meta["validator"]["path_params"]) == 1
        assert len(meta["validator"]["query_params"]) == 1
        assert len(meta["validator"]["header_params"]) == 1

    def test_dependency_param(self):
        async def get_db():
            return "database"

        async def handler(
            db: Annotated[str, Depends(get_db)]
        ) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/test")
        assert len(meta["dependencies"]) == 1
        dep = meta["dependencies"][0]
        assert dep["name"] == "db"
        assert callable(dep["dependency"])
        assert dep["use_cache"] is True

    def test_response_schema(self):
        @dataclass
        class UserResponse:
            id: int
            name: str

        async def handler() -> UserResponse:
            pass

        meta = extract_handler_meta(handler, "GET", "/user")
        assert meta["response_schema"]["type"] == "object"
        assert meta["response_schema"]["class_name"] == "UserResponse"

    def test_skip_special_params(self):
        async def handler(
            self,
            request,
            response,
            user_id: Annotated[str, Path()]
        ) -> dict:
            pass

        meta = extract_handler_meta(handler, "GET", "/users/{user_id}")
        # Only user_id should be extracted
        assert len(meta["validator"]["path_params"]) == 1


class TestRustTypeDescriptor:
    """Test conversion to Rust TypeDescriptor format."""

    def test_string_basic(self):
        schema = {"type": "string"}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust == {"type": "string"}

    def test_string_with_constraints(self):
        schema = {"type": "string", "min_length": 1, "max_length": 100, "pattern": "^[a-z]+$"}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "string"
        assert rust["min_length"] == 1
        assert rust["max_length"] == 100
        assert rust["pattern"] == "^[a-z]+$"

    def test_int_with_constraints(self):
        schema = {"type": "int", "minimum": 0, "maximum": 100}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "int"
        assert rust["minimum"] == 0
        assert rust["maximum"] == 100

    def test_float_with_constraints(self):
        schema = {"type": "float", "minimum": 0.0, "maximum": 1.0}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "float"
        assert rust["minimum"] == 0.0
        assert rust["maximum"] == 1.0

    def test_list_type(self):
        schema = {"type": "list", "items": {"type": "string"}}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "list"
        assert rust["items"] == {"type": "string"}

    def test_nested_object(self):
        schema = {
            "type": "object",
            "fields": [
                {"name": "id", "type": {"type": "int"}, "required": True},
                {"name": "name", "type": {"type": "string"}, "required": True},
            ]
        }
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "object"
        assert len(rust["fields"]) == 2
        assert rust["fields"][0]["name"] == "id"
        assert rust["fields"][0]["required"] is True

    def test_optional_type(self):
        schema = {"type": "optional", "inner": {"type": "string"}}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "optional"
        assert rust["inner"] == {"type": "string"}

    def test_special_types(self):
        for type_name in ["uuid", "email", "url", "datetime", "date", "time"]:
            schema = {"type": type_name}
            rust = schema_to_rust_type_descriptor(schema)
            assert rust == {"type": type_name}

    def test_email_format(self):
        schema = {"type": "string", "format": "email"}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "email"

    def test_uri_format(self):
        schema = {"type": "string", "format": "uri"}
        rust = schema_to_rust_type_descriptor(schema)
        assert rust["type"] == "url"


class TestAnnotatedTypes:
    """Test handling of Annotated types."""

    def test_annotated_strips_annotation(self):
        schema = extract_type_schema(Annotated[str, "some metadata"])
        assert schema == {"type": "string"}

    def test_annotated_with_constraint(self):
        schema = extract_type_schema(Annotated[int, Query(ge=0, le=100)])
        # Annotations are stripped in extract_type_schema
        # Constraints are added in extract_param_info
        assert schema == {"type": "int"}

    def test_nested_annotated(self):
        schema = extract_type_schema(Optional[Annotated[str, "metadata"]])
        assert schema == {"type": "optional", "inner": {"type": "string"}}


class TestEdgeCases:
    """Test edge cases and error handling."""

    def test_untyped_list(self):
        schema = extract_type_schema(list)
        # Should return list with any items
        assert schema["type"] == "list"

    def test_untyped_dict(self):
        schema = extract_type_schema(dict)
        assert schema["type"] == "object"

    def test_any_type(self):
        from typing import Any
        schema = extract_type_schema(Any)
        assert schema == {"type": "any"}

    def test_unknown_type(self):
        class CustomType:
            pass

        schema = extract_type_schema(CustomType)
        assert schema["type"] == "any"
        assert schema["python_type"] == "CustomType"

    def test_handler_without_annotations(self):
        async def handler(user_id):
            pass

        meta = extract_handler_meta(handler, "GET", "/users/{user_id}")
        # Should still extract path param
        assert len(meta["validator"]["path_params"]) == 1

    def test_handler_with_exception_in_hints(self):
        # Test graceful handling when get_type_hints fails
        async def handler(x: "NonExistentType") -> dict:
            pass

        # Should not raise exception
        meta = extract_handler_meta(handler, "GET", "/test")
        assert "validator" in meta


class TestPydanticIntegration:
    """Test Pydantic model extraction (if available)."""

    def test_pydantic_unavailable_fallback(self):
        # When Pydantic is not available, should handle gracefully
        from cclab.quasar.type_extraction import HAS_PYDANTIC

        if not HAS_PYDANTIC:
            # Test that non-Pydantic code works
            @dataclass
            class User:
                name: str

            schema = extract_type_schema(User)
            assert schema["type"] == "object"


class TestBackwardCompatibility:
    """Test backward compatibility."""

    def test_extract_type_info_alias(self):
        # extract_type_info should be an alias for extract_type_schema
        from cclab.quasar.type_extraction import extract_type_info

        schema = extract_type_info(str)
        assert schema == {"type": "string"}
