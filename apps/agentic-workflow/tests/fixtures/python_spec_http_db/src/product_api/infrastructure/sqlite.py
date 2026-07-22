from sqlalchemy import String, create_engine
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column, sessionmaker
from sqlalchemy.pool import StaticPool


class Base(DeclarativeBase):
    pass


class ProductRecord(Base):
    __tablename__ = "products"

    id: Mapped[int] = mapped_column(primary_key=True)
    sku: Mapped[str] = mapped_column(String(64), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)


def session_factory(database_url: str):
    options = {"future": True}
    if database_url == "sqlite+pysqlite:///:memory:":
        options.update({"connect_args": {"check_same_thread": False}, "poolclass": StaticPool})
    engine = create_engine(database_url, **options)
    Base.metadata.create_all(engine)
    return sessionmaker(bind=engine, expire_on_commit=False)
