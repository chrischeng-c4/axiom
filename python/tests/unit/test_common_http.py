"""Tests for shared HTTP types."""

import pytest
from cclab.core import HttpMethod, HttpStatus, BaseRequest, BaseResponse
from dataclasses import dataclass


class TestHttpMethod:
    """Tests for HttpMethod enum."""

    def test_method_values(self):
        assert HttpMethod.GET.value == "GET"
        assert HttpMethod.POST.value == "POST"
        assert HttpMethod.PUT.value == "PUT"
        assert HttpMethod.PATCH.value == "PATCH"
        assert HttpMethod.DELETE.value == "DELETE"
        assert HttpMethod.HEAD.value == "HEAD"
        assert HttpMethod.OPTIONS.value == "OPTIONS"

    def test_method_string_conversion(self):
        assert str(HttpMethod.GET) == "GET"
        assert str(HttpMethod.POST) == "POST"


class TestHttpStatus:
    """Tests for HttpStatus class."""

    def test_is_success(self):
        assert HttpStatus(200).is_success()
        assert HttpStatus(201).is_success()
        assert HttpStatus(204).is_success()
        assert not HttpStatus(400).is_success()
        assert not HttpStatus(500).is_success()

    def test_is_client_error(self):
        assert HttpStatus(400).is_client_error()
        assert HttpStatus(404).is_client_error()
        assert HttpStatus(422).is_client_error()
        assert not HttpStatus(200).is_client_error()
        assert not HttpStatus(500).is_client_error()

    def test_is_server_error(self):
        assert HttpStatus(500).is_server_error()
        assert HttpStatus(503).is_server_error()
        assert not HttpStatus(200).is_server_error()
        assert not HttpStatus(400).is_server_error()

    def test_is_redirect(self):
        assert HttpStatus(301).is_redirect()
        assert HttpStatus(302).is_redirect()
        assert not HttpStatus(200).is_redirect()

    def test_class_constants(self):
        assert HttpStatus.OK.code == 200
        assert HttpStatus.CREATED.code == 201
        assert HttpStatus.BAD_REQUEST.code == 400
        assert HttpStatus.NOT_FOUND.code == 404
        assert HttpStatus.INTERNAL_SERVER_ERROR.code == 500


class TestBaseResponse:
    """Tests for BaseResponse class."""

    @dataclass
    class ConcreteResponse(BaseResponse):
        _body: bytes = b""

        def body_bytes(self) -> bytes:
            return self._body

    def test_status_helpers(self):
        resp = self.ConcreteResponse(status_code=200)
        assert resp.is_success()
        assert not resp.is_client_error()
        assert not resp.is_server_error()

        resp = self.ConcreteResponse(status_code=404)
        assert not resp.is_success()
        assert resp.is_client_error()

        resp = self.ConcreteResponse(status_code=500)
        assert resp.is_server_error()

    def test_header_lookup(self):
        resp = self.ConcreteResponse(
            status_code=200,
            headers={"Content-Type": "application/json", "X-Custom": "value"}
        )
        assert resp.header("content-type") == "application/json"
        assert resp.header("Content-Type") == "application/json"
        assert resp.header("CONTENT-TYPE") == "application/json"
        assert resp.header("x-custom") == "value"
        assert resp.header("nonexistent") is None

    def test_content_type_property(self):
        resp = self.ConcreteResponse(
            status_code=200,
            headers={"Content-Type": "text/html"}
        )
        assert resp.content_type == "text/html"

    def test_content_length_property(self):
        resp = self.ConcreteResponse(
            status_code=200,
            headers={"Content-Length": "123"}
        )
        assert resp.content_length == 123

        resp = self.ConcreteResponse(status_code=200)
        assert resp.content_length is None

    def test_status_property(self):
        resp = self.ConcreteResponse(status_code=404)
        status = resp.status
        assert isinstance(status, HttpStatus)
        assert status.code == 404
        assert status.is_client_error()


class TestBaseRequest:
    """Tests for BaseRequest class."""

    @dataclass
    class ConcreteRequest(BaseRequest):
        _body: bytes = b""

        def body_bytes(self):
            return self._body if self._body else None

    def test_header_lookup(self):
        req = self.ConcreteRequest(
            method="GET",
            url="/test",
            headers={"Authorization": "Bearer token", "Accept": "application/json"}
        )
        assert req.header("authorization") == "Bearer token"
        assert req.header("Authorization") == "Bearer token"
        assert req.header("accept") == "application/json"
        assert req.header("nonexistent") is None

    def test_content_type_property(self):
        req = self.ConcreteRequest(
            method="POST",
            url="/test",
            headers={"Content-Type": "application/json"}
        )
        assert req.content_type == "application/json"
