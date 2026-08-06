"""Domain validation service for peer TLS."""

from __future__ import annotations

from datetime import datetime

from peer_tls.domain.identity import ExpectationKind, IdentityExpectation
from peer_tls.domain.material import MaterialTriple
from peer_tls.domain.verdict import (
    MaterialVerdict,
    Rejection,
    RejectionReason,
    ValidatedMaterial,
    ValidityWindow,
)


def decide_material(
    triple: MaterialTriple,
    expectation: IdentityExpectation,
    instant: datetime,
) -> MaterialVerdict:
    if not expectation.is_well_formed():
        return Rejection(
            reason=RejectionReason.MALFORMED_EXPECTATION,
            detail=f"Expectation of kind {expectation.kind} is malformed",
        )

    if triple.key.public_key_fingerprint != triple.leaf.public_key_fingerprint:
        return Rejection(
            reason=RejectionReason.KEY_DOES_NOT_MATCH_LEAF,
            detail=f"Key fingerprint '{triple.key.public_key_fingerprint}' does not match leaf fingerprint '{triple.leaf.public_key_fingerprint}'",
        )

    if not triple.trust.admits(triple.leaf.issuer_key_id):
        return Rejection(
            reason=RejectionReason.ISSUER_NOT_IN_TRUST_BUNDLE,
            detail=f"Issuer key ID '{triple.leaf.issuer_key_id}' not admitted by trust bundle",
        )

    if expectation.kind == ExpectationKind.SERVING:
        expected_dns_set = {d.value for d in expectation.dns_names}
        san_dns_set = {d.value for d in triple.leaf.subject_alt_names.dns_names}

        if expected_dns_set & san_dns_set:
            pass  # Identity matched in SAN DNS
        elif triple.leaf.common_name in expected_dns_set or any(
            uri in expected_dns_set for uri in triple.leaf.subject_alt_names.uris
        ):
            return Rejection(
                reason=RejectionReason.IDENTITY_IN_WRONG_EXTENSION,
                detail=f"Expected DNS name present in wrong extension (CN '{triple.leaf.common_name}' or SAN URIs)",
            )
        else:
            return Rejection(
                reason=RejectionReason.IDENTITY_MISMATCH,
                detail=f"None of expected DNS names {sorted(expected_dns_set)} found in leaf SAN DNS names",
            )

    elif expectation.kind == ExpectationKind.PEER:
        if expectation.spiffe_id is None:
            return Rejection(
                reason=RejectionReason.MALFORMED_EXPECTATION,
                detail="PEER expectation must specify spiffe_id",
            )
        target_uri = expectation.spiffe_id.uri
        target_path = expectation.spiffe_id.path.lstrip("/")
        target_td = expectation.spiffe_id.trust_domain.value

        san_uris = triple.leaf.subject_alt_names.uris
        san_dns = {d.value for d in triple.leaf.subject_alt_names.dns_names}

        if target_uri in san_uris:
            pass  # Identity matched in SAN URIs
        elif triple.leaf.common_name == target_uri or target_uri in san_dns:
            return Rejection(
                reason=RejectionReason.IDENTITY_IN_WRONG_EXTENSION,
                detail=f"SPIFFE URI '{target_uri}' present in wrong extension (CN or SAN DNS)",
            )
        else:
            # Check for trust domain mismatch (path matches a SAN URI, but trust domain differs)
            path_match_wrong_td = False
            for uri in san_uris:
                if uri.startswith("spiffe://"):
                    body = uri[len("spiffe://") :]
                    parts = body.split("/", 1)
                    if len(parts) == 2:
                        uri_td, uri_path = parts[0], parts[1].lstrip("/")
                        if uri_path == target_path and uri_td != target_td:
                            path_match_wrong_td = True
                            break

            if path_match_wrong_td:
                return Rejection(
                    reason=RejectionReason.TRUST_DOMAIN_MISMATCH,
                    detail=f"SPIFFE path '{target_path}' matched but trust domain mismatched expected '{target_td}'",
                )
            else:
                return Rejection(
                    reason=RejectionReason.IDENTITY_MISMATCH,
                    detail=f"Expected SPIFFE URI '{target_uri}' not found in leaf SAN URIs",
                )

    if instant < triple.leaf.not_before:
        return Rejection(
            reason=RejectionReason.NOT_YET_VALID,
            detail=f"Instant {instant} is before leaf not_before {triple.leaf.not_before}",
        )

    if instant > triple.leaf.not_after:
        return Rejection(
            reason=RejectionReason.EXPIRED,
            detail=f"Instant {instant} is after leaf not_after {triple.leaf.not_after}",
        )

    return ValidatedMaterial(
        window=ValidityWindow(not_before=triple.leaf.not_before, not_after=triple.leaf.not_after),
        identity=expectation,
    )
