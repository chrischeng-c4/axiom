"""Tests for OpenAPI generation."""

import pytest
from typing import Annotated, List, Optional
from dataclasses import dataclass

from cclab.quasar import App, Path, Query, Body, Header
from cclab.quasar.openapi import (
    generate_openapi,
    python_type_to_schema,
    get_swagger_ui_html,
    get_redoc_html,
    type_schema_to_openapi,
    extract_operation_params,
    build_operation,
)
from cclab.quasar.app import RouteInfo


class TestOpenAPIGeneration:
    """Test OpenAPI schema generation."""

    def test_basic_spec(self):
        """Test basic OpenAPI spec generation."""
        spec = generate_openapi(
            title="Test API",
            version="1.0.0",
            description="A test API",
        )

        assert spec["openapi"] == "3.1.0"
        assert spec["info"]["title"] == "Test API"
        assert spec["info"]["version"] == "1.0.0"
        assert spec["info"]["description"] == "A test API"
        assert "paths" in spec
        assert "components" in spec

    def test_with_servers(self):
        """Test OpenAPI spec with server configurations."""
        spec = generate_openapi(
            title="Test",
            version="1.0.0",
            servers=[
                {"url": "https://api.example.com", "description": "Production"},
                {"url": "https://staging.example.com", "description": "Staging"},
            ],
        )

        assert len(spec["servers"]) == 2
        assert spec["servers"][0]["url"] == "https://api.example.com"
        assert spec["servers"][1]["description"] == "Staging"

    def test_with_tags(self):
        """Test OpenAPI spec with tag configurations."""
        spec = generate_openapi(
            title="Test",
            version="1.0.0",
            tags=[
                {"name": "users", "description": "User operations"},
                {"name": "posts", "description": "Post operations"},
            ],
        )

        assert len(spec["tags"]) == 2
        assert spec["tags"][0]["name"] == "users"
        assert spec["tags"][1]["description"] == "Post operations"


class TestSchemaGeneration:
    """Test Python type to OpenAPI schema conversion."""

    def test_string_type(self):
        """Test string type conversion."""
        schema = python_type_to_schema(str, {})
        assert schema == {"type": "string"}

    def test_int_type(self):
        """Test int type conversion."""
        schema = python_type_to_schema(int, {})
        assert schema == {"type": "integer"}

    def test_float_type(self):
        """Test float type conversion."""
        schema = python_type_to_schema(float, {})
        assert schema == {"type": "number"}

    def test_bool_type(self):
        """Test bool type conversion."""
        schema = python_type_to_schema(bool, {})
        assert schema == {"type": "boolean"}

    def test_list_type(self):
        """Test List type conversion."""
        schema = python_type_to_schema(List[str], {})
        assert schema["type"] == "array"
        assert schema["items"]["type"] == "string"

    def test_optional_type(self):
        """Test Optional type conversion."""
        schema = python_type_to_schema(Optional[str], {})
        assert schema["type"] == "string"
        assert schema["nullable"] is True

    def test_list_of_optional(self):
        """Test List[Optional[T]] type conversion."""
        schema = python_type_to_schema(List[Optional[int]], {})
        assert schema["type"] == "array"
        assert schema["items"]["type"] == "integer"
        assert schema["items"]["nullable"] is True

    def test_dataclass_schema(self):
        """Test dataclass type conversion with $ref."""
        @dataclass
        class User:
            name: str
            age: int

        schemas = {}
        schema = python_type_to_schema(User, schemas)

        assert "$ref" in schema
        assert "User" in schemas
        assert schemas["User"]["type"] == "object"
        assert "name" in schemas["User"]["properties"]
        assert "age" in schemas["User"]["properties"]

    def test_nested_dataclass_schema(self):
        """Test nested dataclass type conversion."""
        @dataclass
        class Address:
            street: str
            city: str

        @dataclass
        class User:
            name: str
            address: Address

        schemas = {}
        schema = python_type_to_schema(User, schemas)

        assert "$ref" in schema
        assert "User" in schemas
        assert "Address" in schemas
        assert schemas["User"]["properties"]["address"]["$ref"] == "#/components/schemas/Address"


class TestParameterExtraction:
    """Test parameter extraction from handler signatures."""

    def test_path_parameter(self):
        """Test path parameter extraction."""
        async def handler(user_id: Annotated[str, Path(description="User ID")]) -> dict:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert len(parameters) == 1
        assert parameters[0]["name"] == "user_id"
        assert parameters[0]["in"] == "path"
        assert parameters[0]["required"] is True
        assert parameters[0]["schema"]["type"] == "string"
        assert parameters[0]["description"] == "User ID"

    def test_query_parameters(self):
        """Test query parameter extraction."""
        async def handler(
            skip: Annotated[int, Query(default=0)],
            limit: Annotated[int, Query(default=10)],
        ) -> List[dict]:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert len(parameters) == 2
        assert parameters[0]["name"] == "skip"
        assert parameters[0]["in"] == "query"
        assert parameters[0]["required"] is False
        assert parameters[0]["schema"]["default"] == 0

        assert parameters[1]["name"] == "limit"
        assert parameters[1]["schema"]["default"] == 10

    def test_header_parameter(self):
        """Test header parameter extraction."""
        async def handler(
            authorization: Annotated[str, Header(alias="Authorization")]
        ) -> dict:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert len(parameters) == 1
        assert parameters[0]["name"] == "Authorization"
        assert parameters[0]["in"] == "header"
        assert parameters[0]["required"] is True

    def test_request_body(self):
        """Test request body extraction."""
        @dataclass
        class CreateUser:
            name: str
            email: str

        async def handler(user: Annotated[CreateUser, Body()]) -> dict:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert len(parameters) == 0
        assert request_body is not None
        assert request_body["required"] is True
        assert "application/json" in request_body["content"]
        assert "$ref" in request_body["content"]["application/json"]["schema"]

    def test_response_schema(self):
        """Test response schema extraction."""
        @dataclass
        class User:
            id: str
            name: str

        async def handler() -> User:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert response_schema is not None
        assert "$ref" in response_schema
        assert "User" in schemas

    def test_list_response_schema(self):
        """Test list response schema extraction."""
        @dataclass
        class User:
            id: str
            name: str

        async def handler() -> List[User]:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert response_schema is not None
        assert response_schema["type"] == "array"
        assert "$ref" in response_schema["items"]

    def test_mixed_parameters(self):
        """Test extraction of mixed parameter types."""
        @dataclass
        class UpdateUser:
            name: Optional[str]
            email: Optional[str]

        async def handler(
            user_id: Annotated[str, Path()],
            force: Annotated[bool, Query(default=False)],
            user: Annotated[UpdateUser, Body()],
        ) -> dict:
            pass

        schemas = {}
        parameters, request_body, response_schema = extract_operation_params(handler, schemas)

        assert len(parameters) == 2
        assert parameters[0]["in"] == "path"
        assert parameters[1]["in"] == "query"
        assert request_body is not None


class TestRouteInfo:
    """Test route info to OpenAPI operation conversion."""

    def test_build_operation_basic(self):
        """Test basic operation building."""
        async def handler(user_id: Annotated[str, Path()]) -> dict:
            """Get a user by ID."""
            pass

        route = RouteInfo(
            method="GET",
            path="/users/{user_id}",
            handler=handler,
            name="get_user",
            summary="Get user",
            description="Retrieve a user by ID",
            tags=["users"],
            deprecated=False,
            status_code=200,
        )

        schemas = {}
        operation = build_operation(route, schemas)

        assert operation["operationId"] == "get_user"
        assert operation["summary"] == "Get user"
        assert operation["description"] == "Retrieve a user by ID"
        assert operation["tags"] == ["users"]
        assert "200" in operation["responses"]
        assert len(operation["parameters"]) == 1

    def test_build_operation_deprecated(self):
        """Test deprecated operation."""
        async def handler() -> dict:
            pass

        route = RouteInfo(
            method="GET",
            path="/old-endpoint",
            handler=handler,
            name="old_endpoint",
            summary=None,
            description=None,
            tags=[],
            deprecated=True,
            status_code=200,
        )

        schemas = {}
        operation = build_operation(route, schemas)

        assert operation["deprecated"] is True


class TestAppOpenAPI:
    """Test App OpenAPI integration."""

    def test_app_openapi_schema(self):
        """Test App.openapi() method."""
        app = App(title="My API", version="2.0.0")

        @app.get("/users/{user_id}")
        async def get_user(user_id: Annotated[str, Path()]) -> dict:
            """Get a user."""
            pass

        spec = app.openapi()

        assert spec["info"]["title"] == "My API"
        assert spec["info"]["version"] == "2.0.0"
        assert "/users/{user_id}" in spec["paths"]
        assert "get" in spec["paths"]["/users/{user_id}"]

    def test_app_openapi_json(self):
        """Test App.openapi_json() method."""
        app = App(title="Test", version="1.0.0")

        @app.get("/test")
        async def test_handler() -> dict:
            pass

        json_str = app.openapi_json()

        import json
        spec = json.loads(json_str)
        assert spec["info"]["title"] == "Test"

    def test_operation_parameters(self):
        """Test operation parameters in spec."""
        app = App(title="Test", version="1.0.0")

        @app.get("/items")
        async def list_items(
            skip: Annotated[int, Query(default=0)],
            limit: Annotated[int, Query(default=10)],
        ) -> List[dict]:
            """List items."""
            pass

        spec = app.openapi()
        params = spec["paths"]["/items"]["get"]["parameters"]

        assert len(params) == 2
        assert any(p["name"] == "skip" for p in params)
        assert any(p["name"] == "limit" for p in params)

    def test_request_body(self):
        """Test request body in spec."""
        app = App(title="Test", version="1.0.0")

        @dataclass
        class CreateUser:
            name: str
            email: str

        @app.post("/users")
        async def create_user(user: Annotated[CreateUser, Body()]) -> dict:
            """Create a user."""
            pass

        spec = app.openapi()
        body = spec["paths"]["/users"]["post"]["requestBody"]

        assert body["required"] is True
        assert "application/json" in body["content"]

    def test_multiple_routes(self):
        """Test multiple routes in spec."""
        app = App(title="Test", version="1.0.0")

        @app.get("/users")
        async def list_users() -> List[dict]:
            pass

        @app.post("/users")
        async def create_user(name: Annotated[str, Body()]) -> dict:
            pass

        @app.get("/users/{user_id}")
        async def get_user(user_id: Annotated[str, Path()]) -> dict:
            pass

        spec = app.openapi()

        assert "/users" in spec["paths"]
        assert "/users/{user_id}" in spec["paths"]
        assert "get" in spec["paths"]["/users"]
        assert "post" in spec["paths"]["/users"]
        assert "get" in spec["paths"]["/users/{user_id}"]


class TestDocumentation:
    """Test documentation HTML generation."""

    def test_swagger_ui_html(self):
        """Test Swagger UI HTML generation."""
        html = get_swagger_ui_html("My API", "/openapi.json")

        assert "My API" in html
        assert "/openapi.json" in html
        assert "swagger-ui" in html
        assert "SwaggerUIBundle" in html

    def test_redoc_html(self):
        """Test ReDoc HTML generation."""
        html = get_redoc_html("My API", "/openapi.json")

        assert "My API" in html
        assert "/openapi.json" in html
        assert "redoc" in html
        assert "spec-url" in html

    def test_setup_docs(self):
        """Test setup_docs method."""
        app = App(
            title="Test API",
            version="1.0.0",
            docs_url="/docs",
            redoc_url="/redoc",
            openapi_url="/openapi.json",
        )

        # Setup docs
        app.setup_docs()

        # Check that routes were registered
        routes = app.routes
        route_paths = [r.path for r in routes]

        assert "/openapi.json" in route_paths
        assert "/docs" in route_paths
        assert "/redoc" in route_paths

    def test_setup_docs_idempotent(self):
        """Test that setup_docs can be called multiple times."""
        app = App(title="Test", version="1.0.0")

        app.setup_docs()
        routes_count_1 = len(app.routes)

        app.setup_docs()
        routes_count_2 = len(app.routes)

        # Should not add duplicate routes
        assert routes_count_1 == routes_count_2

    def test_setup_docs_disabled(self):
        """Test setup_docs with docs disabled."""
        app = App(
            title="Test",
            version="1.0.0",
            docs_url=None,
            redoc_url=None,
            openapi_url=None,
        )

        app.setup_docs()

        # No routes should be added
        assert len(app.routes) == 0


class TestTypeSchemaConversion:
    """Test internal type schema to OpenAPI conversion."""

    def test_string_schema(self):
        """Test string schema conversion."""
        internal = {"type": "string", "min_length": 1, "max_length": 100}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi["type"] == "string"
        assert openapi["minLength"] == 1
        assert openapi["maxLength"] == 100

    def test_integer_schema_with_constraints(self):
        """Test integer schema with constraints."""
        internal = {"type": "int", "minimum": 0, "maximum": 100}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi["type"] == "integer"
        assert openapi["minimum"] == 0
        assert openapi["maximum"] == 100

    def test_array_schema(self):
        """Test array schema conversion."""
        internal = {"type": "list", "items": {"type": "string"}}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi["type"] == "array"
        assert openapi["items"]["type"] == "string"

    def test_object_schema(self):
        """Test object schema conversion."""
        internal = {
            "type": "object",
            "fields": [
                {"name": "id", "type": {"type": "string"}, "required": True},
                {"name": "count", "type": {"type": "int"}, "required": False},
            ],
        }
        openapi = type_schema_to_openapi(internal, {})

        assert openapi["type"] == "object"
        assert "id" in openapi["properties"]
        assert "count" in openapi["properties"]
        assert openapi["required"] == ["id"]

    def test_object_with_ref(self):
        """Test object schema with $ref."""
        internal = {
            "type": "object",
            "class_name": "User",
            "fields": [
                {"name": "name", "type": {"type": "string"}, "required": True},
            ],
        }
        schemas = {}
        openapi = type_schema_to_openapi(internal, schemas)

        assert openapi == {"$ref": "#/components/schemas/User"}
        assert "User" in schemas

    def test_optional_schema(self):
        """Test optional schema conversion."""
        internal = {"type": "optional", "inner": {"type": "string"}}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi["type"] == "string"
        assert openapi["nullable"] is True

    def test_union_schema(self):
        """Test union schema conversion."""
        internal = {
            "type": "union",
            "variants": [{"type": "string"}, {"type": "int"}],
            "nullable": False,
        }
        openapi = type_schema_to_openapi(internal, {})

        assert "anyOf" in openapi
        assert len(openapi["anyOf"]) == 2

    def test_special_formats(self):
        """Test special format conversions."""
        test_cases = [
            ({"type": "uuid"}, {"type": "string", "format": "uuid"}),
            ({"type": "datetime"}, {"type": "string", "format": "date-time"}),
            ({"type": "date"}, {"type": "string", "format": "date"}),
            ({"type": "email"}, {"type": "string", "format": "email"}),
            ({"type": "url"}, {"type": "string", "format": "uri"}),
        ]

        for internal, expected in test_cases:
            openapi = type_schema_to_openapi(internal, {})
            assert openapi == expected

    def test_enum_schema(self):
        """Test enum schema conversion."""
        internal = {"type": "enum", "values": ["draft", "published", "archived"]}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi == {"enum": ["draft", "published", "archived"]}

    def test_literal_schema(self):
        """Test literal schema conversion."""
        internal = {"type": "literal", "values": ["admin", "user"]}
        openapi = type_schema_to_openapi(internal, {})

        assert openapi == {"enum": ["admin", "user"]}
