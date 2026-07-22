from product_api.application.products import create_product
from product_api.domain.product import Product


class RecordingRepository:
    def __init__(self) -> None:
        self.received: list[Product] = []

    def add(self, product: Product) -> Product:
        self.received.append(product)
        return Product(id=7, sku=product.sku, name=product.name)

    def get(self, product_id: int) -> Product | None:
        return None


def test_create_product_normalizes_at_application_boundary() -> None:
    repository = RecordingRepository()

    product = create_product(repository, sku=" sku7 ", name="  Release notes ")

    assert product == Product(id=7, sku="SKU7", name="Release notes")
    assert repository.received == [Product(id=0, sku="SKU7", name="Release notes")]
