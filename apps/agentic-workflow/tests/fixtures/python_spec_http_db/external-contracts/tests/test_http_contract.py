from fastapi.testclient import TestClient

from product_api.interface.http import create_app


def client() -> TestClient:
    return TestClient(create_app())


def test_behavior_create_then_read_uses_persistence_boundary() -> None:
    api = client()

    created = api.post("/products", json={"sku": "book7", "name": "Python reference"})
    assert created.status_code == 201
    assert created.json() == {"id": 1, "sku": "BOOK7", "name": "Python reference"}

    read = api.get("/products/1")
    assert read.status_code == 200
    assert read.json() == created.json()


def test_behavior_duplicate_sku_is_persistence_constraint_failure() -> None:
    api = client()
    assert api.post("/products", json={"sku": "book7", "name": "First"}).status_code == 201

    duplicate = api.post("/products", json={"sku": "BOOK7", "name": "Second"})
    assert duplicate.status_code == 409
    assert duplicate.json()["detail"] == "sku already exists"


def test_security_boundary_rejects_persistence_id_and_invalid_sku() -> None:
    api = client()

    injected_id = api.post(
        "/products", json={"id": 999, "sku": "book7", "name": "Injected identifier"}
    )
    assert injected_id.status_code == 422

    invalid_sku = api.post("/products", json={"sku": "../../secrets", "name": "Invalid"})
    assert invalid_sku.status_code == 422
