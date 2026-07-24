from sqlalchemy import select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session

from product_api.application.products import DuplicateSku
from product_api.domain.product import Product
from product_api.infrastructure.sqlite import ProductRecord


class SqlAlchemyProductRepository:
    def __init__(self, session: Session) -> None:
        self._session = session

    def add(self, product: Product) -> Product:
        record = ProductRecord(sku=product.sku, name=product.name)
        self._session.add(record)
        try:
            self._session.commit()
        except IntegrityError as error:
            self._session.rollback()
            raise DuplicateSku("sku already exists") from error
        return Product(id=record.id, sku=record.sku, name=record.name)

    def get(self, product_id: int) -> Product | None:
        record = self._session.scalar(select(ProductRecord).where(ProductRecord.id == product_id))
        if record is None:
            return None
        return Product(id=record.id, sku=record.sku, name=record.name)
