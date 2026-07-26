"""Target-neutral TD declaration for a generated Pet API client."""

__aw_artifact_id__ = "artifact:openapi-target-profile/pet-api-client"


@openapi_client(
    source="openapi/pet.json",
    python="python-3.12",
    typescript="typescript-5.0",
    rust="rust-2024",
)
class PetApi:
    pass
