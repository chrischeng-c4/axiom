from typing import Protocol

from product_api.domain.product import Product, new_product


class DuplicateSku(ValueError):
    """Raised when persistence reports an existing business key."""


class ProductRepository(Protocol):
    def add(self, product: Product) -> Product: ...

    def get(self, product_id: int) -> Product | None: ...


def create_product(repository: ProductRepository, *, sku: str, name: str) -> Product:
    return repository.add(new_product(sku=sku, name=name))


def get_product(repository: ProductRepository, product_id: int) -> Product | None:
    return repository.get(product_id)
