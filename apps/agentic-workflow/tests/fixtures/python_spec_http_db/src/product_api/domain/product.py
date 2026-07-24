from dataclasses import dataclass


class InvalidProduct(ValueError):
    """Raised when a product cannot exist in the domain."""


@dataclass(frozen=True)
class Product:
    id: int
    sku: str
    name: str


def new_product(*, sku: str, name: str) -> Product:
    normalized_sku = sku.strip().upper()
    normalized_name = name.strip()
    if not normalized_sku.isalnum():
        raise InvalidProduct("sku must be alphanumeric")
    if not normalized_name:
        raise InvalidProduct("name must not be blank")
    return Product(id=0, sku=normalized_sku, name=normalized_name)
