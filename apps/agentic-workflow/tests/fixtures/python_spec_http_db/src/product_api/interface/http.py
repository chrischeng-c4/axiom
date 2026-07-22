from fastapi import FastAPI, HTTPException, status
from pydantic import BaseModel, ConfigDict, Field

from product_api.application.products import DuplicateSku, create_product, get_product
from product_api.domain.product import InvalidProduct, Product
from product_api.infrastructure.repository import SqlAlchemyProductRepository
from product_api.infrastructure.sqlite import session_factory


class CreateProductRequest(BaseModel):
    model_config = ConfigDict(extra="forbid")

    sku: str = Field(min_length=1, max_length=64)
    name: str = Field(min_length=1, max_length=200)


class ProductResponse(BaseModel):
    id: int
    sku: str
    name: str

    @classmethod
    def from_domain(cls, product: Product) -> "ProductResponse":
        return cls(id=product.id, sku=product.sku, name=product.name)


def create_app(database_url: str = "sqlite+pysqlite:///:memory:") -> FastAPI:
    sessions = session_factory(database_url)
    app = FastAPI(title="Product reference fixture")

    @app.post("/products", response_model=ProductResponse, status_code=status.HTTP_201_CREATED)
    def create(request: CreateProductRequest) -> ProductResponse:
        with sessions() as session:
            repository = SqlAlchemyProductRepository(session)
            try:
                product = create_product(repository, sku=request.sku, name=request.name)
            except InvalidProduct as error:
                raise HTTPException(status_code=422, detail=str(error)) from error
            except DuplicateSku as error:
                raise HTTPException(status_code=409, detail=str(error)) from error
        return ProductResponse.from_domain(product)

    @app.get("/products/{product_id}", response_model=ProductResponse)
    def read(product_id: int) -> ProductResponse:
        with sessions() as session:
            product = get_product(SqlAlchemyProductRepository(session), product_id)
        if product is None:
            raise HTTPException(status_code=404, detail="product not found")
        return ProductResponse.from_domain(product)

    return app
