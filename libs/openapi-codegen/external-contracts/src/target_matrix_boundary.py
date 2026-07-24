"""Exact generated-client profile matrix; missing cases cannot false-green."""

from dataclasses import dataclass


@dataclass(frozen=True)
class MatrixCase:
    check_id: str
    test_name: str

    @property
    def command(self) -> tuple[str, ...]:
        return (
            "cargo",
            "test",
            "-p",
            "openapi-codegen",
            "--test",
            "target_profile_matrix",
            self.test_name,
            "--",
            "--exact",
        )


MATRIX_CASES = (
    MatrixCase("matrix-python-311", "python_311_generated_model_contract"),
    MatrixCase("matrix-python-312", "python_312_generated_model_contract"),
    MatrixCase("matrix-python-313", "python_313_generated_model_contract"),
    MatrixCase("matrix-python-314", "python_314_generated_model_contract"),
    MatrixCase("matrix-typescript-50", "typescript_50_strict_modern_module_contract"),
    MatrixCase("matrix-rust-2021", "rust_2021_generated_client_contract"),
    MatrixCase(
        "matrix-rust-2024",
        "rust_2024_generated_client_gen_property_contract",
    ),
    MatrixCase("matrix-legacy-default", "legacy_default_output_contract"),
    MatrixCase(
        "matrix-deterministic-artifacts",
        "all_target_requirements_and_artifacts_are_deterministic",
    ),
)

EXPECTED_CHECK_IDS = tuple(case.check_id for case in MATRIX_CASES)
