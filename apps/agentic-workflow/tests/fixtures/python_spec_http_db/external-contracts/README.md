# HTTP/database external contract

These cases observe the FastAPI boundary through HTTP requests only. They prove
the create/read journey, Pydantic boundary validation, and SQLite's unique-SKU
constraint mapping.

SQLite is a local verifier for schema and constraint behavior. It is not
production stability, concurrency, migration, replication, backup, or
performance evidence; those claims require a production-database contract.
