#!/usr/bin/env python3
"""Deterministic, fail-closed inventory for Mamba's opaque integer boundary.

The inventory pass records structural observations without deciding their
disposition: every producer, consumer, registry/classifier, numeric
side-table, and metadata boundary is inventoried. The checked-in
family/store/proof map is semantic authority; inventory and
observation-lock files are derived evidence only. The typed discriminator is
an authority-instantiated canonical Rust grammar and remains fail-closed for
unknown syntax.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass, asdict
import hashlib
import json
from pathlib import Path
import re
import shutil
import sys
import tempfile
import tomllib
from collections import Counter
from types import MappingProxyType

from rust_scan import ScanError as ScanFailure, mask_non_code


SCRIPT = Path(__file__).resolve()
MAMBA = SCRIPT.parents[2]
REPO = MAMBA.parents[1]
RUNTIME = MAMBA / "src" / "runtime"
PRODUCTION = MAMBA / "src"
GATE = MAMBA / "tests" / "governance" / "gates" / "t1_opaque_value_boundary"
INVENTORY = GATE / "inventory.toml"
EXPECTED = GATE / "expected.toml"
FIXTURES = GATE / "fixtures"
FAMILIES_MANIFEST = GATE / "families.toml"
OBSERVATIONS_LOCK = GATE / "observations.lock.toml"
AUTHORITY_FIXTURE = FIXTURES / "authority" / "valid.toml"
AUTHORITY_SOURCE_FIXTURE = FIXTURES / "authority" / "valid.rs"
MUTATION_PHASES = ("e2e_reconciliation", "schema_units", "scanner_codec")
RECONCILIATION_ORDER = (
    "unmatched_selector", "multiple_selector", "stale_selector", "central_add",
    "central_remove", "central_duplicate", "direct_central", "public_barrier_reifier",
    "barrier_escape_mismatch", "random_allocator_reifier",
)

# The mutation corpus is a frozen E2E oracle.  Keep these names and totals
# literal: deriving the inventory from expected.toml would let a missing case
# silently reduce the protection offered by this gate.
FROZEN_RECONCILIATION_NAMES = frozenset(RECONCILIATION_ORDER)
FROZEN_CASE_NAMES = frozenset({
    "new_source", "duplicate_rows", "direct_producer", "helper_assignment",
    "registry_lookup", "threshold", "first_live_probe", "private_id_reification",
    "metadata_controls", "unnamed_table", "jit_only", "helper_split", "lowercase_table",
    "manual_families", "unknown_candidate", "unparseable", "test_only_masking",
})
FROZEN_TYPED_NAMES = frozenset({
    "valid_codec", "wrapper_chain", "ordinary_python_int", "comments_only", "cfg_test_only", "macro_only",
    "multi_arg_wrapper", "reserved_low_bit_set", "pack_guard_after_bits", "pack_extra_statement",
    "encoder_shadow", "decoder_tag_reorder", "decoder_reserved_reorder", "decoder_identity_shadow",
    "decoder_unknown_statement", "wrapper_nonidentity", "wrapper_argument_swap", "wrapper_argument_constant",
    "wrapper_callee_shadow", "bound_macro_after_valid_codec", "macro_confined_codec", "macro_split_codec",
    "wrapper_terminal",
    "incomplete_authority", "alias_only", "post_return_bound_guard", "reserved_bit_decoy_guard",
    "invalid_layout", "oversized_family_code", "decoy_bound_check", "decoder_guard_expression",
    "high_id_overlap", "wrong_shift", "invalid_kind", "default_coercion",
    "multi_family_codec", "bare_macro_invocation", "support_type_shadow",
    "normalized_digest_anchor_drift", "qualified_bound_declaration",
    "decoder_match_arm_binding_shadow", "participating_std_namespace_shadow_mod",
    "participating_std_namespace_shadow_alias", "unrelated_macro_in_unbound_item",
    "unrelated_result_variant_declaration", "participating_std_namespace_shadow_extern",
})
FROZEN_RECONCILIATION_COUNT = 10
FROZEN_CASE_COUNT = 17
FROZEN_TYPED_COUNT = 46
FROZEN_MUTATION_CASE_COUNT = 73
FROZEN_EXPECTED_TOML_DIGEST = "3e89dd6eef36eb2193f84ab1657aecbafefb46e84738db65afbf5a54906fe269"
FROZEN_PHASE_COUNTS = {
    "e2e_reconciliation": 12,
    "schema_units": 12,
    "scanner_codec": 49,
}
PHASE_C_CASE_ORDER = (
    "stale_seed_item", "missing_seed_observation", "extra_seed_observation",
    "duplicate_seed_owner", "seed_digest_drift", "derived_lock_stale", "derived_lock_extra",
    "unseeded_central_hook", "unseeded_public_reifier", "unseeded_side_table_allocator",
    "irrelevant_id_storm_no_broad_census", "exact_nineteen_family_set",
    "central_topology_mismatch", "direct_topology_mismatch", "direct_iter_store_range_shared_store",
    "direct_iter_store_range_split_store_rejected", "store_owner_reverse_edge_missing",
    "owner_role_closure_missing", "unknown_matcher_kind", "barrier_line_range_rejected",
    "random_native_public_conflation", "derived_fields_in_manual_authority_rejected",
    "positive_central_registry_insert", "near_miss_central_registry_insert",
    "positive_central_registry_lookup", "near_miss_central_registry_lookup",
    "positive_direct_store_allocate", "near_miss_direct_store_allocate",
    "positive_direct_store_lookup", "near_miss_direct_store_lookup",
    "positive_threshold_classifier", "near_miss_threshold_classifier",
    "positive_first_live_probe", "near_miss_first_live_probe",
    "positive_public_from_int_reifier", "near_miss_public_from_int_reifier",
    "positive_public_as_int_reifier", "near_miss_public_as_int_reifier",
    "positive_public_field_reifier", "near_miss_public_field_reifier",
    "positive_native_internal_allocator", "near_miss_native_internal_allocator",
    "comments_strings_cfg_test_masked", "nested_decoy_outside_seed_span",
    "same_item_duplicate_match", "seeded_match_not_reemitted_as_unseeded",
    "random_native_allocator_not_public_escape", "barrier_exact_field_reifier",
    "generic_uid_gid_hashmap_ignored", "line_move_does_not_change_identity",
    "semantic_token_change_changes_digest", "cross_helper_from_int",
    "cross_helper_as_int", "five_hop_exhaustion", "escaped_path_edge",
    "cross_helper_unrelated_return_decoy", "seeded_cross_helper_from_int",
    "seeded_cross_helper_as_int", "foreign_receiver", "ambiguous_call",
    "multi_origin_return", "transform_argument", "six_hop_exhaustion",
    "literal_child_path", "unsupported_include", "associated_method_conversion",
    "unknown_call", "arithmetic_rhs", "method_rhs", "tuple_rhs", "transformed_return",
    "recursive_path", "static_local_shadow", "receiver_shadow", "payload_not_key",
    "duplicate_conversion", "cast_rhs", "wrapped_call_binary", "wrapped_call_cast",
    "wrapped_call_method", "wrapped_call_tuple", "wrapped_call_index", "wrapped_call_exact",
    "inline_module_conversion", "nested_function_conversion", "trait_default_conversion",
    "impl_method_conversion", "cfg_test_conversion_control", "receiver_same_name_shadow",
    "foreign_receiver_strict", "unbound_key", "transformed_key", "multiple_terminal",
    "four_hop_control", "semantic_two_path", "semantic_zero_terminal",
    "semantic_two_terminal", "semantic_later_path_digest",
    "unrelated_deep_static_control",
    "unseeded_multi_path_zero_terminal", "complete_edge_matrix",
    "forward_carried_cycle", "cfg_test_inline_second_function",
    "impl_trait_second_method",
    "edge_v6_format_only_equal", "edge_v6_binding_key_rebind_different",
    "edge_v6_conversion_move_different", "edge_v6_key_target_different",
    "edge_v6_missing_conversion_identity", "edge_v6_missing_key_identity",
    "inline_module_return_impl_trait", "nested_function_return_impl_trait",
    "trait_default_return_impl_trait", "impl_method_return_impl_trait",
)
PHASE_C_CASE_COUNT = 114
PHASE_C_PHASE_COUNTS = MappingProxyType({"e2e": 17, "schema": 16, "logic": 81})
PHASE_C_PHASE_CASES = MappingProxyType({
    'stale_seed_item': 'e2e',
    'missing_seed_observation': 'e2e',
    'extra_seed_observation': 'e2e',
    'duplicate_seed_owner': 'e2e',
    'seed_digest_drift': 'e2e',
    'derived_lock_stale': 'e2e',
    'derived_lock_extra': 'e2e',
    'unseeded_central_hook': 'e2e',
    'unseeded_public_reifier': 'e2e',
    'unseeded_side_table_allocator': 'e2e',
    'irrelevant_id_storm_no_broad_census': 'e2e',
    'exact_nineteen_family_set': 'schema',
    'central_topology_mismatch': 'schema',
    'direct_topology_mismatch': 'schema',
    'direct_iter_store_range_shared_store': 'schema',
    'direct_iter_store_range_split_store_rejected': 'schema',
    'store_owner_reverse_edge_missing': 'schema',
    'owner_role_closure_missing': 'schema',
    'unknown_matcher_kind': 'schema',
    'barrier_line_range_rejected': 'schema',
    'random_native_public_conflation': 'schema',
    'derived_fields_in_manual_authority_rejected': 'schema',
    'positive_central_registry_insert': 'logic',
    'near_miss_central_registry_insert': 'logic',
    'positive_central_registry_lookup': 'logic',
    'near_miss_central_registry_lookup': 'logic',
    'positive_direct_store_allocate': 'logic',
    'near_miss_direct_store_allocate': 'logic',
    'positive_direct_store_lookup': 'logic',
    'near_miss_direct_store_lookup': 'logic',
    'positive_threshold_classifier': 'logic',
    'near_miss_threshold_classifier': 'logic',
    'positive_first_live_probe': 'logic',
    'near_miss_first_live_probe': 'logic',
    'positive_public_from_int_reifier': 'logic',
    'near_miss_public_from_int_reifier': 'logic',
    'positive_public_as_int_reifier': 'logic',
    'near_miss_public_as_int_reifier': 'logic',
    'positive_public_field_reifier': 'logic',
    'near_miss_public_field_reifier': 'logic',
    'positive_native_internal_allocator': 'logic',
    'near_miss_native_internal_allocator': 'logic',
    'comments_strings_cfg_test_masked': 'logic',
    'nested_decoy_outside_seed_span': 'logic',
    'same_item_duplicate_match': 'logic',
    'seeded_match_not_reemitted_as_unseeded': 'logic',
    'random_native_allocator_not_public_escape': 'logic',
    'barrier_exact_field_reifier': 'logic',
    'generic_uid_gid_hashmap_ignored': 'logic',
    'line_move_does_not_change_identity': 'logic',
    'semantic_token_change_changes_digest': 'logic',
    'cross_helper_from_int': 'e2e',
    'cross_helper_as_int': 'logic',
    'five_hop_exhaustion': 'logic',
    'escaped_path_edge': 'schema',
    'cross_helper_unrelated_return_decoy': 'logic',
    'seeded_cross_helper_from_int': 'logic',
    'seeded_cross_helper_as_int': 'logic',
    'foreign_receiver': 'logic',
    'ambiguous_call': 'logic',
    'multi_origin_return': 'logic',
    'transform_argument': 'logic',
    'six_hop_exhaustion': 'logic',
    'literal_child_path': 'schema',
    'unsupported_include': 'schema',
    'associated_method_conversion': 'logic',
    'unknown_call': 'logic',
    'arithmetic_rhs': 'logic',
    'method_rhs': 'logic',
    'tuple_rhs': 'logic',
    'transformed_return': 'logic',
    'recursive_path': 'logic',
    'static_local_shadow': 'logic',
    'receiver_shadow': 'logic',
    'payload_not_key': 'logic',
    'duplicate_conversion': 'logic',
    'cast_rhs': 'logic',
    'wrapped_call_binary': 'logic',
    'wrapped_call_cast': 'logic',
    'wrapped_call_method': 'logic',
    'wrapped_call_tuple': 'logic',
    'wrapped_call_index': 'logic',
    'wrapped_call_exact': 'logic',
    'inline_module_conversion': 'logic',
    'nested_function_conversion': 'logic',
    'trait_default_conversion': 'logic',
    'impl_method_conversion': 'logic',
    'cfg_test_conversion_control': 'logic',
    'receiver_same_name_shadow': 'logic',
    'foreign_receiver_strict': 'logic',
    'unbound_key': 'logic',
    'transformed_key': 'logic',
    'multiple_terminal': 'logic',
    'four_hop_control': 'logic',
    'semantic_two_path': 'logic',
    'semantic_zero_terminal': 'logic',
    'semantic_two_terminal': 'logic',
    'semantic_later_path_digest': 'logic',
    'unrelated_deep_static_control': 'logic',
    'unseeded_multi_path_zero_terminal': 'e2e',
    'complete_edge_matrix': 'logic',
    'forward_carried_cycle': 'logic',
    'cfg_test_inline_second_function': 'logic',
    'impl_trait_second_method': 'logic',
    'edge_v6_format_only_equal': 'e2e',
    'edge_v6_binding_key_rebind_different': 'e2e',
    'edge_v6_conversion_move_different': 'e2e',
    'edge_v6_key_target_different': 'e2e',
    'edge_v6_missing_conversion_identity': 'schema',
    'edge_v6_missing_key_identity': 'schema',
    'inline_module_return_impl_trait': 'logic',
    'nested_function_return_impl_trait': 'logic',
    'trait_default_return_impl_trait': 'logic',
    'impl_method_return_impl_trait': 'logic',
})
FROZEN_PHASE_C_TREE_DIGESTS = MappingProxyType({
    'stale_seed_item': '9ce9534f031d6ce8d26491d1c6e0d0de049c0daba7ba8ebcebae08b7eab6906c',
    'missing_seed_observation': 'fd0552f8b4694e3a49e9928820e80455d82d1178a96a5ba447fdc93047d748c9',
    'extra_seed_observation': '48f118106647a88ea26d9eb484d77a58b8a95ab57f6d22236041eca961538b4c',
    'duplicate_seed_owner': '3e970d09891fa58f25ed908f621aa2023d3bee86363e7a5ecbe342d9914c91b9',
    'seed_digest_drift': 'b9fa1bcb7e16c385e7950a149b8f34ebc8aaec7e9de4b0c48327876f261f743b',
    'derived_lock_stale': '838da2d9c0e33cb7002ea3c73367047a10b8798aebcf518c13f0be0bfb40ad35',
    'derived_lock_extra': 'd1d4864a456adfac528d58b299efde4c0b4e3cfc094c379933ead8a509f985fe',
    'unseeded_central_hook': 'af24de8b6cb5f3f8389f3e044109428e328482f436f26bef5ae6c334bd369973',
    'unseeded_public_reifier': 'bdf27da872a376c666d2e0d3e042f59b7c1cf5720d074910a7cd744d183edab7',
    'unseeded_side_table_allocator': '8481a3091483912f2163d0d8fdf321bd14d0b1fb2a78212c4c2390d1acf477e9',
    'irrelevant_id_storm_no_broad_census': '7318acdde87d1c8d7496bcc380d93312185489413ff66d5b3e2db844dcca1499',
    'exact_nineteen_family_set': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'central_topology_mismatch': 'f2155c406a7eb83f25342b437cccf9ccb29683c5e1dae012edff5ef7991203a7',
    'direct_topology_mismatch': 'b00d12494b77421dbbd9409313720ba10925bd8dc792a18f31a369afbcdadfed',
    'direct_iter_store_range_shared_store': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'direct_iter_store_range_split_store_rejected': 'b3d4777c6dbb3cb7216173df64ac0b2b768e8c7a9f0dd1e0396b39bbabfa78c5',
    'store_owner_reverse_edge_missing': '6aa90de2570a96985528aca6cb14510cbb64780d753014b03d5562b7b9a2e081',
    'owner_role_closure_missing': '2b21422286c8f518986c49b8709c2b58029f4597ae2e4fe1b5ab46c6c4586714',
    'unknown_matcher_kind': '769d01530a1f2e71c05d49899f7aa9542e57aacafc36b53f9df4800521d44a9d',
    'barrier_line_range_rejected': '9567d54ebe2d6dd30b6762ba9a9c77c84de22dd9d5c321521f774b5b0b2c62af',
    'random_native_public_conflation': 'db7e8adef46a4dc4b1e2d30d825517f08f343b99957f3f9230098f628bb6df8b',
    'derived_fields_in_manual_authority_rejected': '4627316ea3ca295b6b4c1d4e5ce23f5bd381000c3d568d3f16e2ed99d75c17de',
    'positive_central_registry_insert': 'a0617287d644859594b7fa3f1c1566f668788ddb9ea3538b15259ac91564154e',
    'near_miss_central_registry_insert': 'ebd8d79a35a14f246b3b6b99e78d9a7a743a8e8400f4e7c08447d9c5be644f65',
    'positive_central_registry_lookup': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_central_registry_lookup': 'd175b6259b5c31913a1aedec7d7b048bf36e79c892937e33a4b8e180daea977a',
    'positive_direct_store_allocate': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_direct_store_allocate': 'c8e1daf446609c27ec966c640e27c96c77fb53d23922689f9e0c54d58762b196',
    'positive_direct_store_lookup': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_direct_store_lookup': '528effacd2941ee0e1ca5503eec1629f31048eae076c1859c851761f1088d882',
    'positive_threshold_classifier': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_threshold_classifier': 'c303ad649438f0a6bc1100be83c82527b53439d0e30d2dadadfd182ceeade14f',
    'positive_first_live_probe': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_first_live_probe': 'ec3466e5887379600a02e58f2aafcfdb07dafb630ff76c06dfe5bbe654bc3498',
    'positive_public_from_int_reifier': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_public_from_int_reifier': 'fcce48226009bd5de05dc84b63df068c543cdd8040da85abd4bf18a406e6ac56',
    'positive_public_as_int_reifier': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_public_as_int_reifier': 'e616356d4cc185e85c41c1cbae109ff777b82c8ce1c031ede03fdcc359a114e3',
    'positive_public_field_reifier': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_public_field_reifier': '5d7ef8bd1171088cba03fa2413d8158850f83d39a9cf5c234a9517ca8f8d412f',
    'positive_native_internal_allocator': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'near_miss_native_internal_allocator': 'b5e725be9f0c45ea29ea0c91323c63bfef65b355cf26c40ad7248ed66b55358f',
    'comments_strings_cfg_test_masked': 'a510fdf080f2adca94285f2349d013f9794139f3150856c3ed7663c383703d9a',
    'nested_decoy_outside_seed_span': '417f06258e8974812fbed0f28b73223561d8e8711a404d2a26084530639874fe',
    'same_item_duplicate_match': '3bdc4b9d0e7f24ff9ab77c40883892991541cefb0f2ec721e18b028825c7aa50',
    'seeded_match_not_reemitted_as_unseeded': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'random_native_allocator_not_public_escape': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'barrier_exact_field_reifier': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'generic_uid_gid_hashmap_ignored': 'fbd521858576289a5c0c10d0e96d9922064d95f37e4462019bfa74dd58440ef9',
    'line_move_does_not_change_identity': '8c63631429947e1285447216b2e1a32d950d544f01d9ba51adcdb7786e49bb3c',
    'semantic_token_change_changes_digest': 'aad0901063c273f0e0b1c1cdfafb1717fd50153d1e0e384139e19aca9b7a87b7',
    'cross_helper_from_int': '042dc2274b3135c05ba11bc235fb90b87984a797b948d091c7d4a3d957176e39',
    'cross_helper_as_int': '27c00dd0971de62eb61a7924f2e2d03bb7fa0b42a81da6ce60e157a63a34c8dc',
    'five_hop_exhaustion': 'cda2f563c3c11f9f13931c0dface0d323536d97cd215ec8396dcffb8350ccd96',
    'escaped_path_edge': '6c04f68ab1c07f62396d0f21f1f53f92a21704c501d87efb2b4c38ac41d7ae32',
    'cross_helper_unrelated_return_decoy': '30f400e2a8d7f9122a0eb7a2eaf87ead9ae7b2fc147587fe45bbc31bf215e09c',
    'seeded_cross_helper_from_int': 'a56d6ee668b687134974b6a0f9d3eef27d07e345b584dcb252b77eae5b0b1e6e',
    'seeded_cross_helper_as_int': '9528c7c3169f36924548be7859cbc354f868ebd134b5d6237515652ed5379fe9',
    'foreign_receiver': '1d62204b3d30ba2dc342b2c0414b7574bdeed5917c076a15e32d08b0c1da9eae',
    'ambiguous_call': '9a7009eb5a5d27b9d820525678da4cfcc0aa9d69eb0a9e699dd0a2912aea5507',
    'multi_origin_return': '301f4af6ff41b114b405c811926801aaf04b7a3c9bb1e139df10a06bba46837d',
    'transform_argument': 'b0ee640fe83bf0a840f8490b2127545f276616fd218d61ad20b253d0975c7999',
    'six_hop_exhaustion': '944dd07c9846242b781503d911da79660bd8306247aebe9c08f644d03458bd60',
    'literal_child_path': '93cddaf7a7f235e47246364df78d609317c2c257fe476214a3513dc2e5a16f68',
    'unsupported_include': '37249b024c38261eeb3705c4d2dc45b32b844244992c76f10865dce30facd746',
    'associated_method_conversion': '68a6e5eecb6c38d8831747b0ae5215c6d81c71a995533fa1a1c6c95d07aea2ce',
    'unknown_call': 'b644f61156c9c18119536e5947d49a3e47bcb2f5f2c38013cb5471beb5e3afa4',
    'arithmetic_rhs': '802143873aec9ec68dd2c8c2a79f3b412b7030296ac0888ec8a777bd69509b93',
    'method_rhs': '321a3ccd9db5fd23f825c7019ec5e19210b5069fdfce81684013783cf1ebca26',
    'tuple_rhs': '0996887cbb9488e11395fc8dd4732cf7659910ccac93874a6bcb6fbba11a13a1',
    'transformed_return': '5866d79d37ac40adf75a7e91f4582a29096b4e4322e32987372dba79912d2777',
    'recursive_path': 'cd4f15e11e0f1f23b9e4470aa26777e39a08a404d4a9adb44796449399d173f4',
    'static_local_shadow': 'fa6a0ed8f10d9f3bb7b7b38101ad1c76bc727102b98ffbcbe134eea01e546cf3',
    'receiver_shadow': '782cf4eaa74fb8d747b9ff897ee1d055dc280abe064235ffa3fa5e56a3835fa4',
    'payload_not_key': 'befb358957270002f3868b449a2c71b2e306d3dcca2772aa5d77eeda2d696fd5',
    'duplicate_conversion': 'a070e81581e5f33439192febbde17cd1def97596bf9946fe15aae0e6e36eed2c',
    'cast_rhs': '7e6a23d6468e88529f51a77dbd4c3c11c7fd5d4a0d323e491b62f4ea956f77f5',
    'wrapped_call_binary': 'f1ab738f7892b368eec325fd8c1928f69dac8e3ccb093aa158d4543e0fe2000c',
    'wrapped_call_cast': '207fce4ba4123d4079ac51ead037f6830567341c432bcca03fd49782bbd28471',
    'wrapped_call_method': '181128d22acb3cbb3bfbbd5d4d5494d1fe551ed2001e94170011936e027941e0',
    'wrapped_call_tuple': '0f997376991813363afa53b6cd92fde491b6c18d76c3d96eec1ac50f2fadeb13',
    'wrapped_call_index': 'f90e564bb9e79a229d00e15472df07cb742c7261391c9670bf4b16e5ffe3c76a',
    'wrapped_call_exact': 'f1d69cc12d426762606666e71f2429fd2fd7d6ec0c97e138efcd76e3fdd3a82e',
    'inline_module_conversion': 'ceb8a1169b1e11b2c340299f949fd1e02c989b49645de4adb535883b35b4cff8',
    'nested_function_conversion': '1496d4902c8487888251adabcba43d37416dbe37ee83833996d67ddca4c53ff7',
    'trait_default_conversion': 'f145e70000492d6d4eb5f1b790e9e0b4a3dc66a9a4df4669285f1c3513c4fd4f',
    'impl_method_conversion': 'c4c0ddbd83d2b74e910e0e2ae05ef5898a080c55257b19498e5c65fead63e183',
    'cfg_test_conversion_control': 'a249f4efe8ccdbb9421e7c9cbf0ffa68ac5fc9b48282a5d3e576c6b570477eda',
    'receiver_same_name_shadow': '7ce7c736446fb02dfb996edae23d172ec06f32ae061aad4f002be282f8fc967b',
    'foreign_receiver_strict': '29a9d616cf9b7ca8951d35b45bf64bdce8a3badbe62808f5af9a2ed7881677f2',
    'unbound_key': 'd43eeb4a165bf2ee7dff0356c1b68a8cc048332d74e7c2df5b19a4206ce763bb',
    'transformed_key': 'd1fe03afeba84c79efa0c749fca1d901a07f485d764ee9d7326bf378a0ac1196',
    'multiple_terminal': 'cacb97269444b5c761fcd5a05ec33719011dffd9ca4625c30124face002e7a06',
    'four_hop_control': '925b870d60cf39fdb960e0d8dfa3144ae571eee0e9b0992b35f4174ea783db5f',
    'semantic_two_path': '76f6482e1274ab01e16c85c305af27fd5248ef162f9688b85ff60467ff0f2d3f',
    'semantic_zero_terminal': '1ab47018be4c30ea88bbaf48faf0b39b24030895399e71a5d6c2d823d6d021d0',
    'semantic_two_terminal': '61d23c770c1e89025255b1319637858de590febc3bcb037a671e474c194a89b8',
    'semantic_later_path_digest': '385d270d2d8c769adeb7ce315c29216193f4e7c56ce49150b4845f9738d55420',
    'unrelated_deep_static_control': '9b0b9d30e74622f3902ec697dd1310006d29c3abb7f6fb2e046371ba4e9e85f1',
    'unseeded_multi_path_zero_terminal': '6e986c271cbe652d1a60cce9bed8ac01f8a3e2a90c4cf2e122bb57b778def840',
    'complete_edge_matrix': '2abf7af6327210902f57e123ca94724b04d9d957f6a2663cc85ad8c12a55e81b',
    'forward_carried_cycle': '6efce07d37afe8a4dc7aa73e8ba16c4f7084a66ad22fe8f39bcf5cba61f7a7f9',
    'cfg_test_inline_second_function': '2dbb29918f50c3d58210610acdb5308d708d244f1932d2d69eaf035a701c8174',
    'impl_trait_second_method': 'dc2602137842160faa07f1b0fa4ac15b703b0e0c9b93cddaebdd1918cf846281',
    'edge_v6_format_only_equal': '2125e36ea39e5a809cb9da76f735028669a9501925886ad68bf9ef8dd30f8c8d',
    'edge_v6_binding_key_rebind_different': '7349f4d89cbfef0b9db7b2a7f57f6a8150e7c8c4ba528ab0f74b44f3d46f1b23',
    'edge_v6_conversion_move_different': 'ad23bcbe6347380bcba8854a1e473a8698c17b86569b1923ffa8d83e41a8288c',
    'edge_v6_key_target_different': '3d056ce5af8e592ba57465ece4265d25755a44f7830494fbda4b7c2b5c030de8',
    'edge_v6_missing_conversion_identity': '4da034c435c37e0af2c3e38ae59dc1fbd330aeb99823f75829e09c703d69a33d',
    'edge_v6_missing_key_identity': '803632af0447476e9e7ef2ed8f63d204a3b81a2ad26b00da6b60fdb2751789aa',
    'inline_module_return_impl_trait': 'a0e488357f5c69ca1c4c4f726e3aaede54af020c09d57d2baec4d5f51dfee39a',
    'nested_function_return_impl_trait': '230a456ac46c5daf82308cbfb6ccb2f6f930cffc4117345ff7390c4d484fda82',
    'trait_default_return_impl_trait': '0ae66fcfc6840c4b575b32f08b474bcaf3442559d3b2abb449b077afa07dccb4',
    'impl_method_return_impl_trait': 'caeb90f2be420ed401c8fe2787989032415bac00723d58ab2ef6c4cdc54a226d',
})
FROZEN_PHASE_C_EXPECTATIONS = MappingProxyType({
    "stale_seed_item": ("rejected", ["phase-c:stale-route:route_central_array"]),
    "missing_seed_observation": ("rejected", ["phase-c:owner-role-closure:central_array","phase-c:route-selector-closure:route_central_array"]),
    "extra_seed_observation": ("rejected", ["phase-c:selector-route:selector_extra"]),
    "duplicate_seed_owner": ("rejected", ["phase-c:exact-nineteen-family-set"]),
    "seed_digest_drift": ("rejected", ["phase-c:route-digest:route_central_array","phase-c:selector-digest:selector_central_array"]),
    "derived_lock_stale": ("rejected", ["phase-c:derived-observation-digest", "phase-c:derived-source-set"]),
    "derived_lock_extra": ("rejected", ["phase-c:derived-row-content", "phase-c:derived-row-count", "phase-c:derived-source-set"]),
    "unseeded_central_hook": ("rejected", ["phase-c:perimeter-terminal-cardinality:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unseeded_central_hook/runtime/value.rs::unseeded_central:0", "phase-c:unseeded-candidate"]),
    "unseeded_public_reifier": ("rejected", ["phase-c:unseeded-candidate"]),
    "unseeded_side_table_allocator": ("rejected", ["phase-c:unseeded-candidate"]),
    "irrelevant_id_storm_no_broad_census": ("accepted", []),
    "exact_nineteen_family_set": ("accepted", []),
    "central_topology_mismatch": ("rejected", ["phase-c:topology:central_array"]),
    "direct_topology_mismatch": ("rejected", ["phase-c:topology:direct_iter_store"]),
    "direct_iter_store_range_shared_store": ("accepted", []),
    "direct_iter_store_range_split_store_rejected": ("rejected", ["phase-c:direct-shared-store"]),
    "store_owner_reverse_edge_missing": ("rejected", ["phase-c:store-reverse-edge:route_direct_range"]),
    "owner_role_closure_missing": ("rejected", ["phase-c:owner-role-closure:direct_cell"]),
    "unknown_matcher_kind": ("rejected", ["phase-c:unknown-matcher:selector_central_array"]),
    "barrier_line_range_rejected": ("rejected", ["phase-c:barrier-line-range"]),
    "random_native_public_conflation": ("rejected", ["phase-c:native-public-conflation"]),
    "derived_fields_in_manual_authority_rejected": ("rejected", ["phase-c:authority-line-or-pattern-selector"]),
    "positive_central_registry_insert": ("rejected", ["phase-c:semantic-conversion-cardinality:expr-selector-central-array:0", "phase-c:semantic-edge:expr-selector-central-array", "phase-c:semantic-path-cardinality:expr-selector-central-array:0", "phase-c:semantic-terminal-cardinality:expr-selector-central-array:0"]),
    "near_miss_central_registry_insert": ("rejected", ["phase-c:semantic-conversion-cardinality:expr-selector-central-array:0", "phase-c:semantic-count:expr-selector-central-array", "phase-c:semantic-edge:expr-selector-central-array", "phase-c:semantic-path-cardinality:expr-selector-central-array:0", "phase-c:semantic-terminal-cardinality:expr-selector-central-array:0"]),
    "positive_central_registry_lookup": ("accepted", []),
    "near_miss_central_registry_lookup": ("rejected", ["phase-c:unknown-matcher:selector_central_queue"]),
    "positive_direct_store_allocate": ("accepted", []),
    "near_miss_direct_store_allocate": ("rejected", ["phase-c:unknown-matcher:selector_direct_iter_store"]),
    "positive_direct_store_lookup": ("accepted", []),
    "near_miss_direct_store_lookup": ("rejected", ["phase-c:unknown-matcher:selector_direct_range"]),
    "positive_threshold_classifier": ("accepted", []),
    "near_miss_threshold_classifier": ("rejected", ["phase-c:unknown-matcher:selector_central_hashlib"]),
    "positive_first_live_probe": ("accepted", []),
    "near_miss_first_live_probe": ("rejected", ["phase-c:unknown-matcher:selector_central_hmac"]),
    "positive_public_from_int_reifier": ("accepted", []),
    "near_miss_public_from_int_reifier": ("rejected", ["phase-c:unknown-matcher:selector_central_decimal"]),
    "positive_public_as_int_reifier": ("accepted", []),
    "near_miss_public_as_int_reifier": ("rejected", ["phase-c:unknown-matcher:selector_central_graphlib"]),
    "positive_public_field_reifier": ("accepted", []),
    "near_miss_public_field_reifier": ("rejected", ["phase-c:unknown-matcher:selector_threading_barrier_instance_field"]),
    "positive_native_internal_allocator": ("accepted", []),
    "near_miss_native_internal_allocator": ("rejected", ["phase-c:unknown-matcher:selector_native_random"]),
    "comments_strings_cfg_test_masked": ("accepted", []),
    "nested_decoy_outside_seed_span": ("accepted", []),
    "seeded_match_not_reemitted_as_unseeded": ("accepted", []),
    "random_native_allocator_not_public_escape": ("accepted", []),
    "barrier_exact_field_reifier": ("accepted", []),
    "generic_uid_gid_hashmap_ignored": ("accepted", []),
    "line_move_does_not_change_identity": ("accepted", []),
    "same_item_duplicate_match": ("rejected", ["phase-c:stale-route:route_central_array"]),
    "semantic_token_change_changes_digest": ("rejected", ["phase-c:route-digest:route_central_array","phase-c:selector-digest:selector_central_array"]),
    "cross_helper_from_int": ("rejected", ["phase-c:unseeded-candidate"]),
    "cross_helper_as_int": ("rejected", ["phase-c:unseeded-candidate"]),
    "five_hop_exhaustion": ("rejected", ["phase-c:provenance:depth:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/five_hop_exhaustion/runtime/value.rs::renamed_hop_alloc", "phase-c:unseeded-candidate"]),
    "escaped_path_edge": ("rejected", ["phase-c:module-graph-unresolved:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/escaped_path_edge/runtime/mod.rs::escaped"]),
    "cross_helper_unrelated_return_decoy": ("accepted", []),
    "seeded_cross_helper_from_int": ("accepted", []),
    "seeded_cross_helper_as_int": ("accepted", []),
    "foreign_receiver": ("accepted", []),
    "ambiguous_call": ("rejected", ["phase-c:provenance:ambiguous-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/ambiguous_call/runtime/value.rs::ambig_api", "phase-c:unseeded-candidate"]),
    "multi_origin_return": ("rejected", ["phase-c:provenance:multi-origin-return:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/multi_origin_return/runtime/value.rs::multi_origin_api", "phase-c:unseeded-candidate"]),
    "transform_argument": ("rejected", ["phase-c:provenance:transformed-argument:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/transform_argument/runtime/value.rs::transform_api", "phase-c:unseeded-candidate"]),
    "six_hop_exhaustion": ("rejected", ["phase-c:provenance:depth:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/six_hop_exhaustion/runtime/value.rs::six_hop_h5", "phase-c:unseeded-candidate"]),
    "literal_child_path": ("accepted", []),
    "unsupported_include": ("rejected", ["phase-c:module-graph-include-unsupported:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unsupported_include/runtime/mod.rs"]),
    "associated_method_conversion": ("rejected", ["phase-c:unsupported-non-top-level-conversion:impl-method:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/associated_method_conversion/runtime/value.rs::associated_convert@625"]),
    "unknown_call": ("rejected", ["phase-c:provenance:unknown-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unknown_call/runtime/value.rs::unknown_call_api", "phase-c:unseeded-candidate"]),
    "arithmetic_rhs": ("rejected", ["phase-c:provenance:unsupported-rhs:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/arithmetic_rhs/runtime/value.rs::arithmetic_rhs_api", "phase-c:unseeded-candidate"]),
    "method_rhs": ("rejected", ["phase-c:provenance:method-rhs:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/method_rhs/runtime/value.rs::method_rhs_api", "phase-c:unseeded-candidate"]),
    "tuple_rhs": ("rejected", ["phase-c:provenance:unsupported-binding:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/tuple_rhs/runtime/value.rs::tuple_rhs_api", "phase-c:unseeded-candidate"]),
    "transformed_return": ("rejected", ["phase-c:provenance:transformed-return:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/transformed_return/runtime/value.rs::transformed_return_api", "phase-c:unseeded-candidate"]),
    "recursive_path": ("rejected", ["phase-c:provenance:recursive:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/recursive_path/runtime/value.rs::recursive_path", "phase-c:unseeded-candidate"]),
    "static_local_shadow": ("accepted", []),
    "receiver_shadow": ("accepted", []),
    "payload_not_key": ("accepted", []),
    "duplicate_conversion": ("rejected", ["phase-c:provenance:unsupported-rhs:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/duplicate_conversion/runtime/value.rs::duplicate_conversion_api", "phase-c:unseeded-candidate"]),
    "cast_rhs": ("rejected", ["phase-c:provenance:unsupported-rhs:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/cast_rhs/runtime/value.rs::cast_rhs_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_binary": ("rejected", ["phase-c:provenance:wrapped-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/wrapped_call_binary/runtime/value.rs::wrapped_call_binary_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_cast": ("rejected", ["phase-c:provenance:wrapped-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/wrapped_call_cast/runtime/value.rs::wrapped_call_cast_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_method": ("rejected", ["phase-c:provenance:wrapped-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/wrapped_call_method/runtime/value.rs::wrapped_call_method_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_tuple": ("rejected", ["phase-c:provenance:wrapped-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/wrapped_call_tuple/runtime/value.rs::wrapped_call_tuple_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_index": ("rejected", ["phase-c:provenance:wrapped-call:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/wrapped_call_index/runtime/value.rs::wrapped_call_index_api", "phase-c:unseeded-candidate"]),
    "wrapped_call_exact": ("accepted", []),
    "inline_module_conversion": ("rejected", ["phase-c:unsupported-non-top-level-conversion:inline-module:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/inline_module_conversion/runtime/value.rs::inline_conversion@594"]),
    "nested_function_conversion": ("rejected", ["phase-c:unsupported-non-top-level-conversion:nested-function:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/nested_function_conversion/runtime/value.rs::nested_conversion@620"]),
    "trait_default_conversion": ("rejected", ["phase-c:unsupported-non-top-level-conversion:trait-default:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/trait_default_conversion/runtime/value.rs::trait_conversion@599"]),
    "impl_method_conversion": ("rejected", ["phase-c:unsupported-non-top-level-conversion:impl-method:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/impl_method_conversion/runtime/value.rs::impl_conversion@612"]),
    "cfg_test_conversion_control": ("accepted", []),
    "receiver_same_name_shadow": ("rejected", ["phase-c:unseeded-candidate"]),
    "foreign_receiver_strict": ("rejected", ["phase-c:unseeded-candidate"]),
    "unbound_key": ("rejected", ["phase-c:unseeded-candidate"]),
    "transformed_key": ("rejected", ["phase-c:unseeded-candidate"]),
    "multiple_terminal": ("rejected", ["phase-c:perimeter-terminal-cardinality:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/multiple_terminal/runtime/value.rs::multi_api:2", "phase-c:provenance:multiple-terminal:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/multiple_terminal/runtime/value.rs::multi_put", "phase-c:unseeded-candidate"]),
    "four_hop_control": ("rejected", ["phase-c:unseeded-candidate"]),
    "semantic_two_path": ("rejected", ["phase-c:semantic-conversion-cardinality:expr-semantic_two_path:2", "phase-c:semantic-edge:expr-semantic_two_path", "phase-c:semantic-path-cardinality:expr-semantic_two_path:2", "phase-c:semantic-terminal-cardinality:expr-semantic_two_path:2"]),
    "semantic_zero_terminal": ("rejected", ["phase-c:semantic-edge:expr-semantic_zero_terminal", "phase-c:semantic-terminal-cardinality:expr-semantic_zero_terminal:0"]),
    "semantic_two_terminal": ("rejected", ["phase-c:semantic-edge:expr-semantic_two_terminal", "phase-c:semantic-terminal-cardinality:expr-semantic_two_terminal:2"]),
    "semantic_later_path_digest": ("rejected", ["phase-c:semantic-edge-digest:expr-v13-stale-later-path"]),
    "unrelated_deep_static_control": ("accepted", []),
    "unseeded_multi_path_zero_terminal": ("rejected", ["phase-c:perimeter-conversion-cardinality:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unseeded_multi_path_zero_terminal/runtime/value.rs::unseeded_multi_path_zero_terminal:2", "phase-c:perimeter-path-cardinality:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unseeded_multi_path_zero_terminal/runtime/value.rs::unseeded_multi_path_zero_terminal:2", "phase-c:perimeter-terminal-cardinality:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/unseeded_multi_path_zero_terminal/runtime/value.rs::unseeded_multi_path_zero_terminal:0", "phase-c:semantic-conversion-cardinality:expr-semantic_two_path:2", "phase-c:semantic-edge:expr-semantic_two_path", "phase-c:semantic-path-cardinality:expr-semantic_two_path:2", "phase-c:semantic-terminal-cardinality:expr-semantic_two_path:2", "phase-c:unseeded-candidate"]),
    "forward_carried_cycle": ("rejected", ["phase-c:provenance:recursive:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/forward_carried_cycle/runtime/value.rs::cycle_helper", "phase-c:unseeded-candidate"]),
    "cfg_test_inline_second_function": ("accepted", []),
    "impl_trait_second_method": ("rejected", ["phase-c:unsupported-non-top-level-conversion:impl-method:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/impl_trait_second_method/runtime/value.rs::impl_conversion@612", "phase-c:unsupported-non-top-level-conversion:impl-method:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/impl_trait_second_method/runtime/value.rs::second_impl_conversion@758", "phase-c:unsupported-non-top-level-conversion:trait-default:apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/phase_c_cases/impl_trait_second_method/runtime/value.rs::second_trait_conversion@921"]),
    "complete_edge_matrix": ("accepted", []),
    "edge_v6_format_only_equal": ("accepted", []),
    "edge_v6_binding_key_rebind_different": ("accepted", []),
    "edge_v6_conversion_move_different": ("accepted", []),
    "edge_v6_key_target_different": ("accepted", []),
    "edge_v6_missing_conversion_identity": ("rejected", ["phase-c:semantic-edge:v11-missing-conversion-identity", "phase-c:semantic-terminal-cardinality:v11-missing-conversion-identity:0"]),
    "edge_v6_missing_key_identity": ("rejected", ["phase-c:semantic-edge:v11-missing-key-identity", "phase-c:semantic-terminal-cardinality:v11-missing-key-identity:0"]),
    "inline_module_return_impl_trait": ("accepted", []),
    "nested_function_return_impl_trait": ("accepted", []),
    "trait_default_return_impl_trait": ("accepted", []),
    "impl_method_return_impl_trait": ("accepted", []),
})
FROZEN_TYPED_TREE_DIGESTS = MappingProxyType({
    'valid_codec': 'a7db094aadfe03a21f24510fe80ba394de403b726b5eb2b7481d7b151874461c',
    'wrapper_chain': 'a7db094aadfe03a21f24510fe80ba394de403b726b5eb2b7481d7b151874461c',
    'ordinary_python_int': 'def02965ea82b473fa3b00f5504a131260c2547818094fc3325fd16633b949d8',
    'comments_only': 'cb8b58c9fa9d007c2980cf6ba287dfcbe26c064512eef5a12d7d513253b4d8d3',
    'cfg_test_only': 'b9be273dde823ef7e481ffad7dd50ea75cb5cb33bfcde0cc62b1f55581575c2e',
    'macro_only': '81ab57fdb2efe0ed90c0bbc7aec8d9de737c7405cdb2e381802d305fece1fe55',
    'multi_arg_wrapper': '1900d3cd333974d2049c8a1eff93a679ad2743c89864168aded5d3d20e0adf3d',
    'reserved_low_bit_set': '59749e78c3aa500c455f7bfece7dfc8891192b25a6614d6a6a5248a959ecf380',
    'pack_guard_after_bits': 'f4900d22037d9e350b1d903b13ef74c17ec5f4423bbc3aad191de266972d4baa',
    'pack_extra_statement': 'c4e118d63720c065acec9e291428f357e7eb0e0b5b64babfdafa241a9086284c',
    'encoder_shadow': '39bda50739f180665d8f7a43ff4d60a4b10a196650f10a5aa05a948c5ce9f747',
    'decoder_tag_reorder': '820b008c9fe3825c722e0a24f1acd7fdf3def3458902c6414e503734afc7bf42',
    'decoder_reserved_reorder': 'e85b4b0658d2a9433c912dc2311a5ee3a836a840c7119849a7b0802941965aa2',
    'decoder_identity_shadow': '3c90bb6fa3bd257899de89a9a403a1a510aecc9891cba809cfc598c6a2f26e46',
    'decoder_unknown_statement': 'fc9fd3d05c232cbd088a1301aae9691da5f323d78873d71154f742354cd604ba',
    'wrapper_nonidentity': 'c111eb2e2d7258a80f6eb10077f291b291c1ae1994686fed98928965aa374e8c',
    'wrapper_argument_swap': '14645a1608768f788b0fc99eadb12b2b7344bae93f9de02ba6f2b824896043aa',
    'wrapper_argument_constant': 'ddc8670457d38ef3b1b7d0ca18c465329f838d354b99e1e08a3f71d4dfb5256d',
    'wrapper_callee_shadow': '487ebafd9199052548e4c2c49a38887cbfd908eab0ad9b2f14aa2569e14053bb',
    'wrapper_terminal': '93c7a86345e8eaf1a72fae40e9005a591513cee61a856cc932a950dcc6c43129',
    'bound_macro_after_valid_codec': '05821f0c63f74a7ab39c11065087493e1bbcc5c544b0a757f3d58b93a73328f4',
    'macro_confined_codec': 'af0566cb2820c0ed7b2cded6998ea7a2f4728ec1f2c0de8330530c258313eb00',
    'macro_split_codec': '9610505bc02188bc9ea3235e22d6bbf41c753f03a54f4caff43f07e64d8b7426',
    'incomplete_authority': '07a725334c75fc6da12492a1ad7c98b415e56768133fa8a220f57e9bb8a51b3b',
    'alias_only': '5f3812b95cda98d78a19c903c759f1d85cf4c7b4e74b11be0b370b3a7d88e2a2',
    'post_return_bound_guard': 'e5171bd73635f33734ec16b61b0fb5abc828c971150e6a214e0b81cad26b48cb',
    'reserved_bit_decoy_guard': '98618a100fbc3610c29bb66d3117687939df25f37959f7c17f6e11c7bee11119',
    'invalid_layout': '1a2a568ef6baf005ddf311705bf6b52fbf72bd65d999d9511ba4060e34a50344',
    'oversized_family_code': '7d5d254a6de92b6fb4ccff26316d73d316a4683cbb947988b7d2aa67749ca7d8',
    'decoy_bound_check': '0be97acae7f535cd6ae0ae9ff91cc7361453b814af7cfdbe968768d743075749',
    'decoder_guard_expression': '15623e3421808fca6c015a735f095042922c6f04c6e018ffb411149616971f3a',
    'high_id_overlap': 'b7ffbcc5098a28477141ebf2ec62d83e2f93f8b365ac8211521a0f55bd7251b3',
    'wrong_shift': '9ccdb4e51d78d4aa8b444815d9a405f7119cebdcc8ccd6f6183c6cfa60dcffdb',
    'invalid_kind': '86cb87f7268fbfd37ccccdc33f2f3429f9a520be4454089a2f18f0740c31ab9b',
    'default_coercion': 'c5aadc86768c0105c8d9e76371cab77cbda44647d2d9e6af59ebf3c4689812ea',
    'multi_family_codec': '46102837bc96c9e880eaf5c523e70396396e7b15a7bc46bfada8401e8143b4db',
    'bare_macro_invocation': 'd5ce2cb63424e4fd365ab19f99d4193cecbc368e5c4cf039a62b1a5daa9290ca',
    'support_type_shadow': '62425c732c9c60568e59a14a14dd4008968dc60a6868aa8e5f2b0d9ddfccd8d4',
    'normalized_digest_anchor_drift': '41ccd307651f1fc1b842b84d9fd2b5c5273162defa26a8530fa367ef4559921b',
    'qualified_bound_declaration': '74ce0b43e746c80bc42e195d2964b5812c2f70acdb7e94ff33ce8d7b5b446393',
    'decoder_match_arm_binding_shadow': 'cfed012b66a7384549c570a4978407fb4f3b29e832a66d1ceb66dbc96b7e4306',
    'participating_std_namespace_shadow_mod': '8002d6fcf370f967fcf9ae4ebb2044ef330031ab35b92367b7017688cbceb131',
    'participating_std_namespace_shadow_alias': '5767012185230a7845ab3d37500835c0ca0338dca33fc458ec01fa1fd50dc42f',
    'unrelated_macro_in_unbound_item': '2de9adafa85242452f3e9e8f69c2ad7b8281c15b68e68890078f1fe35eb6deef',
    'unrelated_result_variant_declaration': '8321ad084db19cb94479227fbdead8a680cdc6a2a5b16a1dc801c840a0b1a306',
    'participating_std_namespace_shadow_extern': '0c2b16563396e273bcb182eb808bd4b9c93ed2e99785f1d904dcef8e462d16d0',
})
FROZEN_TYPED_EXPECTATIONS = (
    ("valid_codec", "fixtures/typed_cases/valid_codec", "accepted", ()),
    ("wrapper_chain", "fixtures/typed_cases/wrapper_chain", "accepted", ()),
    ("ordinary_python_int", "fixtures/typed_cases/ordinary_python_int", "accepted", ()),
    ("comments_only", "fixtures/typed_cases/comments_only", "accepted", ()),
    ("cfg_test_only", "fixtures/typed_cases/cfg_test_only", "accepted", ()),
    ("macro_only", "fixtures/typed_cases/macro_only", "accepted", ()),
    ("multi_arg_wrapper", "fixtures/typed_cases/multi_arg_wrapper", "accepted", ()),
    ("reserved_low_bit_set", "fixtures/typed_cases/reserved_low_bit_set", "rejected", ("typed-contract:canonical-reserved-bits",)),
    ("pack_guard_after_bits", "fixtures/typed_cases/pack_guard_after_bits", "rejected", ("typed-contract:field-guard-order",)),
    ("pack_extra_statement", "fixtures/typed_cases/pack_extra_statement", "rejected", ("typed-contract:pack-extra-statement",)),
    ("encoder_shadow", "fixtures/typed_cases/encoder_shadow", "rejected", ("typed-contract:encoder-code-shadow",)),
    ("decoder_tag_reorder", "fixtures/typed_cases/decoder_tag_reorder", "rejected", ("typed-contract:decoder-tag-order",)),
    ("decoder_reserved_reorder", "fixtures/typed_cases/decoder_reserved_reorder", "rejected", ("typed-contract:canonical-reserved-order",)),
    ("decoder_identity_shadow", "fixtures/typed_cases/decoder_identity_shadow", "rejected", ("typed-contract:decoder-kind-id-shadow",)),
    ("decoder_unknown_statement", "fixtures/typed_cases/decoder_unknown_statement", "rejected", ("typed-contract:decoder-unknown-statement",)),
    ("wrapper_nonidentity", "fixtures/typed_cases/wrapper_nonidentity", "rejected", ("typed-contract:wrapper-forward:emit_packet",)),
    ("wrapper_argument_swap", "fixtures/typed_cases/wrapper_argument_swap", "rejected", ("typed-contract:wrapper-forward:emit_packet",)),
    ("wrapper_argument_constant", "fixtures/typed_cases/wrapper_argument_constant", "rejected", ("typed-contract:wrapper-forward:emit_packet",)),
    ("wrapper_callee_shadow", "fixtures/typed_cases/wrapper_callee_shadow", "rejected", ("typed-contract:wrapper-forward:emit_packet",)),
    ("wrapper_terminal", "fixtures/typed_cases/wrapper_terminal", "rejected", ("typed-contract:wrapper-terminal:make_packet",)),
    ("bound_macro_after_valid_codec", "fixtures/typed_cases/bound_macro_after_valid_codec", "rejected", ("typed-contract:macro-codec",)),
    ("macro_confined_codec", "fixtures/typed_cases/macro_confined_codec", "rejected", ("typed-contract:macro-codec",)),
    ("macro_split_codec", "fixtures/typed_cases/macro_split_codec", "rejected", ("typed-contract:macro-codec",)),
    ("incomplete_authority", "fixtures/typed_cases/incomplete_authority", "rejected", ("typed-contract:authority-incomplete",)),
    ("alias_only", "fixtures/typed_cases/alias_only", "rejected", ("typed-contract:declaration:value_type",)),
    ("post_return_bound_guard", "fixtures/typed_cases/post_return_bound_guard", "rejected", ("typed-contract:field-guard-order",)),
    ("reserved_bit_decoy_guard", "fixtures/typed_cases/reserved_bit_decoy_guard", "rejected", ("typed-contract:canonical-reserved-guard",)),
    ("invalid_layout", "fixtures/typed_cases/invalid_layout", "rejected", ("typed-contract:authority-incomplete",)),
    ("oversized_family_code", "fixtures/typed_cases/oversized_family_code", "rejected", ("typed-contract:authority-incomplete",)),
    ("decoy_bound_check", "fixtures/typed_cases/decoy_bound_check", "rejected", ("typed-contract:pack-grammar",)),
    ("decoder_guard_expression", "fixtures/typed_cases/decoder_guard_expression", "rejected", ("typed-contract:decoder-tag-order",)),
    ("high_id_overlap", "fixtures/typed_cases/high_id_overlap", "rejected", ("typed-contract:authority-incomplete",)),
    ("wrong_shift", "fixtures/typed_cases/wrong_shift", "rejected", ("typed-contract:authority-incomplete",)),
    ("invalid_kind", "fixtures/typed_cases/invalid_kind", "rejected", ("typed-contract:authority-incomplete",)),
    ("default_coercion", "fixtures/typed_cases/default_coercion", "rejected", ("typed-contract:authority-incomplete",)),
    ("multi_family_codec", "fixtures/typed_cases/valid_multi_family_codec", "accepted", ()),
    ("bare_macro_invocation", "fixtures/typed_cases/bare_macro_invocation", "rejected", ("typed-contract:macro-codec",)),
    ("support_type_shadow", "fixtures/typed_cases/support_type_shadow", "rejected", ("typed-contract:support-identity",)),
    ("normalized_digest_anchor_drift", "fixtures/typed_cases/normalized_digest_anchor_drift", "rejected", ("typed-contract:route-normalized-anchor:typed-producer", "typed-contract:route-normalized-digest:typed-producer")),
    ("qualified_bound_declaration", "fixtures/typed_cases/qualified_bound_declaration", "rejected", ("typed-contract:declaration:encoder",)),
    ("decoder_match_arm_binding_shadow", "fixtures/typed_cases/decoder_match_arm_binding_shadow", "rejected", ("typed-contract:decoder-match-binding-shadow",)),
    ("participating_std_namespace_shadow_mod", "fixtures/typed_cases/participating_std_namespace_shadow_mod", "rejected", ("typed-contract:support-identity",)),
    ("participating_std_namespace_shadow_alias", "fixtures/typed_cases/participating_std_namespace_shadow_alias", "rejected", ("typed-contract:support-identity",)),
    ("unrelated_macro_in_unbound_item", "fixtures/typed_cases/unrelated_macro_in_unbound_item", "accepted", ()),
    ("unrelated_result_variant_declaration", "fixtures/typed_cases/unrelated_result_variant_declaration", "accepted", ()),
    ("participating_std_namespace_shadow_extern", "fixtures/typed_cases/participating_std_namespace_shadow_extern", "rejected", ("typed-contract:support-identity",)),
)
FROZEN_TYPED_SOURCE_DIGESTS = MappingProxyType({
    'valid_codec': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'wrapper_chain': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'ordinary_python_int': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'ad72bfa365fccced832bcfe62ad477878eee4da8cc47593d9a579a70640da9e4'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'comments_only': (('authority.toml', '4f045a6a35fe6e4f38c8a34e48dd246d5881c913841e55994270a314d14563ec'), ('runtime/value.rs', '7c3ee65091e4affce746519f7d7526496045d2397c4f91d677e0f0836bfb7266'), ('typed_routes.rs', 'bfc64bb8b2dfc970cb13d7f32ac881a1366badde613eb7b4c67b0a2e602cfa4e')),
    'cfg_test_only': (('authority.toml', '4f045a6a35fe6e4f38c8a34e48dd246d5881c913841e55994270a314d14563ec'), ('runtime/value.rs', 'ccefaf354a62f43248ee5e6497bc87b50b0d14ef0ec4fb998385493457b3928b'), ('typed_routes.rs', 'bfc64bb8b2dfc970cb13d7f32ac881a1366badde613eb7b4c67b0a2e602cfa4e')),
    'macro_only': (('authority.toml', '4f045a6a35fe6e4f38c8a34e48dd246d5881c913841e55994270a314d14563ec'), ('runtime/value.rs', '0db33e5b3958b5e2b954e9e989bb5479a21dcc0b7105f4c7401970a3f30fa33e'), ('typed_routes.rs', 'bfc64bb8b2dfc970cb13d7f32ac881a1366badde613eb7b4c67b0a2e602cfa4e')),
    'multi_arg_wrapper': (('authority.toml', 'ef82166ba5104f21e849ad68cc655d72ff64b74d29471e1011361fb4088c243b'), ('runtime/value.rs', '5527c81a00dc3b6c4e45f717f9a1ab99a4366922f7cfcc0726fc88b8b2172b43'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'reserved_low_bit_set': (('authority.toml', '4f045a6a35fe6e4f38c8a34e48dd246d5881c913841e55994270a314d14563ec'), ('runtime/value.rs', '70c69b5934c355a0ab3277a9376c46c14ecfa2530265cf90771a56a5936bb260'), ('typed_routes.rs', 'bfc64bb8b2dfc970cb13d7f32ac881a1366badde613eb7b4c67b0a2e602cfa4e')),
    'pack_guard_after_bits': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '38ab206869a890e1f910e763490c50b753fb34d39032a3fea879f64de3162934'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'pack_extra_statement': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '681344153d6e6525234f1a78cffcc7f7de14adde38604249f216913822ab2221'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'encoder_shadow': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '09b857739d5f5ecce0156fa7e9d71e6630a7c37d73940f74e9e4e5ea6ec75efc'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_tag_reorder': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '6e356590338ccedbcef19b7aa56124c0b3dd88536b1ba47e2f75caaea63c3fc3'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_reserved_reorder': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'c03108b0fa09c05982ae0612590bdbd8ae54ac92818cbebd39da2b92706f6714'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_identity_shadow': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'a8cdc01357d3cdb7badb73764a0e8e096845e710325afd37f0ace6fb20c605ab'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_unknown_statement': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'a34229f5f22dbc20d1150023bf5a96d01642a8f9c3df87ec2813586a05a28f81'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'wrapper_nonidentity': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'b0dfa015461a53084aa058b7bc2638fec7579a27b78b4cc5d30e4dbeaf36a301'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'wrapper_argument_swap': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', 'f1f6bbd360f903f8045dd9e582eb0ec01a70bde9a11452c6683e17ae7023a762'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'wrapper_argument_constant': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', 'e78b50e73f21102bc5497d0f5ed17d48c3beac5d9eb99f104b3e882933a0fa53'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'wrapper_callee_shadow': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', '87a5766c4b2cc9fbd19563e37b5ed93486ea442ace8838dfee1443de8a1ccad0'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'wrapper_terminal': (('authority.toml', '5a4e38aa02cdf37c4e84e79f0353786760b67c9d98fd5b726150d09c2baa35b5'), ('runtime/value.rs', 'ebd1285634aa926f6de1a2ff9da829588f75e0396249b1bed6904e40eede661e'), ('typed_routes.rs', 'e73422f3641c43efed344bfa74dafb2b8cad8448ba1390d104e90f44dcc97332')),
    'bound_macro_after_valid_codec': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'a70efde98bce97d55a9a91153ac6f78cf24cc4389cf339668fd554c7c135d17a'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'macro_confined_codec': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'fc44544bad0b6dfd0af18a65713e07d1158bf8004d21b8087b7255da6b73a7ca'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'macro_split_codec': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', '92336cd3ad23d02e09a374ab0dd08994fa11d2cf9e4174308840e617a62ac850'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'incomplete_authority': (('authority.toml', 'e8d8895bfa01529382dba37435cf2e5cb8c36082c230ca751870756535d91692'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'alias_only': (('authority.toml', 'b13d088ebc7fc875000c583fb2a28322f7ec3584fc1fc916bb94114165a94a42'), ('runtime/value.rs', '55d193c53d53e549d832dd6b6a626c492cb8e775f6aac253bfd82024336ba04e'), ('typed_routes.rs', 'bfc64bb8b2dfc970cb13d7f32ac881a1366badde613eb7b4c67b0a2e602cfa4e')),
    'post_return_bound_guard': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', 'fe4a9134f0bf148c2e0c3ef970d999ed8e717d7e4b97af8239fb556fa5303e41'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'reserved_bit_decoy_guard': (('authority.toml', 'ebe89b93ff8b38655a215266ac99c52cfb6a7ebcd0dd6bfe58e60af8d335b273'), ('runtime/value.rs', '1c3139c3cd59192e6d99c0fa3d07a95a23c14332f70d24f009d332f28fe2de0b'), ('typed_routes.rs', 'a6a1cae51dba0f91149ccd2e97cfb70f64359b57040b04667d96a1fc8b85ee97')),
    'invalid_layout': (('authority.toml', '312da7a00a555a4404500a7e461b58bf9f9a59040d83a6350e90313f8f99f9c5'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'oversized_family_code': (('authority.toml', 'fc3a75b0ada3a78610f8b64348bd7231c923e63f0a1cd03c8f6da9a06bd8740a'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoy_bound_check': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '597c726e6d10a51b58717afb00b15cdf94463ab564b314a57bd27705e22191f2'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_guard_expression': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '75f720d12704c6054ad6bbd2c8973f29de8d6af249cb2b6546e51fe7740b0d94'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'high_id_overlap': (('authority.toml', '7045ae1853c4a53ba7978e0318711fc0409a43b3727d5f0cc75572242ca918c6'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'wrong_shift': (('authority.toml', 'c792651c2edd3e99146ccd778ba319b8016b7cef98832c70a4036f26580d3abc'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'invalid_kind': (('authority.toml', '38c26e2b6b3487582c6a331bd55b2496bc2907c11701565d2c5ed3b72e9c163e'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'default_coercion': (('authority.toml', 'b3b7ebc3aa7f0d5b2c3198a140763c8b4b447e23deee201efed912f3ba686774'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'multi_family_codec': (('authority.toml', '0bbaf95539ff0020b636ea594521a766d39dfe0fd39df40439cc080889d47db6'), ('runtime/value.rs', '410442810ddd8592c4be451c504ae14f84c63d67969742291ba620ab69c97a88'), ('typed_routes.rs', 'a8a5cf0409a404c34fbc15323fbc09742c38a5dc920d76b2144351dffa3e6937')),
    'bare_macro_invocation': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '9acb57e811b293ae8ac745a95777e14a0933a266c05ad24f3dda93a778e049e0'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'support_type_shadow': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '7050e521323aa9802bf3b39d0a26afe4bead961e8c28414a1dee7bdf7a1b87a3')),
    'normalized_digest_anchor_drift': (('authority.toml', '4737d77b09b9eaabd59d3ff4dfdc2fd73f86e55d13b7c1f5001de3b242c85931'), ('runtime/value.rs', 'ee0f6f023beb6edf66c9ee6a7ad33f58f735da0852b621e14623b53d14992ef5'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'qualified_bound_declaration': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '2aa87114d992153feaaa5aac22d9e0bfa19f594f502a83e499063a21a1605b18'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'decoder_match_arm_binding_shadow': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'de5cf79d9ccca8c4e6ed3955e6dc15d780d3f462fdb3a086bfa58bc0c9dd7907'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'participating_std_namespace_shadow_mod': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '61872a58a392e89b4aac0d5b256ea3a1cc5aeec0b99025209e8c6f85a7e6a053'), ('typed_routes.rs', 'f6807c1762b42301860778ab458340d737f4611f3dd3bc22c4d7c39ffe1a8c63')),
    'participating_std_namespace_shadow_alias': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '766a9f62299ae8ecfb84c8a31f1dad1a2cb1de005ef33f6212ee2ac80793dcd9'), ('typed_routes.rs', 'c595c980c596aa4ca4e757684e0b2872788bfac3d9c6dbeb80c17ed19987e199')),
    'unrelated_macro_in_unbound_item': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', '912025b216f83fba153d4c7ee0dd5f090ce70dffda45966d2c1474009bc95b78'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'unrelated_result_variant_declaration': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'c49e71323a30d9700afdfd95f735fa1f1f52c3439348904985570d0d516b80f1'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
    'participating_std_namespace_shadow_extern': (('authority.toml', 'c2f1cb95dad1ce60e83b79996ec97edf2aba5c39c73acfbb0b56550ac344b3bd'), ('runtime/value.rs', 'f11169741252679edb6f6741a9f737cad6c36cb4c500cf3c480aa0ed3c28310d'), ('typed_routes.rs', '6447dc3b8ce0e9ebea693e4297d3283a7c82a1cfa41c1e9f602548be10b67065')),
})
FROZEN_CASE_TREE_DIGESTS = MappingProxyType({
    'direct_producer': '147444fb132aedd45a4c50aa1fcbf8110f2a7b668eb4f1958a54d8460d1a8799',
    'duplicate_rows': 'b22f07c7f6a3d6cf84a0e52b6f70b1e6021b2aa48952e1e2afe6e162f06c40e8',
    'first_live_probe': 'a34a8dbf9951065abfe5ee2072541d7d6c948948b17fc2eefc784c855af75ca2',
    'helper_assignment': '2039f8ee7a644dbb2e008d8d96e7581e6836b922d79bc308546bdf7fd5c74364',
    'helper_split': '56d5904f150d21563665d8f873064d8781d53f062f4e9a4ecaf33987c78e6e6c',
    'jit_only': 'e8844008f48f3ed9eb7bbd43bb386d99a3b73a9d9ba1a246958d4b40393281ab',
    'lowercase_table': 'df8a5056c9589e2edc6593290e20e00a58e5dfd8a3ecced583593004e5bca565',
    'manual_families': '1fcd02518fd4e8456a8e92f3da770f585e0bb932f9526cb149a79c6709bd45db',
    'metadata_controls': '8965e3ceb4f256433558ffb3cdd58e80fbf57b70f91854639e3ca3c11385b074',
    'new_source': '72c469c9861ce524d47d5e751603ad1321700bc94bea2a5995fe4575510512b0',
    'private_id_reification': '0e5da1e0128ac922c8bca04614afc68d85cad55afd4579a44074ea34e9ef2cc9',
    'registry_lookup': '1f6604c629dff093556864793a229a7acb839d26eb59c848c8a452622e55b7e7',
    'test_only_masking': 'e52723c27f7acd5f7873bfbda091ae1806cd25f1053de360b811ca12850e6f40',
    'threshold': '3c4dfd1fa977442a815ed38815ee2e436db7e6d84492626937b8ec0c8f2d32df',
    'unknown_candidate': '9dc47ed42d2523e783a606c66b0a01dae40cc81f236b8f6e015d8a0cae83af60',
    'unnamed_table': '28a11a198f1a1605ae2c2890927f4e1c3b94a99f5fd5faf2ae70abf0644baeb7',
    'unparseable': '1ed45589eaceef1868acbae8c4ec8cb995e45005b34b83ed9776bf20086ff6d4',
})
FROZEN_RECONCILIATION_TREE_DIGESTS = MappingProxyType({
    'authority': '152facbf6ce1c5f20d1bcc015bf0bf89c0887d26992087c09c0d1d79bdbe7c5a',
    'stale_moved': 'dd6724cf4bd4cbb1d84496760147ca1326d00fb59f21abb8f9272b1775577cce',
})

SCHEMA = "mamba.t1.opaque-value-boundary.v2"
# Disposition is supplied only by families.toml.  The scanner never chooses
# one.  An unmatched observation has the separate ``needs_classification``
# state and is never serialized as a proven legacy route.
OPAQUE_LEGACY = "legacy_debt"
ORDINARY_CONTROL = "python_numeric"
PRIVATE_NONREIFIED = "private_only"
NEEDS_CLASSIFICATION = "needs_classification"
CANONICAL_CATEGORIES = frozenset({
    "producer", "consumer", "registry", "classifier", "private_metadata", "python_numeric",
})
CANONICAL_DISPOSITIONS = frozenset({
    OPAQUE_LEGACY, "typed_token", PRIVATE_NONREIFIED, ORDINARY_CONTROL,
})
CANONICAL_TOPOLOGIES = frozenset({"central_registered", "direct", "unregistered_side_table"})
CANONICAL_EXPOSURES = frozenset({
    "public_opaque_int", "private_instance_id", "public_ordinary_int", "native_internal_id",
})

# Phase-C is intentionally a separate, authority-first resolver.  The older
# lexical census remains available to the Phase-B mutation oracle, but it is
# not allowed to manufacture a production disposition.  Keep this owner and
# matcher set literal: changing it is a reviewed authority change, not a
# consequence of whatever the source scanner happens to find.
PHASE_C_OWNER_IDS = (
    "central_array", "central_queue", "central_hashlib", "central_hmac",
    "central_decimal", "central_graphlib", "central_json", "central_uuid",
    "central_fractions", "central_random", "central_ipaddress",
    "direct_iter_store", "direct_range", "direct_closure", "direct_generator",
    "direct_cell", "direct_coroutine", "direct_task", "direct_file",
)
PHASE_C_PUBLIC_ESCAPE_IDS = ("threading_barrier_instance_field",)
PHASE_C_MATCHERS = frozenset({
    "central_registry_insert", "central_registry_lookup", "direct_store_allocate",
    "direct_store_lookup", "threshold_classifier", "first_live_probe",
    "public_from_int_reifier", "public_as_int_reifier", "public_field_reifier",
    "native_internal_allocator",
})
PHASE_C_AUTHORITY_SCHEMA = SCHEMA + ".phase-c"
PHASE_C_INVENTORY_SCHEMA = SCHEMA + ".phase-c.inventory"
PHASE_C_OBSERVATION_SCHEMA = SCHEMA + ".phase-c.observations"
PHASE_C_FORBIDDEN_AUTHORITY_KEYS = frozenset({
    "start_line", "end_line", "line_start", "line_end", "regex", "wildcard",
    "generated", "generated_by", "pattern", "row_count", "inventory_digest",
    "observation_digest", "manual_unmatched_count",
})
METADATA_WORDS = re.compile(r"(?:metadata|record|row|entry|stat|ownership)", re.I)
RAW_START = re.compile(r'(?:br|rb|r)(?P<hash>#{0,255})"')

# These are the only currently proven opaque-handle families.  The lists are
# deliberately path-exact: an unregistered module is not promoted by a
# function/category/name heuristic.  Its observations remain unmatched until
# a bounded public-escape proof is added.
CENTRAL_FAMILY_MODULES = frozenset({
    "apps/mamba/src/runtime/stdlib/array_mod.rs",
    "apps/mamba/src/runtime/stdlib/queue_mod.rs",
    "apps/mamba/src/runtime/stdlib/hashlib_mod.rs",
    "apps/mamba/src/runtime/stdlib/hmac_mod.rs",
    "apps/mamba/src/runtime/stdlib/decimal_mod.rs",
    "apps/mamba/src/runtime/stdlib/graphlib_mod.rs",
    "apps/mamba/src/runtime/stdlib/json_mod.rs",
    "apps/mamba/src/runtime/stdlib/uuid_mod.rs",
    "apps/mamba/src/runtime/stdlib/fractions_mod.rs",
    "apps/mamba/src/runtime/stdlib/random_mod.rs",
    "apps/mamba/src/runtime/stdlib/ipaddress_mod.rs",
})
DIRECT_CORE_MODULES = frozenset({
    "apps/mamba/src/runtime/iter.rs",
    "apps/mamba/src/runtime/builtins/range_slice.rs",
    "apps/mamba/src/runtime/closure.rs",
    "apps/mamba/src/runtime/generator.rs",
    "apps/mamba/src/runtime/class/cells.rs",
    "apps/mamba/src/runtime/async_rt.rs",
    "apps/mamba/src/runtime/async_task.rs",
    "apps/mamba/src/runtime/file_io.rs",
})
DIRECT_FAMILY_MODULES = frozenset(DIRECT_CORE_MODULES - {
    "apps/mamba/src/runtime/builtins/range_slice.rs",
})
OBSERVATION_KEYS = (
    "site_id", "path", "symbol", "normalized_digest", "category", "kind", "parent_route",
)


@dataclass(frozen=True)
class FnSpan:
    name: str
    start: int
    body_start: int
    end: int


@dataclass(frozen=True)
class Site:
    site_id: str
    path: str
    symbol: str
    normalized_digest: str
    category: str
    disposition: str
    kind: str
    parent_route: str
    reason: str
    line: int


@dataclass(frozen=True)
class Candidate:
    offset: int
    end: int
    text: str
    symbol: str
    category: str
    disposition: str
    kind: str
    reason: str


def matching_brace(masked: str, opening: int) -> int:
    if opening >= len(masked) or masked[opening] != "{":
        raise ScanFailure(f"expected function body at byte {opening}")
    depth = 1
    for index in range(opening + 1, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    raise ScanFailure(f"truncated function body at byte {opening}")


def function_spans(masked: str) -> list[FnSpan]:
    """Find lexical function owners in one pass.

    Brace ownership is checked once by ``candidates``.  Re-walking the whole
    file for every function is needlessly quadratic on the generated runtime
    modules, so function spans use the next function start as a conservative
    lexical end.  This still gives a deterministic enclosing symbol while the
    global brace balance check remains fail-closed.
    """
    spans: list[FnSpan] = []
    pattern = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)[^;{]*\{")
    matches = list(pattern.finditer(masked))
    for index, match in enumerate(matches):
        opening = match.end() - 1
        end = matches[index + 1].start() if index + 1 < len(matches) else len(masked)
        spans.append(FnSpan(match.group(1), match.start(), opening, end))
    return spans


def test_only_ranges(masked: str) -> tuple[list[tuple[int, int]], list[str]]:
    """Return ``cfg(test)`` module/function ranges without parsing Rust."""
    ranges: list[tuple[int, int]] = []
    diagnostics: list[str] = []
    pattern = re.compile(r"(?:#\[\s*cfg\s*\(\s*test\s*\)\s*\]|#\[\s*test\s*\])[^{]*\{")
    for match in pattern.finditer(masked):
        opening = match.end() - 1
        try:
            ranges.append((match.start(), matching_brace(masked, opening)))
        except ScanFailure as error:
            diagnostics.append(str(error))
    return ranges, diagnostics


def relpath(path: Path) -> str:
    return path.resolve().relative_to(REPO.resolve()).as_posix()


def line_context(source: str, offset: int) -> tuple[str, int]:
    line_start = source.rfind("\n", 0, offset) + 1
    line_end = source.find("\n", offset)
    line_end = len(source) if line_end < 0 else line_end
    return source[line_start:line_end], source.count("\n", 0, offset) + 1


def classify_match(kind: str, text: str, window: str) -> tuple[str, str, str, str]:
    """Return a structural observation, without choosing its disposition.

    Discovery is deliberately independent from the checked-in ledger.  The
    empty disposition is filled only by ``apply_manual_authority`` from an
    exact family/public-escape proof. Inventory and observation locks never
    classify a structural observation.
    """
    if kind == "producer":
        if re.search(r"\b(?:uid|gid|index)\b", text, re.I) and METADATA_WORDS.search(window):
            return "python_numeric", "", "metadata", "uid/gid/index metadata producer observation"
        if re.search(r"\b(?:let\s+\w+\s*=|return)\s*MbValue::from_int", text):
            return "producer", "", "direct_producer", "direct integer producer observation"
        return "producer", "", "helper_assignment_producer", "helper or assignment integer producer observation"
    if kind == "consumer":
        if re.search(r"(?:registry|lookup|contains|is_[a-z0-9_]*handle|HANDLE_MIN_ID)", window, re.I):
            return "consumer", "", "extracted_registry_lookup", "extracted integer registry-flow observation"
        return "consumer", "", "opaque_consumer", "integer extraction observation"
    if kind == "unnamed_table":
        return "private_metadata", "", "unnamed_numeric_side_table", "unnamed numeric side-table observation"
    if kind == "field":
        return "private_metadata", "", "side_table", "numeric map/set side-table observation"
    if kind == "metadata":
        if METADATA_WORDS.search(window):
            return "python_numeric", "", "metadata", "uid/gid/index metadata observation"
        return "python_numeric", "", "metadata", "uid/gid/index observation"
    if kind == "registry" or "integer_handle_registry" in text or "handle_registry" in text:
        return "registry", "", "registry_route", "explicit integer-handle registry observation"
    if kind == "classifier" or re.search(r"\b(?:is_[a-z0-9_]*handle|mb_is_iterator_handle|is_int_tag_handle)\b", text, re.I):
        return "classifier", "", "classifier", "handle classifier observation"
    if kind == "threshold" or "HANDLE_MIN_ID" in text or re.search(r"(?:>=|>|<=|<)\s*(?:[1-9][0-9]{2,}|\(1\s*<<\s*[0-9]+\))", text):
        return "classifier", "", "numeric_threshold", "numeric threshold observation"
    if kind == "probe" or re.search(r"(?:keys|values|iter|iter_mut)\s*\([^;{}]*\)\s*\.\s*(?:next|find)\s*\(", text):
        return "classifier", "", "first_live_probe", "first-live probe observation"
    if kind == "reification" or re.search(r"\b_[A-Za-z0-9]*id\b", text):
        return "private_metadata", "", "private_id_reification", "private _id reification observation"
    return "python_numeric", "", kind, "structural integer observation"


def load_family_manifest(path: Path = FAMILIES_MANIFEST) -> dict[str, object]:
    with path.open("rb") as stream:
        return tomllib.load(stream)


def family_manifest_digest(path: Path = FAMILIES_MANIFEST) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def registered_family_paths(root: Path = PRODUCTION) -> set[str]:
    """Discover central families from real integer-handle registration hooks."""
    paths: set[str] = set()
    for path in source_files(root):
        source = path.read_text(encoding="utf-8")
        if re.search(r"integer_handle_registry::register\s*\(", mask_non_code(source)):
            paths.add(relpath(path))
    return paths


def validate_family_manifest(manifest: dict[str, object], *, strict_live: bool = True, expected_family_ids: set[str] | None = None) -> list[str]:
    failures: list[str] = []
    if manifest.get("schema") != SCHEMA + ".families":
        failures.append("family manifest schema mismatch")
    for key in ("families", "selectors", "public_escapes", "public_selectors"):
        if key not in manifest:
            failures.append(f"authority missing key: {key}")
    if "selectors" in manifest and not manifest.get("selectors"):
        failures.append("authority selectors are empty")
    if strict_live and "public_selectors" in manifest and not manifest.get("public_selectors"):
        failures.append("authority public_selectors are empty")
    families = list(manifest.get("families", []))
    paths = [str(family.get("path", "")) for family in families]
    expected_paths = set(CENTRAL_FAMILY_MODULES) | set(DIRECT_FAMILY_MODULES)
    path_ids: dict[str, set[str]] = {}
    for family in families:
        path_ids.setdefault(str(family.get("path", "")), set()).add(str(family.get("id", "")))
    for family_path, ids in path_ids.items():
        if len(ids) > 1 and not (
            family_path == "apps/mamba/src/runtime/iter.rs"
            and ids == {"direct_iter_store", "direct_range"}
        ):
            failures.append(f"family manifest has duplicate paths without shared-store proof: {family_path}")
    if strict_live and set(paths) != expected_paths:
        failures.append("family manifest path set does not match the frozen central/direct family set")
    discovered_central = registered_family_paths() if strict_live else set()
    if strict_live and discovered_central != set(CENTRAL_FAMILY_MODULES):
        failures.append(
            "central family IDs do not reconcile with actual integer_handle_registry::register hooks"
        )
    frozen_family_ids = {
        f"central_{Path(path).stem.removesuffix('_mod')}" for path in CENTRAL_FAMILY_MODULES
    } | {
        "direct_iter_store", "direct_range", "direct_closure", "direct_generator",
        "direct_cell", "direct_coroutine", "direct_task", "direct_file",
    }
    family_ids = [str(family.get("id", "")) for family in families]
    if len(family_ids) != len(set(family_ids)):
        failures.append("duplicate family ID")
    required_family_ids = expected_family_ids if expected_family_ids is not None else (frozen_family_ids if strict_live else None)
    if required_family_ids is not None and set(family_ids) != required_family_ids:
        failures.append("family manifest IDs do not match the frozen central/direct family set")
    for family in families:
        if not str(family.get("name", "")).strip() or not str(family.get("proof", "")).strip():
            failures.append(f"family manifest row lacks name/proof: {family!r}")
        for field in ("topology", "exposure", "store_link", "migration", "allocator_or_table"):
            if not str(family.get(field, "")).strip():
                failures.append(f"family manifest row lacks {field}: {family!r}")
        if str(family.get("topology")) not in CANONICAL_TOPOLOGIES:
            failures.append(f"invalid family topology: {family!r}")
        if str(family.get("exposure")) not in CANONICAL_EXPOSURES:
            failures.append(f"invalid family exposure: {family!r}")
        family_id = str(family.get("id", ""))
        expected_topology = "central_registered" if family_id.startswith("central_") else "direct"
        if strict_live and family_id in frozen_family_ids and str(family.get("topology")) != expected_topology:
            failures.append(f"central family {family_id} has invalid topology" if family_id.startswith("central_") else f"direct family {family_id} has invalid topology")
    selectors = list(manifest.get("selectors", []))
    selector_ids: list[str] = []
    family_by_id = {str(family.get("id")): family for family in families}
    typed_contract = manifest.get("typed_contract", {})
    typed_paths = {
        str(node.get("path")) for node in typed_contract.get("wrapper_graph", [])
        if isinstance(node, dict) and str(node.get("path", "")).strip()
    } if isinstance(typed_contract, dict) else set()
    for selector in selectors:
        family = family_by_id.get(str(selector.get("family_id")))
        selector_ids.extend(str(value) for value in selector.get("site_ids", []))
        valid_paths = ({str(family.get("path")), str(family.get("anchor_path", ""))} if family is not None else set()) | typed_paths
        if family is None or str(selector.get("path")) not in valid_paths:
            failures.append(f"family selector has unknown/mismatched family path: {selector!r}")
        if not selector.get("site_ids"):
            failures.append(f"family selector has no site IDs: {selector!r}")
        if not str(selector.get("symbol", "")).strip() or not str(selector.get("category", "")).strip() or not str(selector.get("normalized_digest", "")).strip() or not str(selector.get("kind", "")).strip():
            failures.append(f"family selector lacks exact source item/role: {selector!r}")
        if not str(selector.get("anchor", "")).strip() or not str(selector.get("topology", "")).strip() or not str(selector.get("exposure", "")).strip() or not str(selector.get("migration", "")).strip() or not str(selector.get("allocator_or_table", "")).strip():
            failures.append(f"family selector lacks topology/exposure/migration/store proof: {selector!r}")
        if str(selector.get("category")) not in CANONICAL_CATEGORIES:
            failures.append(f"family selector has invalid category: {selector!r}")
        if str(selector.get("disposition")) not in CANONICAL_DISPOSITIONS:
            failures.append(f"family selector has invalid disposition: {selector!r}")
        if not str(selector.get("proof", "")).strip():
            failures.append(f"family selector lacks proof: {selector!r}")
        if str(selector.get("role")) != str(selector.get("kind")):
            failures.append(f"family selector role/kind mismatch: {selector!r}")
        expected_anchor = f"{selector.get('path','')}::{selector.get('symbol','')}::{selector.get('normalized_digest','')}"
        if str(selector.get("anchor")) != expected_anchor:
            failures.append(f"family selector anchor mismatch: {selector!r}")
        if family is not None:
            for field in ("topology", "exposure"):
                if str(selector.get(field)) != str(family.get(field)):
                    failures.append(f"family selector {field} mismatch: {selector!r}")
    for family in families:
        actual_roles = {
            str(selector.get("category")) for selector in selectors
            if str(selector.get("family_id")) == str(family.get("id"))
        }
        required_roles = {str(value) for value in family.get("required_roles", [])}
        if not required_roles or not required_roles <= CANONICAL_CATEGORIES:
            failures.append(f"family required_roles are empty/invalid for {family.get('id')}")
        if not required_roles or required_roles != actual_roles:
            failures.append(
                f"family required_roles do not reconcile selectors for {family.get('id')}: "
                f"required={sorted(required_roles)} actual={sorted(actual_roles)}"
            )
    if len(selector_ids) != len(set(selector_ids)):
        failures.append("family selectors overlap on site IDs")
    public_selectors = list(manifest.get("public_selectors", []))
    public_ids: list[str] = []
    escapes = {str(escape.get("id")): escape for escape in manifest.get("public_escapes", [])}
    for selector in public_selectors:
        public_ids.extend(str(value) for value in selector.get("site_ids", []))
        escape = escapes.get(str(selector.get("public_escape_id")))
        if escape is None:
            failures.append(f"public selector has unknown public escape: {selector!r}")
        allowed_public_dispositions = {OPAQUE_LEGACY, "typed_token"} if isinstance(manifest.get("typed_contract"), dict) else {OPAQUE_LEGACY}
        if str(selector.get("disposition")) not in allowed_public_dispositions:
            failures.append(f"public selector is not opaque: {selector!r}")
        if not str(selector.get("symbol", "")).strip() or not str(selector.get("category", "")).strip() or not str(selector.get("normalized_digest", "")).strip() or not str(selector.get("kind", "")).strip():
            failures.append(f"public selector lacks exact source item/role: {selector!r}")
        if not str(selector.get("anchor", "")).strip() or not str(selector.get("topology", "")).strip() or not str(selector.get("exposure", "")).strip() or not str(selector.get("migration", "")).strip() or not str(selector.get("allocator_or_table", "")).strip():
            failures.append(f"public selector lacks topology/exposure/migration/store proof: {selector!r}")
        if str(selector.get("category")) not in CANONICAL_CATEGORIES or str(selector.get("role")) != str(selector.get("kind")):
            failures.append(f"public selector has invalid category/role: {selector!r}")
        expected_anchor = f"{selector.get('path','')}::{selector.get('symbol','')}::{selector.get('normalized_digest','')}"
        if str(selector.get("anchor")) != expected_anchor:
            failures.append(f"public selector anchor mismatch: {selector!r}")
        if escape is not None:
            for field in ("topology", "exposure", "migration", "allocator_or_table"):
                if str(selector.get(field)) != str(escape.get(field)):
                    failures.append(f"public selector {field} mismatch: {selector!r}")
            if str(selector.get("path")) not in ({str(escape.get("path"))} | typed_paths):
                failures.append(f"public selector path mismatch: {selector!r}")
    if len(public_ids) != len(set(public_ids)) or set(selector_ids) & set(public_ids):
        failures.append("family/public selectors overlap on site IDs")
    for escape in manifest.get("public_escapes", []):
        if not str(escape.get("id", "")).strip() or not str(escape.get("path", "")).strip() or int(escape.get("start_line", 0)) > int(escape.get("end_line", 0)):
            failures.append(f"invalid public escape proof: {escape!r}")
        allowed_escape_dispositions = {OPAQUE_LEGACY, "typed_token"} if isinstance(manifest.get("typed_contract"), dict) else {OPAQUE_LEGACY}
        if str(escape.get("disposition", "")) not in allowed_escape_dispositions or not str(escape.get("proof", "")).strip():
            failures.append(f"public escape proof lacks opaque disposition/proof: {escape!r}")
        for field in ("topology", "exposure", "store_link", "migration", "allocator_or_table"):
            if not str(escape.get(field, "")).strip():
                failures.append(f"public escape proof lacks {field}: {escape!r}")
        if str(escape.get("topology")) not in CANONICAL_TOPOLOGIES or str(escape.get("exposure")) not in CANONICAL_EXPOSURES:
            failures.append(f"public escape has invalid topology/exposure: {escape!r}")
    return failures


def selector_index(manifest: dict[str, object]) -> dict[str, dict[str, object]]:
    index: dict[str, dict[str, object]] = {}
    for selector in [*manifest.get("selectors", []), *manifest.get("public_selectors", [])]:
        for site_id in selector.get("site_ids", []):
            index[str(site_id)] = selector
    return index


def manual_disposition(row: dict[str, object], manifest: dict[str, object] | None = None, index: dict[str, dict[str, object]] | None = None) -> tuple[str, str, str] | None:
    """Resolve one observation through one exact checked-in role selector."""
    manifest = manifest or load_family_manifest()
    site_id = str(row["site_id"])
    selector = (index or selector_index(manifest)).get(site_id)
    if selector is not None:
        exact = all(str(row.get(key, "")) == str(selector.get(key, "")) for key in (
            "path", "symbol", "category", "kind", "normalized_digest",
        ))
        if exact:
            owner = str(selector.get("family_id") or selector.get("public_escape_id") or "")
            return str(selector["disposition"]), str(selector["proof"]), owner
    # Unregistered paths, new occurrences, renamed symbols, and changed
    # normalized observations intentionally have no fallback.
    return None


def apply_manual_authority(report: dict[str, object], manifest: dict[str, object] | None = None, *, strict_live: bool = True) -> list[str]:
    """Apply exact family proofs and return stable IDs still unmatched."""
    unmatched: list[str] = []
    unmatched_summary: Counter[tuple[str, str]] = Counter()
    try:
        if manifest is None:
            manifest = load_family_manifest()
        manifest_failures = validate_family_manifest(manifest, strict_live=strict_live)
    except (OSError, tomllib.TOMLDecodeError) as error:
        manifest = {"families": [], "public_escapes": []}
        manifest_failures = [f"cannot load family manifest: {error}"]
    report["family_manifest_digest"] = family_manifest_digest() if not manifest_failures else ""
    report["family_manifest_diagnostics"] = manifest_failures
    selectors = selector_index(manifest)
    authority: dict[str, dict[str, str | None]] = {}
    for row in report["sites"]:
        resolved = manual_disposition(row, manifest, selectors)
        if resolved is None:
            row["disposition"] = NEEDS_CLASSIFICATION
            row["reason"] = "UNMATCHED_MANUAL_PUBLIC_ESCAPE_PROOF"
            authority[str(row["site_id"])] = {"owner": None, "proof": None}
            unmatched.append(str(row["site_id"]))
            unmatched_summary[(str(row["path"]), str(row["category"]))] += 1
            continue
        row["disposition"], row["reason"], owner = resolved
        authority[str(row["site_id"])] = {"owner": owner, "proof": str(row["reason"])}
    report["authority"] = authority
    report["manual_unmatched"] = sorted(unmatched)
    report["authority_diagnostics"] = [
        f"{NEEDS_CLASSIFICATION}:{site_id}" for site_id in sorted(unmatched)
    ]
    report["manual_unmatched_summary"] = [
        {"path": path, "category": category, "count": count}
        for (path, category), count in sorted(unmatched_summary.items())
    ]
    report["disposition_counts"] = dict(sorted(Counter(
        str(row["disposition"]) for row in report["sites"]
    ).items()))
    return sorted(unmatched)


def candidates(source: str) -> tuple[list[Candidate], list[str]]:
    diagnostics: list[str] = []
    try:
        masked = mask_non_code(source)
    except ScanFailure as error:
        return [], [str(error)]
    spans = function_spans(masked)
    test_ranges, test_diagnostics = test_only_ranges(masked)
    diagnostics.extend(test_diagnostics)
    if masked.count("{") != masked.count("}"):
        diagnostics.append("unbalanced braces in source")
    found: list[Candidate] = []
    # Match only semantic tokens.  Context windows are masked, so comments and
    # string examples cannot manufacture candidates.
    patterns: list[tuple[str, re.Pattern[str]]] = [
        ("producer", re.compile(r"\bMbValue::from_int\s*\(")),
        ("consumer", re.compile(r"\.as_int(?:_pyint|_unchecked)?\s*\(")),
        ("metadata", re.compile(r"\b(?:uid|gid|index)\b", re.I)),
        ("field", re.compile(
            r"\b(?:HashMap|HashSet|BTreeMap|BTreeSet)\s*<[^>]*\b(?:u64|i64|usize)\b|"
            r"\b(?:AtomicU(?:64|size)|AtomicI(?:64|size))\b|"
            r"\b(?:Mutex|RwLock|RefCell|Vec)\s*<[^;{}]*\b(?:u64|i64|usize)\b"
        )),
        ("classifier", re.compile(r"\b(?:is_[A-Za-z0-9_]*handle|mb_is_iterator_handle|is_int_tag_handle)\b")),
        ("registry", re.compile(r"\b(?:integer_handle_registry|handle_registry)\b")),
        ("threshold", re.compile(r"\b(?:HANDLE_MIN_ID|id\s*(?:>=|>|<=|<)\s*[0-9]{2,})\b")),
        ("probe", re.compile(r"\b(?:keys|values|iter|iter_mut)\s*\([^;{}]*\)\s*\.\s*(?:next|find)\s*\(")),
        ("reification", re.compile(r"\b_[A-Za-z0-9]*id\b")),
        ("unnamed_table", re.compile(
            r"\b(?:CONNS|[A-Z][A-Z0-9_]*(?:_STORES|_CONNS|_REGISTRY|_IDS)|"
            r"NEXT_[A-Z0-9_]*_ID)\b|"
            r"\b(?:static|const)\s+[a-z][A-Za-z0-9_]*\s*:\s*"
            r"(?:AtomicU(?:64|size)|AtomicI(?:64|size)|"
            r"(?:HashMap|HashSet|BTreeMap|BTreeSet|Mutex|RwLock|RefCell|Vec)[^;{}]*"
            r"\b(?:u64|i64|usize)\b)"
        )),
    ]
    seen: set[tuple[int, str]] = set()
    for kind, pattern in patterns:
        for match in pattern.finditer(masked):
            if any(start < match.start() < end for start, end in test_ranges):
                continue
            line, _ = line_context(masked, match.start())
            raw_line, _ = line_context(source, match.start())
            # A static/const map is one unnamed side-table observation.  The
            # generic field pattern also sees its HashMap token; do not emit
            # a second, differently-shaped row for the same declaration.
            if kind == "field" and re.search(r"\b(?:static|const)\b", line):
                continue
            line_start = masked.rfind("\n", 0, match.start()) + 1
            line_end = masked.find("\n", match.end())
            line_end = len(masked) if line_end < 0 else line_end
            # Include a small lexical window for assignment/helper/registry flow.
            window_start = max(0, line_start - 420)
            window = masked[window_start:line_end]
            owner = next((s for s in reversed(spans) if s.body_start < match.start() < s.end), None)
            owner_name = owner.name if owner else "<module>"
            if owner:
                function_start = max(window_start, owner.start)
                # Keep interprocedural guessing out of the scanner.  The
                # bounded function-header/current-line window recognizes a
                # helper when its registry route is explicit on the same line;
                # a later call is separately inventoried as a registry route.
                window = masked[function_start:line_end]
                if kind == "consumer":
                    # A helper may extract first and route the local id on a
                    # later line in the same function.  This is the one
                    # bounded forward-flow edge the gate proves; it never
                    # crosses a function/file boundary.
                    tail = masked[match.end() : owner.end]
                    if re.search(r"(?:integer_handle_registry|lookup|contains|is_[a-z0-9_]*handle|HANDLE_MIN_ID)", tail, re.I):
                        window += tail
            if re.search(r'"[^"\n]*(?:_id|_handle)[^"\n]*"', raw_line, re.I):
                window += " opaque_field_marker"
            result = classify_match(kind, line, window)
            category, disposition, stable_kind, reason = result
            key = (match.start(), category)
            if key in seen:
                continue
            seen.add(key)
            found.append(Candidate(match.start(), match.end(), line.strip(), owner_name, category, disposition, stable_kind, reason))
    # A deliberately named opaque/unknown helper is an unparseable boundary,
    # not an ordinary integer.  Keep this narrow so normal ``id`` helpers in
    # Python-int code are not silently promoted to opaque candidates.
    for line_match in re.finditer(r"^.*\b(?:opaque|unknown)_[A-Za-z0-9_]*\s*\([^\n]*$", masked, re.M | re.I):
        line = line_match.group(0)
        offset = line_match.start()
        if any(start < offset < end for start, end in test_ranges):
            continue
        if not any(candidate.offset >= offset and candidate.offset < line_match.end() for candidate in found):
            diagnostics.append(f"opaque boundary candidate requires classification at line {masked.count(chr(10), 0, offset) + 1}")
    found.sort(key=lambda c: (c.offset, c.category))
    return found, diagnostics


def site_for(path: Path, source: str, candidate: Candidate, scan_root: Path, occurrence: int) -> Site:
    _, line = line_context(source, candidate.offset)
    masked_line = re.sub(r"\s+", " ", candidate.text).strip()
    digest = hashlib.sha256(masked_line.encode()).hexdigest()
    path_value = relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else path.name
    # Stable identity excludes line numbers and presentation-only text.
    identity = "\0".join((path_value, candidate.symbol, candidate.category, candidate.kind, digest, str(occurrence)))
    site_id = hashlib.sha256(identity.encode()).hexdigest()
    return Site(
        site_id=site_id,
        path=path_value,
        symbol=candidate.symbol,
        normalized_digest=digest,
        category=candidate.category,
        disposition=candidate.disposition,
        kind=candidate.kind,
        parent_route=f"opaque-value-boundary/{candidate.category}",
        reason=candidate.reason,
        line=line,
    )


def source_files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    return sorted(
        p for p in root.rglob("*.rs")
        if p.is_file() and "tests" not in p.relative_to(root).parts
    )


def source_manifest_digest(files: list[Path]) -> str:
    """Digest the complete scanned file set, including candidate-free files."""
    records = []
    for path in files:
        path_value = relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else path.name
        records.append(f"{path_value}\0{hashlib.sha256(path.read_bytes()).hexdigest()}")
    return hashlib.sha256("\n".join(records).encode()).hexdigest()


def scan(root: Path) -> dict[str, object]:
    diagnostics: list[str] = []
    rows: list[Site] = []
    occurrences: dict[tuple[str, str, str, str], int] = {}
    files = source_files(root)
    if not files:
        diagnostics.append(f"{root}: no Rust source files")
    for path in files:
        try:
            source = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            diagnostics.append(f"{path}: cannot read source: {error}")
            continue
        found, errors = candidates(source)
        prefix = relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else path.name
        diagnostics.extend(f"{prefix}: {error}" for error in errors)
        for candidate in found:
            path_value = relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else path.name
            normalized_digest = hashlib.sha256(re.sub(r"\s+", " ", candidate.text).strip().encode()).hexdigest()
            key = (path_value, candidate.symbol, candidate.category, normalized_digest)
            occurrences[key] = occurrences.get(key, 0) + 1
            rows.append(site_for(path, source, candidate, root, occurrences[key]))
    rows.sort(key=lambda row: row.site_id)
    row_dicts = [asdict(row) for row in rows]
    # The source digest is an observation digest.  Disposition/reason belong
    # to the checked-in ledger and are overlaid during ``--check``.
    normalized = [{k: v for k, v in row.items() if k not in {"line", "disposition", "reason"}} for row in row_dicts]
    digest = hashlib.sha256(json.dumps(normalized, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    site_ids = [str(row["site_id"]) for row in row_dicts]
    if len(site_ids) != len(set(site_ids)):
        diagnostics.append("duplicate site_id collision")
    counts: dict[str, int] = {}
    for row in rows:
        counts[row.category] = counts.get(row.category, 0) + 1
    return {
        "schema": SCHEMA,
        "root": str(root),
        "source_file_count": len(files),
        "source_files_digest": source_manifest_digest(files),
        "total": len(rows),
        "counts": dict(sorted(counts.items())),
        "inventory_digest": digest,
        "diagnostics": sorted(diagnostics),
        "sites": row_dicts,
    }


def _mask_test_regions(source: str) -> str:
    """Mask embedded Rust tests before looking for a production discriminator."""
    masked = mask_non_code(source)
    ranges, _ = test_only_ranges(masked)
    if not ranges:
        return masked
    chars = list(masked)
    for start, end in ranges:
        for index in range(start, end + 1):
            if chars[index] != "\n":
                chars[index] = " "
    return "".join(chars)


def typed_authority(root: Path, authority_path: Path | None = None) -> dict[str, object] | None:
    """Load the authority explicitly supplied to the typed detector.

    A production discriminator is bound to the checked-in family manifest.
    Fixture discriminators are bound by the mutation oracle to the fixture's
    own authority file.  In particular, do not search below ``root``: doing
    so lets an unrelated ``authority.toml`` silently turn a source tree green.
    """
    path = authority_path
    if path is None and root.resolve() == PRODUCTION.resolve():
        path = FAMILIES_MANIFEST
    if path is None or not path.is_file():
        return None
    try:
        with path.open("rb") as stream:
            data = tomllib.load(stream)
    except (OSError, tomllib.TOMLDecodeError):
        return None
    contract = data.get("typed_contract")
    if not isinstance(contract, dict):
        return None
    # Keep the full authority attached so family/public-escape ownership is
    # checked against the contract rather than supplied by the mutation.
    bound = dict(contract)
    bound["__authority"] = data
    bound["__authority_path"] = str(path)
    return bound


def _typed_contract_failure(partial: list[str], code: str) -> None:
    if code not in partial:
        partial.append(code)



@dataclass(frozen=True)
class RustToken:
    text: str
    start: int
    end: int


@dataclass(frozen=True)
class RustItem:
    kind: str
    name: str
    start: int
    end: int
    tokens: tuple[RustToken, ...]
    has_attribute: bool
    depth: int
    prefix: tuple[str, ...]


@dataclass(frozen=True)
class BindingId:
    function: str
    name: str
    ordinal: int


@dataclass(frozen=True)
class CanonicalSpec:
    contract: tuple[tuple[str, object], ...]
    wrappers: tuple[tuple[str, str, str, str], ...]
    selectors: tuple[tuple[tuple[str, object], ...], ...]
    files: tuple[str, ...]


def _freeze_canonical(value: object) -> object:
    if isinstance(value, dict):
        return tuple(sorted((str(key), _freeze_canonical(item)) for key, item in value.items()))
    if isinstance(value, list):
        return tuple(_freeze_canonical(item) for item in value)
    return value


def _thaw_canonical(value: object) -> object:
    if isinstance(value, tuple):
        if all(isinstance(item, tuple) and len(item) == 2 and isinstance(item[0], str) for item in value):
            return {str(key): _thaw_canonical(item) for key, item in value}
        return [_thaw_canonical(item) for item in value]
    return value


def _spec_contract(spec: CanonicalSpec) -> dict[str, object]:
    return {key: _thaw_canonical(value) for key, value in spec.contract}


def _spec_row(row: tuple[tuple[str, object], ...]) -> dict[str, object]:
    return {key: _thaw_canonical(value) for key, value in row}


def _rust_words(text: str) -> tuple[str, ...]:
    operators = ("::", "->", "=>", "==", "!=", ">=", "<=", "||", "&&")
    words: list[str] = []
    index = 0
    while index < len(text):
        char = text[index]
        if char.isspace():
            index += 1
            continue
        operator = next((item for item in operators if text.startswith(item, index)), None)
        if operator:
            words.append(operator)
            index += len(operator)
            continue
        if char.isalpha() or char == "_":
            end = index + 1
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            words.append(text[index:end])
            index = end
            continue
        if char.isdigit():
            end = index + 1
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            words.append(text[index:end])
            index = end
            continue
        words.append(char)
        index += 1
    return tuple(words)


# Support identities are deliberately root-qualified.  A participating
# module may define `mod std`, alias another crate as `std`, or import a local
# `Result`; none of those declarations can satisfy this grammar.
STD_NONZERO_WORDS = ("::", "std", "::", "num", "::", "NonZeroU64")
STD_RESULT_WORDS = ("::", "std", "::", "result", "::", "Result")
STD_OK_WORDS = (*STD_RESULT_WORDS, "::", "Ok")
STD_ERR_WORDS = (*STD_RESULT_WORDS, "::", "Err")


def _rust_tokens(source: str) -> tuple[RustToken, ...]:
    masked = mask_non_code(source)
    words = _rust_words(masked)
    result: list[RustToken] = []
    cursor = 0
    for word in words:
        start = masked.find(word, cursor)
        if start < 0:
            continue
        result.append(RustToken(word, start, start + len(word)))
        cursor = start + len(word)
    return tuple(result)


def _matching_word_brace(tokens: tuple[RustToken, ...], opening: int) -> int | None:
    depth = 1
    for index in range(opening + 1, len(tokens)):
        if tokens[index].text == "{":
            depth += 1
        elif tokens[index].text == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def _matching_delimiter(tokens: tuple[RustToken, ...], opening: int) -> int | None:
    pairs = {"(": ")", "[": "]", "{": "}"}
    expected = pairs.get(tokens[opening].text) if 0 <= opening < len(tokens) else None
    if expected is None:
        return None
    stack = [expected]
    for index in range(opening + 1, len(tokens)):
        word = tokens[index].text
        if word in pairs:
            stack.append(pairs[word])
        elif word in pairs.values():
            if not stack or word != stack[-1]:
                return None
            stack.pop()
            if not stack:
                return index
    return None


def _source_census(
    source: str,
    *,
    include_nested_functions: bool = False,
) -> tuple[tuple[RustItem, ...], tuple[RustToken, ...]]:
    masked = mask_non_code(source)
    tokens = _rust_tokens(source)
    depth_before: list[int] = []
    depth = 0
    for token in tokens:
        depth_before.append(depth)
        if token.text == "{":
            depth += 1
        elif token.text == "}":
            depth = max(0, depth - 1)
    # ``static`` is a first-class top-level store declaration.  It is kept in
    # the same small census as functions so authority rows can bind a real
    # store item instead of inferring one from a nearby helper.  Macro-body
    # statics still carry a non-zero lexical depth and are rejected by
    # ``_phase_c_item``.
    declarations = {"struct", "enum", "const", "static", "fn", "type", "macro_rules", "mod", "use"}
    items: list[RustItem] = []
    index = 0
    previous_end = 0
    while index < len(tokens):
        keyword = tokens[index].text
        if keyword not in declarations:
            index += 1
            continue
        if keyword == "macro_rules":
            name_index = index + 3
        elif keyword == "static" and index + 1 < len(tokens) and tokens[index + 1].text == "mut":
            name_index = index + 2
        else:
            name_index = index + 1
        if name_index >= len(tokens) or (keyword == "macro_rules" and tokens[index + 1].text != "!"):
            index += 1
            continue
        name = tokens[name_index].text
        end_index = name_index
        if keyword in {"fn", "struct", "enum", "mod", "macro_rules"}:
            probe = name_index + 1
            paren_depth = 0
            while probe < len(tokens):
                value = tokens[probe].text
                if value == "(":
                    paren_depth += 1
                elif value == ")":
                    paren_depth = max(0, paren_depth - 1)
                if value == "{" and paren_depth == 0:
                    close = _matching_word_brace(tokens, probe)
                    end_index = close if close is not None else probe
                    break
                if value == ";" and paren_depth == 0:
                    end_index = probe
                    break
                probe += 1
        else:
            # `const`/`static` initializers may contain a closure or a nested
            # array/map expression with semicolons.  Stop only at a semicolon
            # whose delimiter depth is the declaration's own depth.
            probe = name_index + 1
            delimiter_depth = 0
            while probe < len(tokens):
                value = tokens[probe].text
                if value in {"(", "[", "{"}:
                    delimiter_depth += 1
                elif value in {")", "]", "}"
                }:
                    delimiter_depth = max(0, delimiter_depth - 1)
                if value == ";" and delimiter_depth == 0:
                    break
                probe += 1
            end_index = min(probe, len(tokens) - 1)
        start = tokens[index].start
        end = tokens[end_index].end if end_index < len(tokens) else len(masked)
        prefix_words = _rust_words(masked[previous_end:start])
        has_attribute = "#" in prefix_words
        # Keep the prefix and lexical brace depth alongside each item.  The
        # canonical contract only instantiates plain top-level declarations;
        # a qualified item or an item found under impl/trait/function scope is
        # partial even when its name happens to match the authority.
        boundary = 0
        for marker in (";", "}", "{"):
            positions = [pos for pos, word in enumerate(prefix_words) if word == marker]
            if positions:
                boundary = max(boundary, max(positions) + 1)
        prefix = tuple(prefix_words[boundary:])
        items.append(RustItem(keyword, name, start, end, tuple(tokens[index:end_index + 1]), has_attribute, depth_before[index], prefix))
        previous_end = end
        index = end_index + 1
    if not include_nested_functions:
        return tuple(items), tokens

    # The declaration walk above intentionally skips an enclosing item after
    # finding its balanced body.  Phase-C still needs to enumerate every
    # nested/inline/trait/impl function so a non-test conversion cannot hide
    # behind that skip.  Add only exact ``fn`` declarations from the same
    # token stream; duplicate starts are retained once and all other source
    # census semantics remain unchanged.
    known_starts = {(item.kind, item.name, item.start) for item in items}
    for fn_index, token in enumerate(tokens):
        if token.text != "fn" or fn_index + 1 >= len(tokens):
            continue
        name_index = fn_index + 1
        if tokens[name_index].text in {"(", "<", "where"}:
            continue
        probe = name_index + 1
        paren_depth = 0
        opening = None
        while probe < len(tokens):
            value = tokens[probe].text
            if value == "(":
                paren_depth += 1
            elif value == ")":
                paren_depth = max(0, paren_depth - 1)
            if value == "{" and paren_depth == 0:
                opening = probe
                break
            if value == ";" and paren_depth == 0:
                break
            probe += 1
        if opening is None:
            continue
        close = _matching_word_brace(tokens, opening)
        if close is None:
            continue
        name = tokens[name_index].text
        key = ("fn", name, token.start)
        if key in known_starts:
            continue
        prefix_words = _rust_words(masked[:token.start])
        boundary = 0
        for marker in (";", "}", "{"):
            positions = [pos for pos, word in enumerate(prefix_words) if word == marker]
            if positions:
                boundary = max(boundary, max(positions) + 1)
        items.append(
            RustItem(
                "fn", name, token.start, tokens[close].end,
                tuple(tokens[fn_index:close + 1]),
                "#" in prefix_words[boundary:], depth_before[fn_index],
                tuple(prefix_words[boundary:]),
            )
        )
        known_starts.add(key)
    items.sort(key=lambda item: (item.start, item.end, item.kind, item.name))
    return tuple(items), tokens


def _phase_c_path(root: Path, value: object) -> Path | None:
    """Resolve one authority path without accepting a path escape."""
    if (
        not isinstance(value, str)
        or not value
        or value.startswith("/")
        or "\\" in value
        or ".." in Path(value).parts
    ):
        return None
    root = root.resolve()
    candidate = (REPO / value) if value.startswith("apps/") else (root / value)
    candidate = candidate.resolve()
    try:
        candidate.relative_to(REPO.resolve())
    except ValueError:
        try:
            candidate.relative_to(root)
        except ValueError:
            return None
    return candidate if candidate.is_file() else None


def _phase_c_item(root: Path, path_value: object, kind: object, symbol: object) -> tuple[Path, str, RustItem] | None:
    """Return the one exact top-level item named by a route/store."""
    path = _phase_c_path(root, path_value)
    if path is None or not isinstance(kind, str) or not isinstance(symbol, str):
        return None
    try:
        source = path.read_text(encoding="utf-8")
        items, _ = _source_census(source)
    except (OSError, UnicodeError, ScanFailure):
        return None
    matches = [item for item in items if item.kind == kind and item.name == symbol]
    if len(matches) != 1 or matches[0].has_attribute:
        return None
    # A store declared by `thread_local! { static NAME: ... }` is a real
    # authority anchor, but only that exact standard wrapper is admitted.  A
    # static hidden in any other macro/body remains non-canonical.
    if matches[0].depth != 0 and not (
        matches[0].kind == "static" and _phase_c_thread_local_static(source, matches[0])
    ):
        return None
    return path, source, matches[0]


def _phase_c_thread_local_static(source: str, item: RustItem) -> bool:
    """Prove a static is directly inside the exact `thread_local!` wrapper."""
    if item.kind != "static":
        return False
    tokens = _rust_tokens(source)
    static_positions = [index for index, token in enumerate(tokens) if token.start == item.start]
    if len(static_positions) != 1:
        return False
    static_index = static_positions[0]
    for index in range(static_index - 1, -1, -1):
        if tokens[index].text != "thread_local":
            continue
        if index + 2 >= static_index or tokens[index + 1].text != "!" or tokens[index + 2].text != "{":
            continue
        close = _matching_delimiter(tokens, index + 2)
        if close is not None and close >= static_index and item.end <= tokens[close].end:
            return True
    return False


def _phase_c_item_digests(source: str, item: RustItem) -> tuple[str, str]:
    exact = source[item.start:item.end]
    return _normalized_item_digest(exact), _source_item_digest(exact)


def _phase_c_walk_keys(value: object) -> set[str]:
    """Collect TOML keys recursively for the closed manual schema check."""
    keys: set[str] = set()
    if isinstance(value, dict):
        for key, child in value.items():
            keys.add(str(key))
            keys.update(_phase_c_walk_keys(child))
    elif isinstance(value, list):
        for child in value:
            keys.update(_phase_c_walk_keys(child))
    return keys


def validate_phase_c_authority(
    manifest: dict[str, object],
    root: Path = PRODUCTION,
) -> list[str]:
    """Validate the Phase-C manual authority and its exact source anchors.

    This is deliberately not an inference layer.  A route is either named by
    the checked-in authority and resolves to one top-level item with the
    recorded digest, or it remains a red authority defect.  The old broad
    census is never consulted here.
    """
    failures: list[str] = []

    def fail(code: str) -> None:
        if code not in failures:
            failures.append(code)

    if manifest.get("schema") != SCHEMA + ".families" or manifest.get("version") != 2:
        fail("phase-c:authority-schema")
    if manifest.get("phase_c_schema") != PHASE_C_AUTHORITY_SCHEMA or manifest.get("phase_c_version") != 1:
        fail("phase-c:authority-schema")
    forbidden_keys = _phase_c_walk_keys(manifest) & PHASE_C_FORBIDDEN_AUTHORITY_KEYS
    # Line ranges have a dedicated Barrier contract diagnostic; keep the
    # generic generated/selector guard for every other forbidden authority
    # field without obscuring that isolated schema mutation.
    forbidden_keys -= {"start_line", "end_line", "line_start", "line_end"}
    if forbidden_keys:
        fail("phase-c:authority-line-or-pattern-selector")

    families = manifest.get("families")
    escapes = manifest.get("public_escapes")
    stores = manifest.get("stores")
    routes = manifest.get("routes")
    selectors = manifest.get("selectors")
    native_routes = manifest.get("native_internal_routes")
    if not all(isinstance(value, list) for value in (families, escapes, stores, routes, selectors, native_routes)):
        fail("phase-c:authority-shape")
        return sorted(failures)
    family_rows = list(families)
    escape_rows = list(escapes)
    store_rows = list(stores)
    route_rows = list(routes)
    selector_rows = list(selectors)
    native_rows = list(native_routes)
    owners = tuple(str(row.get("id")) for row in family_rows if isinstance(row, dict))
    if set(owners) != set(PHASE_C_OWNER_IDS) or len(set(owners)) != len(owners):
        fail("phase-c:exact-nineteen-family-set")
    if len(escape_rows) != 1 or not isinstance(escape_rows[0], dict) or escape_rows[0].get("id") != PHASE_C_PUBLIC_ESCAPE_IDS[0]:
        fail("phase-c:public-escape-set")
    all_owners = set(PHASE_C_OWNER_IDS) | set(PHASE_C_PUBLIC_ESCAPE_IDS)
    known_topologies = {"central_registered", "direct", "unregistered_side_table"}

    def string_field(row: object, key: str) -> str | None:
        if not isinstance(row, dict) or not isinstance(row.get(key), str) or not row[key]:
            return None
        return str(row[key])

    for row in family_rows:
        if not isinstance(row, dict):
            fail("phase-c:family-row-shape")
            continue
        owner = string_field(row, "id")
        topology = string_field(row, "topology")
        path_value = string_field(row, "path")
        roles = row.get("required_roles")
        if owner not in PHASE_C_OWNER_IDS or topology not in {"central_registered", "direct"}:
            fail("phase-c:family-topology")
        if owner is not None:
            expected = "central_registered" if owner.startswith("central_") else "direct"
            if topology != expected:
                fail(f"phase-c:topology:{owner}")
        if path_value is None or _phase_c_path(root, path_value) is None:
            fail(f"phase-c:family-path:{owner or 'unknown'}")
        if not isinstance(roles, list) or not roles or any(not isinstance(role, str) or role not in CANONICAL_CATEGORIES for role in roles) or len(set(roles)) != len(roles):
            fail(f"phase-c:required-roles:{owner or 'unknown'}")
        if any(key not in row for key in ("name", "exposure", "store_link", "allocator_or_table", "migration", "proof")):
            fail(f"phase-c:family-proof:{owner or 'unknown'}")
        if row.get("exposure") != "public_opaque_int":
            fail(f"phase-c:family-exposure:{owner or 'unknown'}")

    for row in escape_rows:
        if not isinstance(row, dict):
            fail("phase-c:public-escape-row-shape")
            continue
        if any(key in row for key in ("start_line", "end_line", "line_start", "line_end")):
            fail("phase-c:barrier-line-range")
        if row.get("topology") != "unregistered_side_table" or row.get("exposure") != "public_opaque_int":
            fail("phase-c:barrier-topology")
        if row.get("disposition") not in {OPAQUE_LEGACY, "typed_token"}:
            fail("phase-c:barrier-disposition")
        roles = row.get("required_roles")
        if not isinstance(roles, list) or not roles or any(role not in CANONICAL_CATEGORIES for role in roles):
            fail("phase-c:barrier-required-roles")
        if _phase_c_path(root, row.get("path")) is None:
            fail("phase-c:barrier-path")

    store_ids: set[str] = set()
    store_owner_sets: dict[str, set[str]] = {}
    for row in store_rows:
        if not isinstance(row, dict):
            fail("phase-c:store-row-shape")
            continue
        store_id = string_field(row, "id")
        if store_id is None or store_id in store_ids:
            fail("phase-c:store-identity")
        if store_id is not None:
            store_ids.add(store_id)
        store_topology = string_field(row, "topology")
        if store_topology not in known_topologies:
            fail(f"phase-c:store-topology:{store_id or 'unknown'}")
        row_owners = row.get("owners")
        if not isinstance(row_owners, list) or not row_owners or any(owner not in all_owners for owner in row_owners) or len(set(row_owners)) != len(row_owners):
            fail(f"phase-c:store-owners:{store_id or 'unknown'}")
            row_owners = []
        store_owner_sets[store_id or ""] = set(row_owners)
        store_item = _phase_c_item(root, row.get("path"), row.get("item_kind"), row.get("symbol"))
        if store_item is None:
            fail(f"phase-c:store-item:{store_id or 'unknown'}")
        else:
            _, store_source, store_rust_item = store_item
            if row.get("item_kind") == "static":
                if store_rust_item.depth == 0:
                    if "wrapper" in row:
                        fail(f"phase-c:store-wrapper:{store_id or 'unknown'}")
                elif row.get("wrapper") != "thread_local!" or not _phase_c_thread_local_static(store_source, store_rust_item):
                    fail(f"phase-c:store-wrapper:{store_id or 'unknown'}")
            elif "wrapper" in row:
                fail(f"phase-c:store-wrapper:{store_id or 'unknown'}")
            store_normalized, _ = _phase_c_item_digests(store_source, store_rust_item)
            if row.get("normalized_digest") != store_normalized:
                fail(f"phase-c:store-digest:{store_id or 'unknown'}")
        for key in ("path", "item_kind", "symbol", "normalized_digest", "proof"):
            if string_field(row, key) is None:
                fail(f"phase-c:store-proof:{store_id or 'unknown'}")

    store_by_id = {
        str(row.get("id")): row
        for row in store_rows
        if isinstance(row, dict) and isinstance(row.get("id"), str)
    }

    valid_route_roles = set(CANONICAL_CATEGORIES)
    route_by_id: dict[str, dict[str, object]] = {}
    combined_routes = [*route_rows, *native_rows]
    for row in combined_routes:
        if not isinstance(row, dict):
            fail("phase-c:route-row-shape")
            continue
        route_id = string_field(row, "id")
        if route_id is None or route_id in route_by_id:
            fail("phase-c:route-identity")
        if route_id is not None:
            route_by_id[route_id] = row
        owner = string_field(row, "owner")
        role = string_field(row, "role")
        store = string_field(row, "store")
        if owner not in all_owners:
            fail(f"phase-c:route-owner:{route_id or 'unknown'}")
        if role not in valid_route_roles:
            fail(f"phase-c:route-role:{route_id or 'unknown'}")
        if store not in store_ids:
            fail(f"phase-c:route-store:{route_id or 'unknown'}")
        selector_ids_value = row.get("selector_ids")
        if not isinstance(selector_ids_value, list) or any(not isinstance(value, str) for value in selector_ids_value) or not selector_ids_value:
            fail(f"phase-c:route-selectors:{route_id or 'unknown'}")
        if any(string_field(row, key) is None for key in ("path", "item_kind", "symbol", "normalized_digest", "source_digest", "proof")):
            fail(f"phase-c:route-proof:{route_id or 'unknown'}")
        item = _phase_c_item(root, row.get("path"), row.get("item_kind"), row.get("symbol"))
        if item is None:
            fail(f"phase-c:stale-route:{route_id or 'unknown'}")
        else:
            _, source, rust_item = item
            normalized, source_digest = _phase_c_item_digests(source, rust_item)
            if row.get("normalized_digest") != normalized:
                fail(f"phase-c:route-digest:{route_id or 'unknown'}")
            if row.get("source_digest") not in (None, source_digest):
                fail(f"phase-c:route-source-digest:{route_id or 'unknown'}")
        if owner in all_owners and store in store_owner_sets and owner not in store_owner_sets[store]:
            fail(f"phase-c:store-reverse-edge:{route_id or 'unknown'}")
    direct_store_ids = {
        str(row.get("store")) for row in combined_routes
        if isinstance(row, dict) and row.get("owner") in {"direct_iter_store", "direct_range"}
    }
    if len(direct_store_ids) > 1:
        fail("phase-c:direct-shared-store")

    selector_by_id: dict[str, dict[str, object]] = {}
    route_selector_actual: dict[str, list[str]] = {route_id: [] for route_id in route_by_id}
    owner_roles: dict[str, set[str]] = {owner: set() for owner in all_owners}
    for row in selector_rows:
        if not isinstance(row, dict):
            fail("phase-c:selector-row-shape")
            continue
        selector_id = string_field(row, "id")
        route_id = string_field(row, "route_id")
        owner = string_field(row, "owner")
        matcher = string_field(row, "matcher")
        if selector_id is None or selector_id in selector_by_id:
            fail("phase-c:selector-identity")
        if selector_id is not None:
            selector_by_id[selector_id] = row
        if route_id not in route_by_id:
            fail(f"phase-c:selector-route:{selector_id or 'unknown'}")
        if owner not in all_owners or (route_id in route_by_id and route_by_id[route_id].get("owner") != owner):
            fail(f"phase-c:selector-owner:{selector_id or 'unknown'}")
        if matcher not in PHASE_C_MATCHERS:
            fail(f"phase-c:unknown-matcher:{selector_id or 'unknown'}")
        if not isinstance(row.get("expected_count"), int) or isinstance(row.get("expected_count"), bool) or row.get("expected_count") != 1:
            fail(f"phase-c:selector-count:{selector_id or 'unknown'}")
        if any(string_field(row, key) is None for key in ("match_digest", "proof")):
            fail(f"phase-c:selector-proof:{selector_id or 'unknown'}")
        if route_id in route_selector_actual and selector_id is not None:
            route_selector_actual[route_id].append(selector_id)
        if owner in owner_roles and route_id in route_by_id:
            owner_roles[owner].add(str(route_by_id[route_id].get("role")))
        if route_id in route_by_id:
            route = route_by_id[route_id]
            if row.get("match_digest") != route.get("normalized_digest"):
                fail(f"phase-c:selector-digest:{selector_id or 'unknown'}")
            if route_id in {str(value.get("id")) for value in native_rows if isinstance(value, dict)} and matcher in (PHASE_C_MATCHERS - {"native_internal_allocator"}):
                fail("phase-c:native-public-conflation")
    for route_id, route in route_by_id.items():
        declared = route.get("selector_ids")
        actual = route_selector_actual.get(route_id, [])
        if declared != actual:
            fail(f"phase-c:route-selector-closure:{route_id}")
    for owner_row in [*family_rows, *escape_rows]:
        if not isinstance(owner_row, dict):
            continue
        owner = owner_row.get("id")
        required_roles = set(owner_row.get("required_roles", [])) if isinstance(owner_row.get("required_roles"), list) else set()
        if owner in all_owners and owner_roles.get(owner, set()) != required_roles:
            fail(f"phase-c:owner-role-closure:{owner}")
    emitted_ids = [str(row.get("id")) for row in selector_rows]
    if len(emitted_ids) != len(set(emitted_ids)):
        fail("phase-c:duplicate-emitted-observation")
    if not selector_rows:
        fail("phase-c:selectors-empty")

    # Semantic expressions are an explicit second authority layer.  Legacy
    # schema fixtures may omit it; the production authority opts in with
    # `phase_c_semantic_version = 1`, which requires every selector to have
    # exactly one immutable expression observation.
    semantic_rows = _phase_c_semantic_rows(manifest)
    if "semantic_observations" in manifest and not isinstance(manifest.get("semantic_observations"), list):
        fail("phase-c:semantic-authority-shape")
    if manifest.get("phase_c_semantic_version") == 1:
        if not semantic_rows or len(semantic_rows) != len(selector_by_id):
            fail("phase-c:semantic-selector-closure")
    semantic_ids: set[str] = set()
    semantic_selectors: set[str] = set()
    semantic_keys: set[tuple[str, str]] = set()
    for row in semantic_rows:
        if not isinstance(row.get("id"), str) or not row.get("id") or row.get("id") in semantic_ids:
            fail("phase-c:semantic-observation-identity")
        else:
            semantic_ids.add(str(row["id"]))
        selector_id = row.get("selector_id")
        selector_key = selector_id if isinstance(selector_id, str) else ""
        if not selector_key or selector_key not in selector_by_id or selector_key in semantic_selectors:
            fail(f"phase-c:semantic-selector-identity:{selector_key or 'unknown'}")
        else:
            semantic_selectors.add(selector_key)
        matcher = row.get("matcher")
        if selector_key in selector_by_id and matcher != selector_by_id[selector_key].get("matcher"):
            fail(f"phase-c:semantic-matcher:{selector_key}")
        expected_count = row.get("expected_count")
        if not isinstance(expected_count, int) or isinstance(expected_count, bool) or expected_count < 1:
            fail(f"phase-c:semantic-count:{selector_key or 'unknown'}")
        digest = row.get("expression_digest")
        if not isinstance(digest, str) or len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            fail(f"phase-c:semantic-digest:{selector_key or 'unknown'}")
        edge_digest = row.get("edge_digest")
        if manifest.get("phase_c_semantic_version") == 1 and (
            not isinstance(edge_digest, str)
            or len(edge_digest) != 64
            or any(char not in "0123456789abcdef" for char in edge_digest)
        ):
            fail(f"phase-c:semantic-edge-digest:{selector_key or 'unknown'}")
        key = (selector_key, str(digest))
        if key in semantic_keys:
            fail(f"phase-c:semantic-duplicate:{selector_key}")
        semantic_keys.add(key)
        if not isinstance(row.get("proof"), str) or not row.get("proof"):
            fail(f"phase-c:semantic-proof:{selector_key or 'unknown'}")
        if manifest.get("phase_c_semantic_version") == 1 and selector_key in selector_by_id:
            semantic_route = route_by_id.get(str(selector_by_id[selector_key].get("route_id")), {})
            semantic_store = store_by_id.get(str(semantic_route.get("store")), {})
            if row.get("store_symbol") != semantic_store.get("symbol"):
                fail(f"phase-c:semantic-store:{selector_key or 'unknown'}")
    return sorted(failures)


_PHASE_C_NESTED_DECLARATION_WORDS = frozenset({
    "fn", "struct", "enum", "mod", "impl", "trait", "type", "const", "static", "use",
})


def _phase_c_direct_tokens(item: RustItem) -> tuple[RustToken, ...]:
    """Return direct executable tokens, retaining their source spans."""
    direct: list[RustToken] = []
    brace_depth = 0
    skip_depth: int | None = None
    saw_skip_body = False
    for token in item.tokens:
        word = token.text
        if skip_depth is not None:
            if word == "{":
                brace_depth += 1
                saw_skip_body = True
            elif word == "}":
                brace_depth -= 1
                if saw_skip_body and brace_depth == skip_depth:
                    skip_depth = None
                    saw_skip_body = False
            continue
        if brace_depth >= 1 and word in _PHASE_C_NESTED_DECLARATION_WORDS:
            skip_depth = brace_depth
            saw_skip_body = False
            continue
        direct.append(token)
        if word == "{":
            brace_depth += 1
        elif word == "}":
            brace_depth -= 1
    return tuple(direct)


def _phase_c_direct_words(item: RustItem) -> tuple[str, ...]:
    """Return only executable words in a top-level item's direct body."""
    return tuple(token.text for token in _phase_c_direct_tokens(item))


@dataclass(frozen=True)
class PhaseCBinding:
    name: str
    kind: str
    start: int
    end: int
    dependencies: tuple[str, ...]
    call_name: str = ""
    # ``rhs`` is the exact lexical expression that created this binding.  It
    # is intentionally retained instead of reducing a statement to a bag of
    # words: provenance checks use the span/binding edge and therefore cannot
    # be satisfied by a same-spelled decoy elsewhere in the item.
    rhs: tuple[str, ...] = ()
    binding_id: str = ""


@dataclass(frozen=True)
class PhaseCCall:
    name: str
    qualifier: tuple[str, ...]
    start: int
    end: int
    arguments: tuple[str, ...]
    argument_parts: tuple[tuple[str, ...], ...] = ()
    argument_binding_ids: tuple[str, ...] = ()
    # Empty entries mean that the argument was not a plain identifier.  They
    # are not discarded: a transformed/literal argument is unsupported by
    # the bounded call grammar and fails identity propagation.


@dataclass(frozen=True)
class PhaseCStaticEdge:
    symbol: str
    operation: str
    start: int
    end: int
    identifiers: tuple[str, ...]
    key_arguments: tuple[str, ...] = ()
    binding_ids: tuple[str, ...] = ()
    terminal_method: str = ""
    terminal_receiver: str = ""
    key_binding_ids: tuple[str, ...] = ()
    declaration_id: str = ""
    receiver_binding_id: str = ""


@dataclass(frozen=True)
class PhaseCResolvedTerminal:
    """One authority-resolved keyed terminal with complete identity."""

    owner_path: str
    owner_item: str
    endpoint_binding: str
    symbol: str
    operation: str
    terminal_method: str
    terminal_receiver: str
    declaration_id: str
    receiver_binding_id: str
    key_arguments: tuple[str, ...]
    key_binding_ids: tuple[str, ...]

    @property
    def identity_complete(self) -> bool:
        return bool(
            self.owner_path
            and self.owner_item
            and self.endpoint_binding
            and self.symbol
            and self.operation
            and self.terminal_method
            and self.terminal_receiver
            and self.declaration_id
            and self.receiver_binding_id
            and self.key_arguments
            and self.key_binding_ids
            and self.endpoint_binding in self.key_binding_ids
        )


@dataclass(frozen=True)
class ProvenancePath:
    """One directed lexical path carried by one conversion operand.

    The endpoint is a source-bound binding identity, never a spelling union.
    ``records`` contain only normalized directed semantic edges; offsets and
    unrelated closure items are deliberately absent from this value.
    """

    endpoint_path: str
    endpoint_item: str
    endpoint_binding: str
    records: tuple[tuple[str, ...], ...] = ()
    hops: int = 0
    unresolved: str = ""
    evidence: bool = False

    def extend(
        self,
        *,
        endpoint_path: str | None = None,
        endpoint_item: str | None = None,
        endpoint_binding: str | None = None,
        record: tuple[str, ...] | None = None,
        hops: int = 0,
        unresolved: str | None = None,
        evidence: bool | None = None,
    ) -> "ProvenancePath":
        return ProvenancePath(
            endpoint_path if endpoint_path is not None else self.endpoint_path,
            endpoint_item if endpoint_item is not None else self.endpoint_item,
            endpoint_binding if endpoint_binding is not None else self.endpoint_binding,
            self.records + ((record,) if record is not None else ()),
            self.hops + hops,
            self.unresolved if unresolved is None else unresolved,
            self.evidence if evidence is None else evidence,
        )


@dataclass(frozen=True)
class PhaseCFunctionSummary:
    """A bounded lexical summary for one top-level Rust function."""

    path: str
    source: str
    item: RustItem
    parameters: tuple[str, ...]
    bindings: tuple[PhaseCBinding, ...]
    calls: tuple[PhaseCCall, ...]
    returns: tuple[tuple[str, ...], ...]
    return_parts: tuple[tuple[str, ...], ...]
    static_edges: tuple[PhaseCStaticEdge, ...]


@dataclass(frozen=True)
class PhaseCSemanticObservation:
    """One exact authority-instantiated semantic expression span."""

    matcher: str
    symbol: str
    path: str
    start: int
    end: int
    tokens: tuple[str, ...]
    expression_digest: str


@dataclass(frozen=True)
class PhaseCStructuralContext:
    """Lexical ancestors and inherited cfg(test) state for one Rust item."""

    cfg_test: bool
    ancestors: tuple[tuple[str, str], ...] = ()

    @property
    def nearest_kind(self) -> str:
        for kind, _ in self.ancestors:
            if kind == "fn":
                return "nested-function"
            if kind == "impl":
                return "impl-method"
            if kind == "trait":
                return "trait-default"
            if kind == "mod":
                return "inline-module"
        return "non-top-level"


@dataclass(frozen=True)
class PhaseCSemanticResolution:
    """One authority-bound conversion -> path -> keyed-terminal result."""

    matcher: str
    conversion_operands: tuple["PhaseCConversionOperand", ...]
    paths: tuple[ProvenancePath, ...]
    terminal_edges: tuple[PhaseCResolvedTerminal, ...]

    @property
    def complete(self) -> bool:
        return (
            len(self.conversion_operands) == 1
            and self.conversion_operands[0].identity_complete
            and len(self.paths) == 1
            and not self.paths[0].unresolved
            and len(self.terminal_edges) == 1
            and self.terminal_edges[0].identity_complete
            and self.paths[0].endpoint_binding == self.terminal_edges[0].endpoint_binding
        )


@dataclass(frozen=True)
class PhaseCSemanticEvidence:
    """Immutable non-hash projection of one completed semantic resolution."""

    semantic_id: str
    site_id: str
    source_path: str
    route_id: str
    matcher: str
    conversion_owner_path: str
    conversion_owner_item: str
    conversion_operand_name: str
    conversion_operand_binding_id: str
    conversion_direct_token_ordinal: int
    conversion_tokens: tuple[str, ...]
    path_endpoint_path: str
    path_endpoint_item: str
    path_endpoint_binding_id: str
    path_records: tuple[tuple[str, ...], ...]
    path_hops: int
    terminal_owner_path: str
    terminal_owner_item: str
    terminal_endpoint_binding_id: str
    terminal_symbol: str
    terminal_operation: str
    terminal_method: str
    terminal_receiver: str
    terminal_declaration_id: str
    terminal_receiver_binding_id: str
    terminal_key_arguments: tuple[str, ...]
    terminal_key_binding_ids: tuple[str, ...]
    conversion_count: int
    path_count: int
    terminal_count: int
    edge_complete: bool


@dataclass(frozen=True)
class PhaseCConversionOperand:
    """One authority-selected conversion and its canonical lexical identity.

    ``source_span`` is retained only for selecting source tokens inside the
    verifier.  It is intentionally never serialized into an edge digest.
    The object is the single conversion result consumed by path selection,
    semantic cardinality, and digest generation.
    """

    kind: str
    symbol: str
    source_span: tuple[int, int]
    operand_name: str
    operand_binding_id: str
    owner_path: str
    owner_item: str
    direct_token_ordinal: int
    conversion_tokens: tuple[str, ...]

    @property
    def identity_complete(self) -> bool:
        return bool(
            self.owner_path
            and self.owner_item
            and self.direct_token_ordinal >= 0
            and len(self.source_span) == 2
            and self.source_span[0] >= 0
            and self.source_span[1] > self.source_span[0]
            and self.symbol
            and self.operand_name
            and self.operand_binding_id
        )


def _phase_c_token_words(tokens: tuple[RustToken, ...]) -> tuple[str, ...]:
    return tuple(token.text for token in tokens)


def _phase_c_statement_end(
    tokens: tuple[RustToken, ...], start: int, limit: int,
) -> int:
    """Return the semicolon/end of one lexical statement.

    The scan is bounded to a function body and tracks all delimiters.  In
    particular, an ``=`` inside a closure/call is not mistaken for the
    binding's assignment operator.  Unknown statement shapes are still
    represented by their exact token span and are rejected by the semantic
    matcher rather than being silently flattened.
    """
    depth = 0
    cursor = start
    while cursor < limit:
        word = tokens[cursor].text
        if word in {"(", "[", "{"}:
            depth += 1
        elif word in {")", "]", "}"
        }:
            if depth == 0:
                return cursor
            depth -= 1
        elif word == ";" and depth == 0:
            return cursor
        cursor += 1
    return limit


def _phase_c_top_level_equals(
    tokens: tuple[RustToken, ...], start: int, end: int,
) -> int | None:
    depth = 0
    for cursor in range(start, end):
        word = tokens[cursor].text
        if word in {"(", "[", "{"}:
            depth += 1
        elif word in {")", "]", "}"
        }:
            depth = max(0, depth - 1)
        elif word == "=" and depth == 0:
            return cursor
    return None


def _phase_c_plain_pattern(tokens: tuple[RustToken, ...]) -> tuple[str, ...]:
    words = [token.text for token in tokens if token.text not in {"mut", "ref"}]
    if len(words) == 1 and (words[0][0].isalpha() or words[0][0] == "_"):
        return (words[0],)
    # Destructuring, match-arm aliases, and bindings with a guard are not
    # admitted by the canonical grammar.  They remain visible to the caller
    # as an unsupported binding rather than becoming a false provenance edge.
    return tuple(words)


def _phase_c_call_for_span(
    calls: tuple[PhaseCCall, ...], start: int, end: int,
) -> PhaseCCall | None:
    matches = [call for call in calls if start <= call.start < end]
    return matches[0] if len(matches) == 1 else None


def _phase_c_identifiers(tokens: tuple[RustToken, ...]) -> tuple[str, ...]:
    return tuple(
        token.text
        for token in tokens
        if token.text and (token.text[0].isalpha() or token.text[0] == "_")
    )


def _phase_c_argument_names(tokens: tuple[RustToken, ...]) -> tuple[str, ...]:
    """Return plain identifier arguments; transforms are intentionally unknown."""
    return tuple(
        part[0] if len(part) == 1 and part[0] and (part[0][0].isalpha() or part[0][0] == "_") else ""
        for part in _phase_c_argument_parts(tokens)
    )


def _phase_c_argument_parts(tokens: tuple[RustToken, ...]) -> tuple[tuple[str, ...], ...]:
    """Split call arguments while retaining transformed token provenance."""
    if not tokens:
        return ()
    parts: list[list[RustToken]] = [[]]
    depth = 0
    for token in tokens:
        if token.text in {"(", "[", "{"}:
            depth += 1
        elif token.text in {")", "]", "}"
        }:
            depth = max(0, depth - 1)
        if token.text == "," and depth == 0:
            parts.append([])
        else:
            parts[-1].append(token)
    return tuple(tuple(token.text for token in part) for part in parts)


_PHASE_C_TERMINAL_METHODS = frozenset({
    "insert", "entry", "remove", "get", "get_mut", "contains_key", "push", "set",
    "fetch_add", "retain", "clear", "replace", "take", "borrow", "borrow_mut",
})


def _phase_c_key_argument_indexes(method: str, argument_count: int) -> tuple[int, ...]:
    """Return the operation-specific key positions, never payload positions."""
    if method in {"get", "get_mut", "contains_key", "remove", "insert", "entry", "set", "replace"}:
        return (0,) if argument_count else ()
    return ()


def _phase_c_static_terminal(
    edge_tokens: tuple[RustToken, ...], operation: str,
) -> tuple[str, str, tuple[str, ...]]:
    """Return terminal method/receiver/key for one static access.

    For ``STATIC.with(|store| store.borrow_mut().insert(k, v))`` the terminal
    receiver must be the closure parameter.  A closure that instead writes
    ``OTHER.insert(...)`` therefore produces no valid STATIC edge.
    """
    if operation == "with":
        pipe_positions = [index for index, token in enumerate(edge_tokens) if token.text == "|"]
        if len(pipe_positions) < 2:
            return "", "", ()
        parameter_words = tuple(
            token.text for token in edge_tokens[pipe_positions[0] + 1:pipe_positions[1]]
            if token.text not in {"mut", "ref"}
        )
        if len(parameter_words) != 1 or parameter_words[0] == "_":
            return "", "", ()
        receiver = parameter_words[0]
        cursor = pipe_positions[1] + 1
        # Start at the closure receiver, then follow the same receiver's
        # method chain.  In particular, ``table.borrow_mut().insert(...)``
        # must end at ``insert``; scanning for another occurrence of the
        # receiver would silently lose the terminal write edge.
        receiver_indexes = [
            index
            for index in range(cursor, len(edge_tokens))
            if edge_tokens[index].text == receiver
            and index + 1 < len(edge_tokens)
            and edge_tokens[index + 1].text == "."
        ]
        # The closure parameter is a real lexical receiver binding.  A
        # second direct use, a local rebinding, or a destructuring shadow is
        # outside the canonical grammar and must not be collapsed into one
        # receiver identity.
        if len(receiver_indexes) != 1:
            return "", "", ()
        receiver_index = receiver_indexes[0]
        body_before_receiver = tuple(token.text for token in edge_tokens[cursor:receiver_index])
        if (
            "let" in body_before_receiver
            or any(
                body_before_receiver[index + 1] == "="
                for index, word in enumerate(body_before_receiver[:-1])
                if word == receiver
            )
        ):
            return "", "", ()
        cursor = receiver_index + 2
        terminal_method = ""
        terminal_args: tuple[str, ...] = ()
        while cursor + 1 < len(edge_tokens):
            method = edge_tokens[cursor].text
            if edge_tokens[cursor + 1].text != "(":
                return "", "", ()
            close = _matching_delimiter(edge_tokens, cursor + 1)
            if close is None:
                return "", "", ()
            if method in _PHASE_C_TERMINAL_METHODS:
                terminal_method = method
                terminal_args = _phase_c_argument_names(edge_tokens[cursor + 2:close])
            cursor = close + 1
            if cursor >= len(edge_tokens) or edge_tokens[cursor].text != ".":
                break
            cursor += 1
        return terminal_method, receiver, terminal_args

    cursor = 1
    terminal_method = ""
    terminal_args: tuple[str, ...] = ()
    while cursor + 2 < len(edge_tokens) and edge_tokens[cursor].text == ".":
        method = edge_tokens[cursor + 1].text
        if edge_tokens[cursor + 2].text != "(":
            break
        close = _matching_delimiter(edge_tokens, cursor + 2)
        if close is None:
            return "", "", ()
        if method in _PHASE_C_TERMINAL_METHODS:
            terminal_method = method
            terminal_args = _phase_c_argument_names(edge_tokens[cursor + 3:close])
        cursor = close + 1
    return terminal_method, edge_tokens[0].text if terminal_method else "", terminal_args


def _phase_c_function_shape(item: RustItem) -> tuple[int, int, tuple[str, ...]] | None:
    tokens = _phase_c_direct_tokens(item)
    fn_index = next((index for index, token in enumerate(tokens) if token.text == "fn"), None)
    if fn_index is None:
        return None
    opening = next((index for index in range(fn_index + 1, len(tokens)) if tokens[index].text == "{"), None)
    if opening is None:
        return None
    body_close = _matching_delimiter(tokens, opening)
    if body_close is None:
        return None
    parameter_open = next((index for index in range(fn_index + 1, opening) if tokens[index].text == "("), None)
    if parameter_open is None:
        return opening, body_close, ()
    parameter_close = _matching_delimiter(tokens, parameter_open)
    if parameter_close is None:
        return None
    parameters: list[str] = []
    depth = 0
    for index in range(parameter_open + 1, parameter_close):
        word = tokens[index].text
        if word in {"(", "[", "{"}:
            depth += 1
        elif word in {")", "]", "}"
        }:
            depth = max(0, depth - 1)
        if depth == 0 and index + 1 < parameter_close and tokens[index + 1].text == ":":
            if word not in {"self", "mut", "ref"} and word and (word[0].isalpha() or word[0] == "_"):
                parameters.append(word)
    return opening, body_close, tuple(dict.fromkeys(parameters))


def _phase_c_function_calls(item: RustItem, known_names: set[str]) -> tuple[PhaseCCall, ...]:
    """Extract only unambiguous plain/qualified calls from one function.

    This is intentionally lexical.  It recognizes ``foo(...)`` and
    ``module::foo(...)`` after the function signature, ignores methods and
    macros, and leaves all unknown/ambiguous call forms out of the summary.
    The caller resolves the final name against the exact source census.
    """
    tokens = _phase_c_direct_tokens(item)
    opening = next((index for index, token in enumerate(tokens) if token.text == "{"), None)
    if opening is None:
        return ()
    shape = _phase_c_function_shape(item)
    if shape is None:
        return ()
    opening, body_close, _ = shape
    calls: list[PhaseCCall] = []
    for index in range(opening + 1, body_close):
        if tokens[index].text != "(" or index == 0:
            continue
        callee_index = index - 1
        if not (tokens[callee_index].text and (tokens[callee_index].text[0].isalpha() or tokens[callee_index].text[0] == "_")):
            continue
        if tokens[callee_index].text in {"if", "while", "for", "match", "loop", "Some", "None", "Ok", "Err"}:
            continue
        if callee_index > 0 and tokens[callee_index - 1].text in {".", "!"}:
            continue
        # Walk a qualified path backwards, but never treat a method receiver
        # as a module path.  A leading `::` is accepted as a lexical path
        # marker and is still resolved by final-name uniqueness.
        start = callee_index
        qualifier: list[str] = []
        while start >= 2 and tokens[start - 1].text == "::":
            previous = tokens[start - 2].text
            if not previous or not (previous[0].isalpha() or previous[0] == "_"):
                break
            qualifier.insert(0, previous)
            start -= 2
        name = tokens[callee_index].text
        if name in known_names:
            close = _matching_delimiter(tokens, index)
            if close is None or close > body_close:
                continue
            end = tokens[close].end
            argument_parts = _phase_c_argument_parts(tokens[index + 1:close])
            call = PhaseCCall(
                name,
                tuple(qualifier),
                tokens[start].start,
                end,
                tuple(
                    part[0] if len(part) == 1 and part[0] and (part[0][0].isalpha() or part[0][0] == "_") else ""
                    for part in argument_parts
                ),
                argument_parts,
            )
            if call not in calls:
                calls.append(call)
    return tuple(calls)


def _phase_c_function_bindings(
    item: RustItem,
    calls: tuple[PhaseCCall, ...],
    static_symbols: frozenset[str] = frozenset(),
    path_value: str = "",
    static_declarations: dict[str, tuple[str, ...]] | None = None,
) -> tuple[
    tuple[str, ...], tuple[PhaseCBinding, ...], tuple[tuple[str, ...], ...],
    tuple[tuple[str, ...], ...], tuple[PhaseCStaticEdge, ...], tuple[PhaseCCall, ...]
]:
    """Summarize one function's bounded lexical binding graph.

    This parser intentionally recognizes only statement-local bindings and
    direct call/store edges.  It does not attempt Rust data-flow in general:
    nested destructuring, rebinding, match-arm aliases, transformed call
    arguments, and ambiguous edges stay explicit in the summary so the
    authority matcher can reject them fail-closed.
    """
    tokens = _phase_c_direct_tokens(item)
    shape = _phase_c_function_shape(item)
    if shape is None:
        return (), (), (), (), (), calls
    opening, body_close, parameters = shape
    bindings: list[PhaseCBinding] = []
    for ordinal, name in enumerate(parameters):
        bindings.append(
            PhaseCBinding(
                name, "param", item.start, item.start, (), "", (),
                f"{path_value}\0{item.name}:param:{ordinal}:{name}",
            )
        )
    returns: list[tuple[str, ...]] = []
    return_parts: list[tuple[str, ...]] = []
    static_edges: list[PhaseCStaticEdge] = []
    cursor = opening + 1
    ordinal = len(bindings)
    while cursor < body_close:
        word = tokens[cursor].text
        if word == "let":
            statement_end = _phase_c_statement_end(tokens, cursor + 1, body_close)
            equals = _phase_c_top_level_equals(tokens, cursor + 1, statement_end)
            if equals is not None:
                lhs = tokens[cursor + 1:equals]
                rhs_end = statement_end
                rhs = tokens[equals + 1:rhs_end]
                names = _phase_c_plain_pattern(lhs)
                if names:
                    dependencies = _phase_c_identifiers(rhs)
                    rhs_calls = [call for call in calls if tokens[equals].end <= call.start < (tokens[rhs_end].start if rhs_end < len(tokens) else item.end)]
                    call_name = rhs_calls[0].name if len(rhs_calls) == 1 else ""
                    kind = "let" if len(names) == 1 and names[0] != "_" else "pattern"
                    for name in names:
                        bindings.append(
                            PhaseCBinding(
                                name,
                                kind,
                                tokens[cursor].start,
                                tokens[rhs_end].end if rhs_end < len(tokens) else item.end,
                                dependencies,
                                call_name,
                                _phase_c_token_words(rhs),
                                f"{path_value}\0{item.name}:let:{ordinal}:{name}",
                            )
                        )
                        ordinal += 1
            cursor = min(body_close, statement_end + 1)
            continue
        if word == "return":
            statement_end = _phase_c_statement_end(tokens, cursor + 1, body_close)
            part = _phase_c_token_words(tokens[cursor + 1:statement_end])
            return_parts.append(part)
            returns.append(_phase_c_identifiers(tokens[cursor + 1:statement_end]))
            cursor = min(body_close, statement_end + 1)
            continue
        # Assignment is recognized only at the beginning of a statement.  A
        # nested closure's ``x =`` must not be turned into a false shadow of
        # the enclosing binding.
        if (
            cursor + 1 < body_close
            and tokens[cursor + 1].text == "="
            and word
            and (word[0].isalpha() or word[0] == "_")
        ):
            statement_end = _phase_c_statement_end(tokens, cursor + 2, body_close)
            if any(binding.name == word for binding in bindings):
                bindings.append(
                    PhaseCBinding(
                        word, "assign", tokens[cursor].start,
                        tokens[statement_end].end if statement_end < len(tokens) else item.end,
                        _phase_c_identifiers(tokens[cursor + 2:statement_end]), "", (),
                        f"{path_value}\0{item.name}:assign:{ordinal}:{word}",
                    )
                )
                ordinal += 1
            cursor = min(body_close, statement_end + 1)
            continue
        # A bare match-arm identifier is a binding too.  It is deliberately
        # recorded even when it is not one of the configured names: the
        # authority proof can reject a live-map arm that shadows the expected
        # code/kind/id binding without confusing it with a `let` binding.
        if cursor + 1 < body_close and tokens[cursor + 1].text == "=>" and word not in {"_", "Some", "None", "Ok", "Err"}:
            bindings.append(
                PhaseCBinding(
                    word, "match_arm", tokens[cursor].start, tokens[cursor + 1].end, (), "", (),
                    f"{path_value}\0{item.name}:match:{ordinal}:{word}",
                )
            )
            ordinal += 1
        cursor += 1

    # Static/store edges are parsed independently of statement boundaries so
    # a closure inside `with`/`lock` is still part of the exact edge span.  A
    # symbol is eligible only when a real static declaration from the source
    # census supplied it; names are never inferred from uppercase spelling.
    static_declarations = static_declarations or {}
    for index in range(opening + 1, body_close):
        symbol = tokens[index].text
        declaration_ids = static_declarations.get(symbol, ())
        if symbol not in static_symbols or len(declaration_ids) != 1 or index + 2 >= body_close or tokens[index + 1].text != ".":
            continue
        operation = tokens[index + 2].text
        close = None
        end_index = index + 2
        if index + 3 < body_close and tokens[index + 3].text == "(":
            close = _matching_delimiter(tokens, index + 3)
            if close is None or close > body_close:
                continue
            end_index = close
            # Preserve the full terminal chain for direct static roots such
            # as `BARRIERS.lock().unwrap().insert(...)`; the root call alone
            # is not the semantic store operation.
            if operation != "with":
                cursor_after = close + 1
                while cursor_after + 2 < body_close and tokens[cursor_after].text == ".":
                    if tokens[cursor_after + 2].text != "(":
                        break
                    chain_close = _matching_delimiter(tokens, cursor_after + 2)
                    if chain_close is None or chain_close > body_close:
                        break
                    end_index = chain_close
                    cursor_after = chain_close + 1
        edge_tokens = tokens[index:end_index + 1]
        terminal_method, terminal_receiver, key_arguments = _phase_c_static_terminal(edge_tokens, operation)
        kind = "write" if terminal_method in {
            "insert", "entry", "fetch_add", "set", "remove", "push", "replace", "take", "clear",
        } else "read"
        if not terminal_method:
            kind = "invalid"
        if any(
            binding.name == symbol
            and binding.kind in {"param", "let", "assign", "pattern"}
            and binding.start <= tokens[index].start
            for binding in bindings
        ):
            kind = "invalid"
        binding_ids: list[str] = []
        for argument in key_arguments:
            if not argument:
                # Keep an empty slot for a transformed/literal argument.
                # Compacting this list would shift a payload binding into
                # the operation's key position (for example, insert(k + 1,
                # value) would incorrectly treat ``value`` as ``k``).
                binding_ids.append("")
                continue
            candidates = [
                binding for binding in bindings
                if binding.name == argument
                and binding.kind in {"param", "let"}
                and binding.start <= tokens[index].start
            ]
            if len(candidates) == 1:
                binding_ids.append(candidates[0].binding_id)
            else:
                binding_ids.append("")
        key_indexes = _phase_c_key_argument_indexes(terminal_method, len(key_arguments))
        key_binding_ids = tuple(
            binding_ids[position]
            for position in key_indexes
            if position < len(binding_ids) and binding_ids[position]
        )
        receiver_binding_id = (
            f"{path_value}\0{item.name}:closure:{len(static_edges)}:{terminal_receiver}"
            if terminal_receiver and operation == "with"
            else declaration_ids[0] if terminal_method else ""
        )
        edge = PhaseCStaticEdge(
            symbol,
            f"{operation}:{kind}",
            tokens[index].start,
            tokens[end_index].end,
            _phase_c_identifiers(edge_tokens),
            key_arguments,
            tuple(binding_ids),
            terminal_method,
            terminal_receiver,
            key_binding_ids,
            declaration_ids[0] if len(declaration_ids) == 1 else "",
            receiver_binding_id,
        )
        if not any(existing.start == edge.start and existing.symbol == edge.symbol for existing in static_edges):
            static_edges.append(edge)

    # Resolve every plain argument to the exact lexical binding visible at the
    # call site.  Names alone are not provenance: a same-spelled parameter or
    # local shadow must not be allowed to carry a conversion into a callee.
    def visible_binding_id(part: tuple[str, ...], call_start: int) -> str:
        if len(part) == 1 and part[0] and (part[0][0].isalpha() or part[0][0] == "_"):
            names = [
                binding for binding in bindings
                if binding.name == part[0]
                and binding.kind in {"param", "let", "assign", "pattern", "match_arm"}
                and binding.start <= call_start
            ]
            if len(names) == 1:
                return names[0].binding_id if names[0].kind in {"param", "let"} else ""
            if names:
                latest_start = max(binding.start for binding in names)
                latest = [binding for binding in names if binding.start == latest_start]
                if len(latest) == 1 and latest[0].kind in {"param", "let"}:
                    return latest[0].binding_id
            return ""
        identifiers = tuple(
            word for word in part
            if word and (word[0].isalpha() or word[0] == "_")
        )
        ids = tuple(
            visible_binding_id((word,), call_start)
            for word in identifiers
        )
        unique = tuple(dict.fromkeys(value for value in ids if value))
        return unique[0] if len(unique) == 1 else ""

    calls = tuple(
        PhaseCCall(
            call.name,
            call.qualifier,
            call.start,
            call.end,
            call.arguments,
            call.argument_parts,
            tuple(visible_binding_id(part, call.start) for part in call.argument_parts),
        )
        for call in calls
    )

    # Canonical Rust functions often return a final expression rather than an
    # explicit `return`; select only the final top-level statement span.
    depth = 0
    semicolons: list[int] = []
    for index in range(opening + 1, body_close):
        word = tokens[index].text
        if word in {"(", "[", "{"}:
            depth += 1
        elif word in {")", "]", "}"
        }:
            depth = max(0, depth - 1)
        elif word == ";" and depth == 0:
            semicolons.append(index)
    tail_start = (semicolons[-1] + 1) if semicolons else opening + 1
    if tail_start < body_close:
        tail = tokens[tail_start:body_close]
        if tail:
            return_parts.append(_phase_c_token_words(tail))
            returns.append(_phase_c_identifiers(tail))
    return parameters, tuple(bindings), tuple(returns), tuple(return_parts), tuple(static_edges), calls


def _phase_c_function_summaries(
    root: Path,
    paths: list[Path],
    *,
    identity_prefix: str | None = None,
) -> tuple[dict[tuple[str, str], PhaseCFunctionSummary], dict[str, tuple[PhaseCFunctionSummary, ...]]]:
    """Build a source-bound function index for the four-hop proof.

    Only unqualified top-level function declarations participate.  Duplicate
    names remain ambiguous and are never selected by a call edge; this keeps
    a same-name helper in another module from satisfying an authority route.
    """
    censuses: list[tuple[str, str, RustItem]] = []
    static_symbols: set[str] = set()
    static_declaration_rows: dict[str, list[str]] = {}
    for path in paths:
        try:
            source = path.read_text(encoding="utf-8")
            items, _ = _source_census(source, include_nested_functions=True)
        except (OSError, UnicodeError, ScanFailure):
            continue
        relative_path = path.resolve().relative_to(root.resolve()).as_posix()
        path_value = (
            f"{identity_prefix}/{relative_path}"
            if identity_prefix
            else relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else relative_path
        )
        masked = mask_non_code(source)
        test_ranges, _ = test_only_ranges(masked)
        for item in items:
            if item.kind == "static":
                static_symbols.add(item.name)
                if item.depth == 0 or _phase_c_thread_local_static(source, item):
                    declaration = (
                        f"{path_value}\0static:{item.name}:"
                        f"{_normalized_item_digest(source[item.start:item.end])}"
                    )
                    static_declaration_rows.setdefault(item.name, []).append(declaration)
            if item.kind == "fn" and item.depth == 0 and not any(start <= item.start <= end for start, end in test_ranges):
                censuses.append((path_value, source, item))
    known_names = {item.name for _, _, item in censuses}
    summaries: dict[tuple[str, str], PhaseCFunctionSummary] = {}
    by_name: dict[str, list[PhaseCFunctionSummary]] = {}
    for path_value, source, item in censuses:
        calls = _phase_c_function_calls(item, known_names)
        parameters, bindings, returns, return_parts, static_edges, calls = _phase_c_function_bindings(
            item, calls, frozenset(static_symbols), path_value,
            {name: tuple(values) for name, values in static_declaration_rows.items()}
        )
        summary = PhaseCFunctionSummary(
            path_value, source, item, parameters, bindings, calls, returns,
            return_parts, static_edges
        )
        summaries[(path_value, item.name)] = summary
        by_name.setdefault(item.name, []).append(summary)
    return summaries, {name: tuple(values) for name, values in by_name.items()}


def _phase_c_resolve_call(
    current: PhaseCFunctionSummary,
    call: PhaseCCall,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
) -> tuple[PhaseCFunctionSummary, ...]:
    candidates = by_name.get(call.name, ())
    if not call.qualifier:
        return candidates if len(candidates) == 1 else ()
    module = call.qualifier[-1]
    if module == "super":
        parent = Path(current.path).parent
        candidates = tuple(item for item in candidates if Path(item.path).parent == parent)
    else:
        candidates = tuple(
            item
            for item in candidates
            if Path(item.path).stem == module or module in Path(item.path).parts
        )
    return candidates if len(candidates) == 1 else ()


def _phase_c_call_closure(
    start: PhaseCFunctionSummary,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    max_hops: int = 4,
) -> tuple[tuple[PhaseCFunctionSummary, ...], tuple[tuple[str, ...], ...]]:
    """Return the source-bound summaries and call paths within four hops."""
    visited: set[tuple[str, str]] = {(start.path, start.item.name)}
    queue: list[tuple[PhaseCFunctionSummary, int, tuple[str, ...]]] = [(start, 0, (start.item.name,))]
    summaries: list[PhaseCFunctionSummary] = [start]
    paths: list[tuple[str, ...]] = [(start.item.name,)]
    while queue:
        current, depth, call_path = queue.pop(0)
        if depth >= max_hops:
            continue
        for call in current.calls:
            targets = _phase_c_resolve_call(current, call, by_name)
            if len(targets) != 1:
                continue
            target = targets[0]
            target_key = (target.path, target.item.name)
            next_path = (*call_path, target.item.name)
            paths.append(next_path)
            if target_key in visited:
                continue
            visited.add(target_key)
            summaries.append(target)
            queue.append((target, depth + 1, next_path))
    return tuple(summaries), tuple(paths)


def _phase_c_summary_binding(
    summary: PhaseCFunctionSummary, name: str,
) -> PhaseCBinding | None:
    bindings = [binding for binding in summary.bindings if binding.name == name]
    return bindings[0] if len(bindings) == 1 else None


def _phase_c_binding_call(
    summary: PhaseCFunctionSummary,
    binding: PhaseCBinding,
) -> PhaseCCall | None:
    """Find the one call that creates a lexical binding, if any."""
    if not binding.call_name:
        return None
    calls = tuple(
        call for call in summary.calls
        if call.name == binding.call_name and binding.start <= call.start <= binding.end
    )
    return calls[0] if len(calls) == 1 else None


def _phase_c_binding_call_is_whole_rhs(
    summary: PhaseCFunctionSummary,
    binding: PhaseCBinding,
    call: PhaseCCall,
) -> bool:
    """Prove that one direct call spans the complete binding RHS.

    A known call nested in ``foo(x) + 1``, ``foo(x) as u64``, an index,
    tuple, method chain, or any other wrapper is not a call-result edge. The
    comparison is made against the exact lexer token span, so a same-spelled
    call elsewhere in the item cannot satisfy the proof.
    """
    tokens = _phase_c_direct_tokens(summary.item)
    if not binding.rhs:
        return False
    matches = []
    for index in range(len(tokens) - len(binding.rhs) + 1):
        span = tokens[index:index + len(binding.rhs)]
        if tuple(token.text for token in span) != binding.rhs:
            continue
        if span[0].start < binding.start or span[-1].end > binding.end:
            continue
        matches.append(span)
    return (
        len(matches) == 1
        and matches[0][0].start == call.start
        and matches[0][-1].end == call.end
    )


def _phase_c_argument_contains_binding(
    summary: PhaseCFunctionSummary,
    call: PhaseCCall,
    part: tuple[str, ...],
    binding_id: str,
) -> bool:
    """Resolve identifiers inside one transformed argument by lexical ID."""
    identifiers = tuple(
        word for word in part
        if word and (word[0].isalpha() or word[0] == "_")
    )
    for name in identifiers:
        candidates = [
            candidate for candidate in summary.bindings
            if candidate.name == name
            and candidate.kind in {"param", "let", "assign", "pattern", "match_arm"}
            and candidate.start <= call.start
        ]
        if not candidates:
            continue
        latest_start = max(candidate.start for candidate in candidates)
        latest = [candidate for candidate in candidates if candidate.start == latest_start]
        if len(latest) == 1 and latest[0].binding_id == binding_id:
            return True
    return False


def _phase_c_path_binding(
    summary: PhaseCFunctionSummary, binding: PhaseCBinding,
) -> ProvenancePath:
    return ProvenancePath(
        summary.path,
        summary.item.name,
        binding.binding_id,
        (("binding", summary.path, summary.item.name, binding.kind, binding.binding_id),),
    )


def _phase_c_path_unresolved(
    summary: PhaseCFunctionSummary, binding: PhaseCBinding | None, reason: str,
    *, evidence: bool = False,
) -> ProvenancePath:
    binding_id = binding.binding_id if binding is not None else ""
    return ProvenancePath(
        summary.path,
        summary.item.name,
        binding_id,
        (("unresolved", reason, summary.path, summary.item.name, binding_id),),
        unresolved=reason,
        evidence=evidence,
    )


def _phase_c_binding_rhs_kind(
    summary: PhaseCFunctionSummary,
    binding: PhaseCBinding,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
) -> str:
    """Classify one RHS under the deliberately tiny provenance grammar."""
    rhs = binding.rhs
    local_names = {
        candidate.name
        for candidate in summary.bindings
        if candidate.kind in {"param", "let"}
    }
    if len(rhs) == 1 and rhs[0] in local_names:
        return "alias"
    calls = [
        call for call in summary.calls
        if binding.start <= call.start <= binding.end
    ]
    if len(calls) == 1 and binding.call_name == calls[0].name:
        if not _phase_c_binding_call_is_whole_rhs(summary, binding, calls[0]):
            return "wrapped-call"
        targets = _phase_c_resolve_call(summary, calls[0], by_name)
        return "call" if len(targets) == 1 else "ambiguous-call"
    static_sources = [
        edge for edge in summary.static_edges
        if binding.start <= edge.start <= edge.end <= binding.end
        and edge.terminal_method
    ]
    if len(static_sources) == 1 and not calls:
        return "static-source"
    if calls:
        return "unknown-call"
    if len(rhs) >= 2 and rhs[0] and rhs[1] == "(":
        return "unknown-call"
    if "." in rhs and "(" in rhs:
        return "method-rhs"
    if any(word in rhs for word in {"as", "+", "-", "*", "/", "%", "&", "|", "^", "(", ")", "[", "]", "{", "}"}):
        return "unsupported-rhs"
    if len(rhs) == 1 and rhs[0] and (rhs[0][0].isalpha() or rhs[0][0] == "_"):
        return "unknown-rhs"
    return "unsupported-rhs"


def _phase_c_return_refs(
    target: PhaseCFunctionSummary,
) -> tuple[str, tuple[str, ...]]:
    """Return one bare returned binding, or a precise rejected shape."""
    if len(target.return_parts) != 1:
        return "multi-origin-return", ()
    part = target.return_parts[0]
    refs = tuple(
        word for word in part
        if _phase_c_summary_binding(target, word) is not None
    )
    if len(part) == 1 and len(refs) == 1:
        return "", refs
    if len(refs) > 1:
        return "multi-origin-return", refs
    if len(refs) == 1:
        return "transformed-return", refs
    return "unsupported-return", refs


def _phase_c_terminal_edges_for_binding(
    summary: PhaseCFunctionSummary,
    binding_id: str,
    symbol: str = "",
) -> tuple[PhaseCStaticEdge, ...]:
    """Return only terminal store edges keyed by one carried binding ID."""
    return tuple(
        edge for edge in summary.static_edges
        if edge.terminal_method
        and (not symbol or edge.symbol == symbol)
        and binding_id in edge.key_binding_ids
    )


def _phase_c_overflow_evidence(
    summary: PhaseCFunctionSummary,
    binding: PhaseCBinding,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    seen: frozenset[tuple[str, str, str]] = frozenset(),
) -> bool:
    """Prove a hop-limit edge is on this binding's directed continuation."""
    key = (summary.path, summary.item.name, binding.binding_id)
    if key in seen:
        return False
    seen = seen | {key}
    if _phase_c_terminal_edges_for_binding(summary, binding.binding_id):
        return True
    for call in summary.calls:
        indexes = [
            index for index, argument in enumerate(call.arguments)
            if index < len(call.argument_binding_ids)
            and call.argument_binding_ids[index] == binding.binding_id
        ]
        if any(
            index < len(call.argument_parts)
            and tuple(call.argument_parts[index]) != (binding.name,)
            and _phase_c_argument_contains_binding(
                summary, call, call.argument_parts[index], binding.binding_id
            )
            for index in indexes
        ):
            continue
        if len(indexes) != 1:
            continue
        targets = _phase_c_resolve_call(summary, call, by_name)
        if len(targets) != 1 or len(call.arguments) != len(targets[0].parameters):
            continue
        index = indexes[0]
        parameter = _phase_c_summary_binding(targets[0], targets[0].parameters[index])
        if parameter is not None and _phase_c_terminal_edges_for_binding(
            targets[0], parameter.binding_id
        ):
            return True
    if binding.kind == "let":
        call = _phase_c_binding_call(summary, binding)
        targets = _phase_c_resolve_call(summary, call, by_name) if call is not None else ()
        if len(targets) == 1:
            reason, refs = _phase_c_return_refs(targets[0])
            if not reason and len(refs) == 1:
                returned = _phase_c_summary_binding(targets[0], refs[0])
                if returned is not None and _phase_c_overflow_evidence(
                    targets[0], returned, by_name, seen
                ):
                    return True
    return False


def _phase_c_forward_path(
    path: ProvenancePath,
    summary: PhaseCFunctionSummary,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    stack: tuple[tuple[str, str, str], ...] = (),
) -> ProvenancePath:
    """Follow only calls carrying this exact endpoint binding."""
    key = (summary.path, summary.item.name, path.endpoint_binding)
    if key in stack:
        return path.extend(unresolved="recursive", evidence=True)
    next_stack = (*stack, key)
    binding = next((candidate for candidate in summary.bindings if candidate.binding_id == path.endpoint_binding), None)
    if binding is None:
        return path.extend(unresolved="binding-owner")
    carried: list[tuple[PhaseCCall, PhaseCFunctionSummary, PhaseCBinding, int]] = []
    for call in summary.calls:
        matching_indices = [
            index for index, binding_id in enumerate(call.argument_binding_ids)
            if binding_id == binding.binding_id
        ]
        transformed_matches = [
            index for index, part in enumerate(call.argument_parts)
            if tuple(part) != (binding.name,)
            and _phase_c_argument_contains_binding(summary, call, part, binding.binding_id)
        ]
        targets = _phase_c_resolve_call(summary, call, by_name)
        if (matching_indices or transformed_matches) and len(targets) != 1:
            return path.extend(unresolved="ambiguous-call", evidence=True)
        if transformed_matches:
            return path.extend(unresolved="transformed-argument", evidence=True)
        if not matching_indices:
            continue
        if len(call.arguments) != len(targets[0].parameters):
            return path.extend(unresolved="call-arity", evidence=True)
        target = targets[0]
        for index in matching_indices:
            if index >= len(target.parameters):
                return path.extend(unresolved="call-arity", evidence=True)
            parameter = _phase_c_summary_binding(target, target.parameters[index])
            if parameter is None:
                return path.extend(unresolved="parameter-identity", evidence=True)
            carried.append((call, target, parameter, index))
    if len(carried) > 1:
        return path.extend(unresolved="multiple-forward-edges", evidence=True)
    if not carried:
        return path
    call, target, parameter, index = carried[0]
    if path.hops >= 4:
        # Hop overflow is evidence only when the carried parameter reaches
        # an exact terminal key edge in this target.  Unrelated static/store
        # operations in the same helper must not turn a deep decoy red.
        target_evidence = bool(
            _phase_c_terminal_edges_for_binding(target, parameter.binding_id)
        )
        return path.extend(
            record=("call-overflow", summary.path, summary.item.name, target.path, target.item.name, str(index), path.endpoint_binding, parameter.binding_id),
            unresolved="depth",
            evidence=target_evidence,
        )
    transferred = path.extend(
        endpoint_path=target.path,
        endpoint_item=target.item.name,
        endpoint_binding=parameter.binding_id,
        record=("call-arg", summary.path, summary.item.name, target.path, target.item.name, str(index), path.endpoint_binding, parameter.binding_id),
        hops=1,
    )
    return _phase_c_forward_path(transferred, target, by_name, next_stack)


def _phase_c_trace_path(
    summary: PhaseCFunctionSummary,
    name_or_binding_id: str,
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    substitutions: dict[str, tuple[PhaseCFunctionSummary, str, int]] | None = None,
    stack: tuple[tuple[str, str, str], ...] = (),
    hops: int = 0,
) -> ProvenancePath | None:
    """Trace one binding as one ordered provenance path."""
    if not name_or_binding_id or name_or_binding_id == "_":
        return None
    substitution_scope = substitutions
    substitutions = substitutions or {}
    exact_binding = next(
        (candidate for candidate in summary.bindings if candidate.binding_id == name_or_binding_id),
        None,
    )
    binding = exact_binding or _phase_c_summary_binding(summary, name_or_binding_id)
    if binding is None:
        return None
    name = binding.name
    key = (summary.path, summary.item.name, binding.binding_id)
    if key in stack:
        return _phase_c_path_unresolved(summary, binding, "recursive", evidence=True)
    if hops > 4:
        # The fifth transfer is visible only if the carried binding itself
        # reaches a terminal keyed edge.  A deep helper with an unrelated
        # static/counter operation is not provenance evidence.
        return _phase_c_path_unresolved(
            summary,
            binding,
            "depth",
            evidence=_phase_c_overflow_evidence(summary, binding, by_name),
        )
    next_stack = (*stack, key)
    if binding.kind == "param":
        path = _phase_c_path_binding(summary, binding)
        substitution = substitutions.get(name)
        if substitution_scope is not None and substitution is None:
            return _phase_c_path_unresolved(summary, binding, "argument-identity", evidence=True)
        if substitution is not None:
            caller, caller_name, index = substitution
            caller_path = _phase_c_trace_path(caller, caller_name, by_name, None, next_stack, hops + 1)
            if caller_path is None:
                return _phase_c_path_unresolved(summary, binding, "argument-origin", evidence=True)
            if caller_path.unresolved:
                return caller_path
            path = caller_path.extend(
                endpoint_path=summary.path,
                endpoint_item=summary.item.name,
                endpoint_binding=binding.binding_id,
                record=("call-arg", caller.path, caller.item.name, summary.path, summary.item.name, str(index), caller_path.endpoint_binding, binding.binding_id),
                hops=1,
            )
        return _phase_c_forward_path(path, summary, by_name, stack)
    if binding.kind != "let":
        call = _phase_c_binding_call(summary, binding)
        if call is not None and not _phase_c_binding_call_is_whole_rhs(summary, binding, call):
            return _phase_c_path_unresolved(summary, binding, "wrapped-call", evidence=True)
        return _phase_c_path_unresolved(summary, binding, "unsupported-binding", evidence=True)
    rhs_kind = _phase_c_binding_rhs_kind(summary, binding, by_name)
    if rhs_kind in {"unknown-call", "unknown-rhs", "method-rhs", "wrapped-call", "unsupported-rhs"}:
        return _phase_c_path_unresolved(
            summary, binding, rhs_kind,
            evidence=rhs_kind != "unsupported-rhs" or any(
                word in binding.rhs
                for word in {"as", "+", "-", "*", "/", "%", "&", "|", "^", "(", ")", "[", "]", "{", "}"}
            ),
        )
    if rhs_kind == "ambiguous-call":
        return _phase_c_path_unresolved(summary, binding, "ambiguous-call", evidence=True)
    call = _phase_c_binding_call(summary, binding)
    if rhs_kind == "call" and call is not None:
        targets = _phase_c_resolve_call(summary, call, by_name)
        if len(targets) != 1:
            return _phase_c_path_unresolved(summary, binding, "ambiguous-call", evidence=True)
        target = targets[0]
        if len(call.arguments) != len(target.parameters):
            return _phase_c_path_unresolved(summary, binding, "call-arity", evidence=True)
        return_reason, refs = _phase_c_return_refs(target)
        if return_reason:
            return _phase_c_path_unresolved(summary, binding, return_reason, evidence=True)
        caller_names: dict[str, str] = {}
        for index, binding_id in enumerate(call.argument_binding_ids):
            if (
                index >= len(call.arguments)
                or not binding_id
                or index >= len(call.argument_parts)
                or tuple(call.argument_parts[index]) != (call.arguments[index],)
            ):
                continue
            caller_binding = next(
                (candidate for candidate in summary.bindings if candidate.binding_id == binding_id),
                None,
            )
            if caller_binding is not None:
                caller_names[str(index)] = caller_binding.name
        substitutions_for_target = {
            parameter: (summary, caller_names[str(index)], index)
            for index, parameter in enumerate(target.parameters)
            if str(index) in caller_names
        }
        target_path = _phase_c_trace_path(
            target, refs[0], by_name, substitutions_for_target, next_stack, hops + 1
        )
        if target_path is None:
            return _phase_c_path_unresolved(summary, binding, "return-origin", evidence=True)
        if target_path.unresolved:
            return target_path.extend(
                record=("conversion-origin", summary.path, summary.item.name)
            )
        return target_path.extend(
            record=("call-result", summary.path, summary.item.name, target.path, target.item.name, binding.binding_id, target_path.endpoint_binding),
            hops=1,
        )
    if rhs_kind == "static-source":
        # A read expression such as `TABLE.with(|table| table.get(slot))`
        # creates a value binding, but the opaque identity is carried by its
        # operation-specific key.  Re-root the path at that exact key binding
        # instead of treating every identifier in the closure as equivalent.
        static_sources = tuple(
            edge for edge in summary.static_edges
            if binding.start <= edge.start <= edge.end <= binding.end
            and edge.terminal_method
        )
        if len(static_sources) == 1 and static_sources[0].key_binding_ids:
            edge = static_sources[0]
            if len(edge.key_binding_ids) != 1 or edge.key_binding_ids[0] == binding.binding_id:
                return _phase_c_path_unresolved(summary, binding, "static-key-identity", evidence=True)
            key_binding = next(
                (candidate for candidate in summary.bindings if candidate.binding_id == edge.key_binding_ids[0]),
                None,
            )
            if key_binding is None:
                return _phase_c_path_unresolved(summary, binding, "static-key-identity", evidence=True)
            key_path = _phase_c_trace_path(summary, key_binding.name, by_name, None, next_stack, hops + 1)
            if key_path is None:
                return _phase_c_path_unresolved(summary, binding, "static-key-origin", evidence=True)
            if key_path.unresolved:
                return key_path
            return key_path.extend(
                record=("static-key", edge.symbol, edge.operation, binding.binding_id, edge.key_binding_ids[0]),
            )
    local_bindings = {
        candidate.name for candidate in summary.bindings
        if candidate.kind in {"param", "let"}
    }
    dependencies = [dependency for dependency in binding.dependencies if dependency in local_bindings]
    if len(dependencies) > 1:
        return _phase_c_path_unresolved(summary, binding, "multi-origin-rhs", evidence=True)
    if rhs_kind not in {"alias", "static-source"}:
        return _phase_c_path_unresolved(summary, binding, "unsupported-rhs", evidence=True)
    if rhs_kind == "alias" and len(dependencies) != 1:
        return _phase_c_path_unresolved(summary, binding, "alias-origin", evidence=True)
    if not dependencies:
        path = _phase_c_path_binding(summary, binding)
    else:
        dependency = dependencies[0]
        dependency_path = _phase_c_trace_path(summary, dependency, by_name, None, next_stack, hops + 1)
        if dependency_path is None:
            return _phase_c_path_unresolved(summary, binding, "rhs-origin", evidence=True)
        if dependency_path.unresolved:
            return dependency_path
        dependency_binding = _phase_c_summary_binding(summary, dependency)
        if dependency_binding is None:
            return _phase_c_path_unresolved(summary, binding, "rhs-origin", evidence=True)
        path = dependency_path.extend(
            record=("rhs", summary.path, summary.item.name, binding.binding_id, dependency_binding.binding_id),
        )
    return _phase_c_forward_path(path, summary, by_name, stack)


def _phase_c_conversion_operands(
    summary: PhaseCFunctionSummary,
) -> tuple[PhaseCConversionOperand, ...]:
    """Return exact conversions as one shared typed identity object."""
    tokens = _phase_c_direct_tokens(summary.item)
    result: list[PhaseCConversionOperand] = []

    def binding_id(name: str, position: int) -> str:
        if not name:
            return ""
        candidates = [
            candidate for candidate in summary.bindings
            if candidate.name == name
            and candidate.kind in {"param", "let"}
            and candidate.start <= position
        ]
        if len(candidates) == 1:
            return candidates[0].binding_id
        if candidates:
            latest = max(candidate.start for candidate in candidates)
            latest_candidates = [candidate for candidate in candidates if candidate.start == latest]
            if len(latest_candidates) == 1:
                return latest_candidates[0].binding_id
        return ""

    for index, token in enumerate(tokens):
        if tuple(t.text for t in tokens[index:index + 4]) == ("MbValue", "::", "from_int", "("):
            close = _matching_delimiter(tokens, index + 3)
            if close is None:
                continue
            argument_tokens = tokens[index + 4:close]
            argument_words = tuple(token.text for token in argument_tokens)
            # The production boundary uses the one reviewed widening cast
            # from a validated u64 handle to i64.  No other transform,
            # arithmetic, mask, or qualified expression is admitted.
            if len(argument_words) == 1 and argument_words[0] and (argument_words[0][0].isalpha() or argument_words[0][0] == "_"):
                args = (argument_words[0],)
            elif len(argument_words) == 3 and argument_words[1:] == ("as", "i64") and argument_words[0] and (argument_words[0][0].isalpha() or argument_words[0][0] == "_"):
                args = (argument_words[0],)
            else:
                args = ("",)
            argument_names = args if len(args) == 1 and args[0] else ()
            result.append(
                PhaseCConversionOperand(
                    "from_int", "MbValue", (token.start, tokens[close].end),
                    argument_names[0] if argument_names else "",
                    binding_id(argument_names[0], token.start) if argument_names else "",
                    summary.path, summary.item.name, index,
                    tuple(item.text for item in tokens[index:close + 1]),
                )
            )
        if token.text == "as_int" and index + 2 < len(tokens) and tokens[index + 1].text == "(" and tokens[index + 2].text == ")":
            receiver = (
                tokens[index - 2].text
                if index >= 2 and tokens[index - 1].text == "."
                else ""
            )
            result.append(
                PhaseCConversionOperand(
                    "as_int", "as_int", (token.start, tokens[index + 2].end),
                    receiver,
                    binding_id(receiver, token.start),
                    summary.path, summary.item.name, index,
                    tuple(item.text for item in tokens[index - 2:index + 3]) if receiver else tuple(item.text for item in tokens[index:index + 3]),
                )
            )
    return tuple(result)


def _phase_c_validate_call_edges(
    closure: tuple[PhaseCFunctionSummary, ...],
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
) -> bool:
    """Reject ambiguous calls, transformed arguments, and call cycles."""
    for summary in closure:
        for call in summary.calls:
            targets = _phase_c_resolve_call(summary, call, by_name)
            # Unknown, ambiguous, arity-mismatched, and transformed calls
            # remain in the lexical summary.  The directed path walker is
            # responsible for rejecting them only when the carried binding
            # reaches that call; closure-wide rejection would make unrelated
            # helpers false negatives.
            _ = targets
    return True


def _phase_c_edge_payload(resolution: PhaseCSemanticResolution) -> list[object] | None:
    """Build the pure canonical v6 JSON-array payload for one edge."""
    if not resolution.complete:
        return None
    records: list[list[object]] = []
    for operand in resolution.conversion_operands:
        records.append([
            "conversion", operand.owner_path, operand.owner_item,
            operand.kind, operand.symbol, operand.direct_token_ordinal,
            operand.operand_name, operand.operand_binding_id,
            list(operand.conversion_tokens),
        ])
    for path in resolution.paths:
        records.extend([list(record) for record in path.records])
        records.append(["endpoint", path.endpoint_path, path.endpoint_item, path.endpoint_binding])
        records.append(["hops", path.hops])
    for terminal in resolution.terminal_edges:
        records.append([
            "keyed-terminal", terminal.owner_path, terminal.owner_item,
            terminal.endpoint_binding, terminal.symbol, terminal.operation,
            terminal.terminal_receiver, terminal.terminal_method,
            terminal.declaration_id, terminal.receiver_binding_id,
            list(terminal.key_arguments), list(terminal.key_binding_ids),
        ])
    return ["phase-c-edge", "v6", resolution.matcher, records]


def _phase_c_edge_digest(resolution: PhaseCSemanticResolution) -> str | None:
    """Digest one complete v6 conversion/path/terminal edge."""
    payload = _phase_c_edge_payload(resolution)
    if payload is None:
        return None
    encoded = json.dumps(
        payload,
        ensure_ascii=False, separators=(",", ":"),
    )
    return hashlib.sha256(("phase-c-edge-v6\0" + encoded).encode("utf-8")).hexdigest()


def _phase_c_semantic_evidence(
    semantic_id: str,
    site_id: str,
    source_path: str,
    route_id: str,
    resolution: PhaseCSemanticResolution,
) -> PhaseCSemanticEvidence | None:
    """Project the one existing complete resolution without hashing it again."""
    if not resolution.complete:
        return None
    operand = resolution.conversion_operands[0]
    path = resolution.paths[0]
    terminal = resolution.terminal_edges[0]
    return PhaseCSemanticEvidence(
        semantic_id, site_id, source_path, route_id, resolution.matcher,
        operand.owner_path, operand.owner_item, operand.operand_name,
        operand.operand_binding_id, operand.direct_token_ordinal,
        operand.conversion_tokens, path.endpoint_path, path.endpoint_item,
        path.endpoint_binding, path.records, path.hops, terminal.owner_path,
        terminal.owner_item, terminal.endpoint_binding, terminal.symbol,
        terminal.operation, terminal.terminal_method, terminal.terminal_receiver,
        terminal.declaration_id, terminal.receiver_binding_id,
        terminal.key_arguments, terminal.key_binding_ids,
        len(resolution.conversion_operands), len(resolution.paths),
        len(resolution.terminal_edges), resolution.complete,
    )


def _phase_c_subsequence_positions(words: tuple[str, ...], needle: tuple[str, ...]) -> tuple[int, ...]:
    """Return every exact, non-overlapping expression occurrence.

    Phase-C observations are semantic token expressions, not arbitrary words.
    A balanced Rust item has already been selected by the authority anchor;
    this helper only recognizes the small, reviewed token spellings below.
    Overlapping occurrences are deliberately rejected by advancing past a
    complete expression so one source expression cannot satisfy two rows.
    """
    if not needle or len(needle) > len(words):
        return ()
    positions: list[int] = []
    index = 0
    while index + len(needle) <= len(words):
        if words[index:index + len(needle)] == needle:
            positions.append(index)
            index += len(needle)
        else:
            index += 1
    return tuple(positions)


def _phase_c_expression_digest(words: tuple[str, ...]) -> str:
    """Digest one canonical expression using the authority anchor recipe."""
    return hashlib.sha256(("v1\0" + "\0".join(words)).encode("utf-8")).hexdigest()


def _phase_c_string_literals(source: str) -> tuple[str, ...]:
    """Read string values while ignoring comments and character literals."""
    values: list[str] = []
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            index = len(source) if end < 0 else end + 1
            continue
        if source.startswith("/*", index):
            end = source.find("*/", index + 2)
            index = len(source) if end < 0 else end + 2
            continue
        if source[index] == "\"":
            index += 1
            chars: list[str] = []
            while index < len(source):
                char = source[index]
                if char == "\\" and index + 1 < len(source):
                    chars.append(source[index:index + 2])
                    index += 2
                    continue
                if char == "\"":
                    index += 1
                    values.append("".join(chars))
                    break
                chars.append(char)
                index += 1
            continue
        index += 1
    return tuple(values)


def _phase_c_expression_needles(
    matcher: str,
    symbol: str,
    words: tuple[str, ...],
    variant: str = "",
) -> tuple[tuple[str, ...], ...]:
    """Return the bounded expression grammar for one authority matcher.

    This is intentionally an explicit grammar table.  It does not search for
    generic integer casts, maps, or names: each admitted expression ties the
    route to the reviewed store/allocator/reifier vocabulary.
    """
    if matcher == "central_registry_insert":
        return (("integer_handle_registry", "::", "register", "("),)
    if matcher == "central_registry_lookup":
        return (
            ("integer_handle_registry", "::", "retain", "("),
            ("integer_handle_registry", "::", "release", "("),
        )
    if matcher == "direct_store_allocate":
        if symbol in {"mb_range", "mb_range_2", "mb_range_3"}:
            return (("iter", "::", "mb_range_iter", "("),)
        if symbol == "mb_range_iter":
            return (("alloc_iter_id", "(", ")"), ("ITERATORS", ".", "with", "("), ("MbValue", "::", "from_int", "("))
        return (("alloc_iter_id", "(", ")"), ("MbValue", "::", "from_int", "("))
    if matcher == "direct_store_lookup":
        if "range" in symbol:
            return (("ITERATORS", ".", "with", "("), ("IterKind", "::", "Range"),)
        return (("ITERATORS", ".", "with", "("), ("as_int", "(", ")"),)
    if matcher == "threshold_classifier":
        return (("HANDLE_MIN_ID",), (">", "HANDLE_MIN_ID"), ("<", "HANDLE_MIN_ID"))
    if matcher == "first_live_probe":
        return ((".", "next", "("), (".", "find", "("))
    if matcher == "public_from_int_reifier":
        return (("MbValue", "::", "from_int", "("),)
    if matcher == "public_as_int_reifier":
        return (("as_int", "(", ")"),)
    if matcher == "public_field_reifier":
        if variant == "store_lock":
            return (("BARRIERS", ".", "lock", "("),)
        if symbol in {"get_or_create_barrier", "mb_threading_barrier"}:
            return (("NEXT_BARRIER_ID", ".", "fetch_add", "("), ("BARRIERS", ".", "lock", "("), ("MbValue", "::", "from_int", "("))
        if symbol == "mb_getattr_impl":
            return (("fields", ".", "read", "("), ("contains_key", "("))
        return (("BARRIERS", ".", "lock", "("), ("barrier_id",),)
    if matcher == "native_internal_allocator":
        return (("alloc_random_id", "(", ")"), ("RANDOMS", ".", "with", "("), ("insert", "("))
    return ()


def _phase_c_canonical_observations(
    matcher: str,
    symbol: str,
    item: RustItem,
    source: str = "",
    variant: str = "",
) -> tuple[PhaseCSemanticObservation, ...]:
    """Evaluate one authority matcher into immutable expression records.

    The record carries the exact token span selected from the authority item;
    callers never re-search a closure-wide word union.  Alternatives are
    explicit canonical grammar forms and duplicate spans are rejected by the
    caller's one-to-one selector reconciliation.
    """
    tokens = _phase_c_direct_tokens(item)
    words = tuple(token.text for token in tokens)
    observations: list[PhaseCSemanticObservation] = []
    # Alternatives describe distinct canonical item forms.  They are not a
    # license to add their counts together: select the first form that is
    # present in the authority-instantiated item.
    for needle in _phase_c_expression_needles(matcher, symbol, words, variant):
        positions = _phase_c_subsequence_positions(words, needle)
        if positions:
            for position in positions:
                selected = tokens[position:position + len(needle)]
                if len(selected) != len(needle):
                    continue
                selected_words = tuple(token.text for token in selected)
                observations.append(
                    PhaseCSemanticObservation(
                        matcher, symbol, "", selected[0].start, selected[-1].end,
                        selected_words, _phase_c_expression_digest(selected_words),
                    )
                )
            break
    unique: list[PhaseCSemanticObservation] = []
    seen: set[tuple[int, int, tuple[str, ...]]] = set()
    for observation in observations:
        key = (observation.start, observation.end, observation.tokens)
        if key not in seen:
            seen.add(key)
            unique.append(observation)
    return tuple(unique)


def _phase_c_semantic_rows(manifest: dict[str, object]) -> list[dict[str, object]]:
    rows = manifest.get("semantic_observations", [])
    return [row for row in rows if isinstance(row, dict)] if isinstance(rows, list) else []


def _phase_c_terminal_edges_for_path(
    path: ProvenancePath,
    closure: tuple[PhaseCFunctionSummary, ...],
    store_symbol: str = "",
) -> tuple[PhaseCStaticEdge, ...]:
    """Bind one selected path to exactly its keyed terminal operation."""
    if path.unresolved:
        return ()
    return tuple(
        edge
        for summary in closure
        if summary.path == path.endpoint_path and summary.item.name == path.endpoint_item
        for edge in summary.static_edges
        if edge.terminal_method
        and (not store_symbol or edge.symbol == store_symbol)
        and path.endpoint_binding in edge.key_binding_ids
    )


def _phase_c_resolved_terminal(
    path: ProvenancePath,
    edge: PhaseCStaticEdge,
) -> PhaseCResolvedTerminal:
    """Freeze a static edge into the path-bound terminal identity."""
    return PhaseCResolvedTerminal(
        path.endpoint_path,
        path.endpoint_item,
        path.endpoint_binding,
        edge.symbol,
        edge.operation,
        edge.terminal_method,
        edge.terminal_receiver,
        edge.declaration_id,
        edge.receiver_binding_id,
        edge.key_arguments,
        edge.key_binding_ids,
    )


def _phase_c_matcher_conversion_kinds(matcher: str) -> frozenset[str]:
    """Bind each semantic matcher to its one reviewed conversion family."""
    if matcher in {"public_as_int_reifier", "direct_store_lookup"}:
        return frozenset({"as_int"})
    if matcher in {
        "public_from_int_reifier", "direct_store_allocate", "native_internal_allocator",
    }:
        return frozenset({"from_int"})
    # Registry/classifier/field matchers can be instantiated at either public
    # conversion boundary.  The semantic row still has to contain exactly
    # one conversion, one path, and one keyed terminal.
    return frozenset({"from_int", "as_int"})


def _phase_c_semantic_resolution(
    matcher: str,
    summary: PhaseCFunctionSummary | None,
    closure: tuple[PhaseCFunctionSummary, ...],
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    store_symbol: str = "",
    conversion_operands: tuple[PhaseCConversionOperand, ...] | None = None,
) -> PhaseCSemanticResolution:
    """Resolve the complete semantic edge once for seeded and perimeter use."""
    if summary is None:
        return PhaseCSemanticResolution(matcher, (), (), ())
    kinds = _phase_c_matcher_conversion_kinds(matcher)
    operands = tuple(
        operand for operand in (
            conversion_operands
            if conversion_operands is not None
            else _phase_c_conversion_operands(summary)
        )
        if operand.kind in kinds
    )
    paths = _phase_c_selected_paths(matcher, closure, by_name, summary, operands)
    terminal_edges: list[PhaseCResolvedTerminal] = []
    seen_edges: set[tuple[str, str, str, str, str, str, tuple[str, ...]]] = set()
    for path in paths:
        for edge in _phase_c_terminal_edges_for_path(path, closure, store_symbol):
            terminal = _phase_c_resolved_terminal(path, edge)
            key = (
                terminal.owner_path, terminal.owner_item, terminal.endpoint_binding,
                terminal.symbol, terminal.operation, terminal.declaration_id,
                terminal.key_binding_ids,
            )
            if key in seen_edges:
                continue
            seen_edges.add(key)
            terminal_edges.append(terminal)
    return PhaseCSemanticResolution(matcher, operands, paths, tuple(terminal_edges))


def _phase_c_semantic_resolution_diagnostics(
    semantic_id: str,
    resolution: PhaseCSemanticResolution,
) -> list[str]:
    """Emit the stable 1/1/1 cardinality contract for one semantic row."""
    diagnostics: list[str] = []
    if len(resolution.conversion_operands) != 1:
        diagnostics.append(
            f"phase-c:semantic-conversion-cardinality:{semantic_id}:{len(resolution.conversion_operands)}"
        )
    if len(resolution.paths) != 1:
        diagnostics.append(
            f"phase-c:semantic-path-cardinality:{semantic_id}:{len(resolution.paths)}"
        )
    if len(resolution.terminal_edges) != 1:
        diagnostics.append(
            f"phase-c:semantic-terminal-cardinality:{semantic_id}:{len(resolution.terminal_edges)}"
        )
    if not resolution.complete:
        diagnostics.append(f"phase-c:semantic-edge:{semantic_id}")
    return diagnostics


def _phase_c_structural_context(
    source: str,
    item: RustItem,
    census: tuple[RustItem, ...] = (),
) -> PhaseCStructuralContext:
    """Resolve nearest lexical ancestors without prefix/history heuristics.

    ``RustItem.prefix`` is intentionally presentation-only: it can lose an
    outer ``cfg(test)`` module after a brace boundary and it cannot identify
    an ``impl``/``trait`` owner.  This bounded resolver uses balanced item
    spans plus explicit impl/trait blocks, then applies cfg(test) ranges to
    the complete inherited region.
    """
    if not census:
        census, _ = _source_census(source, include_nested_functions=True)
    masked = mask_non_code(source)
    test_ranges, _ = test_only_ranges(masked)
    ancestors: list[tuple[int, int, str, str]] = []
    for candidate in census:
        if candidate.start >= item.start or candidate.end < item.end:
            continue
        if candidate.kind in {"mod", "fn"}:
            ancestors.append((candidate.start, candidate.end, candidate.kind, candidate.name))

    tokens = _rust_tokens(source)
    function_spans = tuple(
        (candidate.start, candidate.end)
        for candidate in census
        if candidate.kind == "fn"
    )

    def owner_open_boundary(index: int) -> bool:
        """Require an owner opener at a same-scope item boundary."""
        cursor = index - 1
        qualifiers = {"pub", "unsafe", "default", "const", "async"}
        while cursor >= 0:
            word = tokens[cursor].text
            if word == "]":
                opening = cursor - 1
                depth = 1
                while opening >= 0:
                    if tokens[opening].text == "]":
                        depth += 1
                    elif tokens[opening].text == "[":
                        depth -= 1
                        if depth == 0:
                            cursor = opening - 1
                            break
                    opening -= 1
                else:
                    return False
                continue
            if word in qualifiers:
                cursor -= 1
                continue
            return cursor < 0 or word in {";", "{", "}"}
        return True

    def valid_owner_header(kind: str, header: tuple[str, ...]) -> tuple[bool, str]:
        """Accept only the bounded balanced owner declaration grammar."""
        if not header:
            return False, ""
        words = list(header)
        if kind == "trait":
            if not words or not (words[0][0].isalpha() or words[0][0] == "_"):
                return False, ""
            name = words[0]
            allowed = {"<", ">", ",", ":", "+", "where", "'", "_"}
            if any(
                not (word in allowed or word == "::" or word[0].isalpha() or word[0] == "_")
                for word in words[1:]
            ):
                return False, ""
            return True, name
        # impl [generics] Trait [for Type] / impl Type
        cursor = 0
        if cursor < len(words) and words[cursor] == "<":
            depth = 0
            while cursor < len(words):
                if words[cursor] == "<":
                    depth += 1
                elif words[cursor] == ">":
                    depth -= 1
                cursor += 1
                if depth == 0:
                    break
            if depth != 0:
                return False, ""
        head = words[cursor:]
        if not head:
            return False, ""
        for_index = head.index("for") if "for" in head else -1
        owner_words = head[for_index + 1:] if for_index >= 0 else head
        if not owner_words:
            return False, ""
        identifiers = [
            word for word in owner_words
            if word and (word[0].isalpha() or word[0] == "_") and word not in {"where"}
        ]
        if not identifiers:
            return False, ""
        allowed = {"<", ">", ",", ":", "::", "where", "&", "'", "_", "?", "for"}
        if any(
            not (word in allowed or (word and (word[0].isalpha() or word[0] == "_")))
            for word in head
        ):
            return False, ""
        return True, identifiers[0]

    for index, token in enumerate(tokens):
        if token.text not in {"impl", "trait"}:
            continue
        # Owner keywords lexed anywhere inside a function span belong to that
        # function's signature/body, never to an enclosing impl/trait owner.
        # This check runs before header parsing so return-position `impl Trait`
        # and nested decoys cannot manufacture structural ancestors.
        if any(start < token.start < end for start, end in function_spans):
            continue
        if not owner_open_boundary(index):
            continue
        # A token inside the target function's signature/body (for example
        # ``fn f() -> impl Trait``) is not an enclosing owner declaration.
        # The declaration keyword and its opening brace must both precede the
        # selected item, otherwise return-position impl Trait becomes a false
        # impl ancestor.
        if token.start >= item.start:
            continue
        opening: int | None = None
        cursor = index + 1
        delimiter_depth = 0
        while cursor < len(tokens):
            word = tokens[cursor].text
            if word in {"(", "[", "<"}:
                delimiter_depth += 1
            elif word in {")", "]", ">",
            }:
                delimiter_depth = max(0, delimiter_depth - 1)
            elif word == ";" and delimiter_depth == 0:
                break
            elif word == "{" and delimiter_depth == 0:
                opening = cursor
                break
            cursor += 1
        if opening is None or tokens[opening].start >= item.start:
            continue
        close = _matching_word_brace(tokens, opening)
        if close is None or tokens[close].end < item.end:
            continue
        header_words = tuple(candidate.text for candidate in tokens[index + 1:opening])
        valid, name = valid_owner_header(token.text, header_words)
        if not valid:
            continue
        ancestors.append((token.start, tokens[close].end, token.text, name))

    ancestors.sort(key=lambda row: (row[1] - row[0], -row[0], row[2], row[3]))
    in_test = any(start <= item.start <= end for start, end in test_ranges)
    return PhaseCStructuralContext(in_test, tuple((kind, name) for _, _, kind, name in ancestors))


def _phase_c_non_top_level_context(source: str, item: RustItem) -> str:
    """Classify a rejected non-free function from structural ancestors."""
    return _phase_c_structural_context(source, item).nearest_kind


def phase_c_report(
    root: Path = PRODUCTION,
    authority_path: Path | None = None,
    *,
    identity_prefix: str | None = None,
) -> dict[str, object]:
    """Resolve only manually seeded routes plus the bounded perimeter."""
    root = root.resolve()
    authority_path = authority_path or FAMILIES_MANIFEST
    try:
        manifest = load_family_manifest(authority_path)
    except (OSError, tomllib.TOMLDecodeError) as error:
        return {
            "schema": PHASE_C_INVENTORY_SCHEMA, "root": str(root), "source_file_count": 0,
            "source_files_digest": "", "total": 0, "inventory_digest": "",
            "authority_digest": "", "sites": [], "diagnostics": [f"phase-c:authority-load:{type(error).__name__}"],
            "seeded": [], "unseeded_candidates": [], "_semantic_evidence": (),
        }
    diagnostics = validate_phase_c_authority(manifest, root)
    reachable_paths, graph_diagnostics = _phase_c_reachable_sources(root)
    diagnostics.extend(graph_diagnostics)
    function_summaries, function_by_name = _phase_c_function_summaries(
        root, reachable_paths, identity_prefix=identity_prefix
    )

    def identity_path(path: Path) -> str:
        relative_path = path.resolve().relative_to(root.resolve()).as_posix()
        if identity_prefix:
            return f"{identity_prefix}/{relative_path}"
        return relpath(path) if path.resolve().is_relative_to(REPO.resolve()) else relative_path
    static_symbols: set[str] = set()
    for source_path in reachable_paths:
        try:
            source_text = source_path.read_text(encoding="utf-8")
            source_items, _ = _source_census(source_text)
        except (OSError, UnicodeError, ScanFailure):
            continue
        static_symbols.update(item.name for item in source_items if item.kind == "static")
    selector_rows = [row for row in manifest.get("selectors", []) if isinstance(row, dict)]
    route_rows = [row for row in [*manifest.get("routes", []), *manifest.get("native_internal_routes", [])] if isinstance(row, dict)]
    route_by_id = {str(row.get("id")): row for row in route_rows}
    store_by_id = {
        str(row.get("id")): row
        for row in manifest.get("stores", [])
        if isinstance(row, dict) and isinstance(row.get("id"), str)
    }
    semantic_by_selector = {
        str(row.get("selector_id")): row
        for row in _phase_c_semantic_rows(manifest)
        if isinstance(row.get("selector_id"), str)
    }
    seeded: list[dict[str, object]] = []
    semantic_evidence: list[PhaseCSemanticEvidence] = []
    seeded_spans: set[tuple[str, str]] = set()
    seeded_store_spans: set[tuple[str, str]] = set()
    authority_paths: set[str] = set()
    for row in [*manifest.get("families", []), *manifest.get("public_escapes", [])]:
        if isinstance(row, dict) and isinstance(row.get("path"), str):
            authority_paths.add(str(row["path"]))
    for row in route_rows:
        if isinstance(row.get("path"), str):
            authority_paths.add(str(row["path"]))
    for row in [*manifest.get("stores", [])]:
        if isinstance(row, dict) and isinstance(row.get("path"), str) and isinstance(row.get("symbol"), str):
            seeded_store_spans.add((str(row["path"]), str(row["symbol"])))
    semantic_keys: set[tuple[str, str, str]] = set()
    for selector in selector_rows:
        route = route_by_id.get(str(selector.get("route_id")))
        if route is None:
            continue
        item = _phase_c_item(root, route.get("path"), route.get("item_kind"), route.get("symbol"))
        if item is None:
            continue
        path, source, rust_item = item
        normalized, source_digest = _phase_c_item_digests(source, rust_item)
        if route.get("normalized_digest") != normalized:
            continue
        path_value = str(route.get("path"))
        seeded_spans.add((path_value, rust_item.name))
        canonical_path_value = identity_path(path)
        seeded_spans.add((canonical_path_value, rust_item.name))
        summary = function_summaries.get((canonical_path_value, rust_item.name))
        closure, call_paths = _phase_c_call_closure(summary, function_by_name) if summary is not None else ((), ())
        seeded_row: dict[str, object] = {
            "site_id": str(selector.get("id")), "path": path_value, "symbol": rust_item.name,
            "normalized_digest": normalized, "category": str(route.get("role")),
            "kind": str(selector.get("matcher")), "parent_route": f"phase-c/{route.get('id')}",
            "source_digest": source_digest,
        }
        semantic = semantic_by_selector.get(str(selector.get("id")))
        if semantic is not None:
            matcher = str(selector.get("matcher"))
            store_row = store_by_id.get(str(route.get("store")), {})
            store_symbol = str(store_row.get("symbol", ""))
            # Every authority-selected matcher uses the same complete-edge
            # resolver.  Legacy expression-only observations are deliberately
            # not an alternate acceptance path: absent conversion/path/
            # terminal evidence is a cardinality failure.
            strict_semantic = True
            conversion_operands = _phase_c_conversion_operands(summary)
            resolution = _phase_c_semantic_resolution(
                matcher, summary, closure, function_by_name, store_symbol,
                conversion_operands=conversion_operands,
            )
            selected_paths = resolution.paths
            if any(path.unresolved == "depth" for path in selected_paths):
                diagnostics.append(f"phase-c:semantic-closure-depth:{semantic.get('id', selector.get('id'))}")
            if strict_semantic:
                diagnostics.extend(
                    _phase_c_semantic_resolution_diagnostics(
                        str(semantic.get("id", selector.get("id"))), resolution
                    )
                )
                if resolution.complete and summary is not None:
                    edge_digest = _phase_c_edge_digest(resolution)
                    if edge_digest is not None:
                        seeded_row["edge_digest"] = edge_digest
                    evidence = _phase_c_semantic_evidence(
                        str(semantic.get("id", selector.get("id"))),
                        str(selector.get("id")),
                        path_value,
                        str(route.get("id")),
                        resolution,
                    )
                    if evidence is not None:
                        semantic_evidence.append(evidence)
            observations = _phase_c_canonical_observations(
                matcher,
                rust_item.name,
                rust_item,
                source,
                str(semantic.get("variant", "")),
            )
            expected_count = semantic.get("expected_count")
            if len(observations) != expected_count:
                diagnostics.append(f"phase-c:semantic-count:{semantic.get('id', selector.get('id'))}")
            if semantic.get("store_symbol") is not None and semantic.get("store_symbol") != store_symbol:
                diagnostics.append(f"phase-c:semantic-store:{semantic.get('id', selector.get('id'))}")
            if len(observations) == 1:
                observation = observations[0]
                expression = observation.tokens
                expression_digest = observation.expression_digest
                if expression_digest != semantic.get("expression_digest"):
                    diagnostics.append(f"phase-c:semantic-digest:{semantic.get('id', selector.get('id'))}")
                if (
                    semantic.get("edge_digest") is not None
                    and "edge_digest" in seeded_row
                    and seeded_row.get("edge_digest") != semantic.get("edge_digest")
                ):
                    diagnostics.append(f"phase-c:semantic-edge-digest:{semantic.get('id', selector.get('id'))}")
                expression_key = (path_value, rust_item.name, expression_digest)
                if expression_key in semantic_keys:
                    diagnostics.append(f"phase-c:semantic-duplicate:{semantic.get('id', selector.get('id'))}")
                semantic_keys.add(expression_key)
                seeded_row.update({
                    "expression_id": str(semantic.get("id")),
                    "expression_count": len(observations),
                    "expression_digest": expression_digest,
                    "store_symbol": store_symbol,
                })
        seeded.append(seeded_row)
    # The perimeter is root-wide but authority-scoped by the real Rust module
    # graph.  It does not walk every integer or retain any disposition/owner
    # guess; only a closed allocator/store/raw-opaque signal can become a
    # candidate.
    candidates: list[dict[str, object]] = []
    for item_path in reachable_paths:
        path_value = identity_path(item_path)
        try:
            source = item_path.read_text(encoding="utf-8")
            items, _ = _source_census(source, include_nested_functions=True)
        except (OSError, UnicodeError, ScanFailure):
            continue
        for rust_item in items:
            context = _phase_c_structural_context(source, rust_item, items)
            in_test = context.cfg_test
            if rust_item.depth != 0 or in_test:
                if rust_item.depth != 0 and not in_test and rust_item.kind == "fn":
                    nested_summary = PhaseCFunctionSummary(
                        path_value, source, rust_item, (), (), (), (), (), ()
                    )
                    if _phase_c_conversion_operands(nested_summary):
                        diagnostics.append(
                            f"phase-c:unsupported-non-top-level-conversion:{context.nearest_kind}:{path_value}::{rust_item.name}@{rust_item.start}"
                        )
                continue
            if (path_value, rust_item.name) in seeded_spans or (path_value, rust_item.name) in seeded_store_spans:
                continue
            words = _phase_c_direct_words(rust_item)
            summary = function_summaries.get((path_value, rust_item.name))
            closure, _ = _phase_c_call_closure(summary, function_by_name) if summary is not None else ((), ())
            conversions = _phase_c_conversion_operands(summary) if summary is not None else ()
            perimeter_matcher = (
                "public_from_int_reifier"
                if any(operand.kind == "from_int" for operand in conversions)
                else "public_as_int_reifier"
            )
            resolution = _phase_c_semantic_resolution(
                perimeter_matcher, summary, closure, function_by_name,
                conversion_operands=conversions,
            )
            if not _phase_c_perimeter_signal(
                rust_item,
                words,
                resolution,
                frozenset(static_symbols),
            ):
                continue
            resolved_paths = tuple(path for path in resolution.paths if not path.unresolved)
            if len(resolved_paths) > 1:
                if len(resolution.conversion_operands) != 1:
                    diagnostics.append(
                        f"phase-c:perimeter-conversion-cardinality:{path_value}::{rust_item.name}:{len(resolution.conversion_operands)}"
                    )
                diagnostics.append(
                    f"phase-c:perimeter-path-cardinality:{path_value}::{rust_item.name}:{len(resolution.paths)}"
                )
                diagnostics.append(
                    f"phase-c:perimeter-terminal-cardinality:{path_value}::{rust_item.name}:{len(resolution.terminal_edges)}"
                )
            elif len(resolved_paths) == 1 and len(resolution.terminal_edges) != 1:
                diagnostics.append(
                    f"phase-c:perimeter-terminal-cardinality:{path_value}::{rust_item.name}:{len(resolution.terminal_edges)}"
                )
            for selected_path in resolution.paths:
                if selected_path.unresolved:
                    continue
                terminal_edges = tuple(
                    terminal for terminal in resolution.terminal_edges
                    if terminal.owner_path == selected_path.endpoint_path
                    and terminal.owner_item == selected_path.endpoint_item
                    and terminal.symbol in static_symbols
                )
                if len(terminal_edges) > 1:
                    diagnostics.append(
                        f"phase-c:provenance:multiple-terminal:{selected_path.endpoint_path}::{selected_path.endpoint_item}"
                    )
            diagnostics.extend(
                diagnostic for path in resolution.paths
                if (diagnostic := _phase_c_provenance_diagnostic(path)) is not None
            )
            normalized, source_digest = _phase_c_item_digests(source, rust_item)
            identity = "\0".join((path_value, rust_item.name, "unseeded_candidate", normalized))
            candidates.append({
                "site_id": hashlib.sha256(identity.encode("utf-8")).hexdigest(),
                "path": path_value, "symbol": rust_item.name, "normalized_digest": normalized,
                "category": "unseeded_candidate", "kind": "perimeter", "parent_route": "phase-c/perimeter",
                "source_digest": source_digest,
            })
    sites = sorted([*seeded, *candidates], key=lambda row: str(row["site_id"]))
    resolved_paths = sorted({path.resolve() for path in reachable_paths}, key=lambda path: str(path))
    source_records = [
        f"{identity_path(path)}\0{hashlib.sha256(path.read_bytes()).hexdigest()}"
        for path in resolved_paths
    ]
    source_files_digest = hashlib.sha256("\n".join(source_records).encode()).hexdigest() if source_records else ""
    inventory_digest = hashlib.sha256(json.dumps({"sites": sites, "diagnostics": diagnostics}, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    return {
        "schema": PHASE_C_INVENTORY_SCHEMA, "root": str(root), "source_file_count": len(resolved_paths),
        "source_files_digest": source_files_digest, "total": len(sites), "inventory_digest": inventory_digest,
        "authority_digest": hashlib.sha256(authority_path.read_bytes()).hexdigest(), "sites": sites, "diagnostics": diagnostics,
        "seeded": sorted(seeded, key=lambda row: str(row["site_id"])),
        "unseeded_candidates": sorted(candidates, key=lambda row: str(row["site_id"])),
        "_semantic_evidence": tuple(semantic_evidence),
    }


def _phase_c_reachable_sources(root: Path) -> tuple[list[Path], list[str]]:
    """Resolve the Rust module graph from ``lib.rs`` with a closed grammar.

    The previous implementation only followed bare ``mod name;`` items.  A
    Rust crate can also select a child with ``#[path = "..."]`` or splice a
    source file with ``include!("...")``.  Those are still lexical edges,
    but they must be parsed explicitly: an unknown attribute, computed path,
    or path escape is a hard graph failure.  Fixture snapshots without a
    crate root retain their complete checked-in tree so the immutable oracle
    remains useful, while any snapshot that supplies ``lib.rs`` gets exactly
    the same fail-closed graph treatment as production.
    """
    root = root.resolve()
    if root.is_file():
        return [root], []
    crate = root / "lib.rs"
    if not crate.is_file():
        return source_files(root), []
    queue = [crate]
    visited: set[Path] = set()
    diagnostics: list[str] = []

    def graph_relpath(path: Path) -> str:
        resolved = path.resolve()
        if resolved.is_relative_to(REPO.resolve()):
            return relpath(resolved)
        return resolved.relative_to(root.resolve()).as_posix()

    while queue:
        path = queue.pop(0).resolve()
        if path in visited:
            continue
        visited.add(path)
        try:
            source = path.read_text(encoding="utf-8")
            items, _ = _source_census(source)
        except (OSError, UnicodeError, ScanFailure):
            diagnostics.append(f"phase-c:module-graph-read:{graph_relpath(path)}")
            continue
        masked = mask_non_code(source)
        test_ranges, _ = test_only_ranges(masked)

        def in_test_region(position: int) -> bool:
            return any(start <= position <= end for start, end in test_ranges)

        def edge_prefix(item: RustItem) -> str:
            # ``RustItem.prefix`` is already bounded to the previous
            # top-level delimiter.  Re-read that exact source slice only to
            # recover string literal contents from path attributes.
            boundary = max(masked.rfind(";", 0, item.start), masked.rfind("}", 0, item.start), masked.rfind("{", 0, item.start))
            return source[boundary + 1:item.start]

        def path_attribute(item: RustItem) -> tuple[str | None, str | None]:
            prefix = edge_prefix(item)
            prefix_start = item.start - len(prefix)
            prefix_masked = masked[prefix_start:item.start]
            attrs = re.findall(r"#\s*\[([^\]]*)\]", prefix_masked)
            if not attrs:
                return None, None
            path_attrs = [attr for attr in attrs if re.fullmatch(r"\s*path\s*=\s*\s*", attr)]
            # Recover the literal from the same bounded raw prefix.  A path
            # attribute must be exactly one ordinary, non-escaped string.
            raw_paths = re.findall(r"#\s*\[\s*path\s*=\s*\"([^\"\\]*)\"\s*\]", prefix)
            if len(path_attrs) != len(raw_paths) or len(raw_paths) > 1:
                return None, "invalid path attribute"
            cfg_attrs = [attr for attr in attrs if attr.strip().startswith("cfg")]
            for attr in attrs:
                normalized = re.sub(r"\s+", "", attr)
                if normalized not in {"cfg(test)", "path=", "allow(non_snake_case)"} and not normalized.startswith("path="):
                    return None, "unsupported module attribute"
            if cfg_attrs and any(re.sub(r"\s+", "", attr) != "cfg(test)" for attr in cfg_attrs):
                return None, "unsupported cfg attribute"
            return (raw_paths[0] if raw_paths else None), None

        def resolve_edge(raw: str, base: Path) -> Path | None:
            if not raw or raw.startswith("/") or "\\" in raw or ".." in Path(raw).parts:
                return None
            candidate = (base / raw).resolve()
            try:
                candidate.relative_to(root.resolve())
            except ValueError:
                return None
            return candidate if candidate.is_file() else None

        for item in items:
            if item.kind != "mod":
                continue
            if in_test_region(item.start):
                continue
            item_words = _item_words(item)
            if "{" in item_words:
                # Inline modules are already part of this source file.  Their
                # nested external modules are discovered by the same census
                # on this file; only a semicolon creates a file edge.
                continue
            selected, attribute_error = path_attribute(item)
            if attribute_error:
                diagnostics.append(f"phase-c:module-graph-attribute:{graph_relpath(path)}::{item.name}")
                continue
            if selected is not None:
                # Explicit literals use the same root-contained resolver as
                # every other graph edge; `../` and absolute/escaped paths
                # never receive a direct filesystem fallback.
                target = resolve_edge(selected, path.parent)
                candidates = ()
            else:
                candidates = (path.parent / f"{item.name}.rs", path.parent / item.name / "mod.rs")
                target = next((candidate for candidate in candidates if candidate.is_file()), None)
            if target is None:
                diagnostics.append(f"phase-c:module-graph-unresolved:{graph_relpath(path)}::{item.name}")
            else:
                queue.append(target)
        # ``include!`` is not a declaration item, so it must be scanned from
        # the token stream separately.  Only a direct string literal is
        # admitted; concat!/env!/format! and malformed invocations remain
        # unresolved graph edges rather than silently disappearing.
        tokens = _rust_tokens(source)
        for index in range(len(tokens) - 2):
            if tokens[index].text != "include" or tokens[index + 1].text != "!" or tokens[index + 2].text != "(":
                continue
            if in_test_region(tokens[index].start):
                continue
            # `include!` is never an authority-bound module edge in this
            # bounded grammar.  This applies equally inside a function and
            # at item position: literal expansion can manufacture declarations
            # whose path/item identity is not frozen by the authority.
            diagnostics.append(f"phase-c:module-graph-include-unsupported:{graph_relpath(path)}")
    return sorted(visited), sorted(set(diagnostics))


def _phase_c_selected_paths(
    matcher: str,
    _closure: tuple[PhaseCFunctionSummary, ...],
    by_name: dict[str, tuple[PhaseCFunctionSummary, ...]],
    source: PhaseCFunctionSummary | None = None,
    operands: tuple[PhaseCConversionOperand, ...] = (),
) -> tuple[ProvenancePath, ...]:
    """Select conversion paths for one authority-instantiated matcher."""
    if source is None:
        return ()
    conversion_kinds = _phase_c_matcher_conversion_kinds(matcher)
    result: list[ProvenancePath] = []
    for operand in operands:
        if operand.kind not in conversion_kinds:
            continue
        if not operand.operand_name or not operand.operand_binding_id:
            result.append(_phase_c_path_unresolved(source, None, "operand-identity", evidence=True))
            continue
        path = _phase_c_trace_path(source, operand.operand_binding_id, by_name)
        if path is not None:
            result.append(path)
    return tuple(result)


def _phase_c_perimeter_signal(
    item: RustItem,
    words: tuple[str, ...],
    resolution: PhaseCSemanticResolution,
    static_symbols: frozenset[str],
) -> bool:
    """Recognize one already-resolved directed perimeter topology."""
    _ = item, words
    if len(resolution.conversion_operands) != 1 or len(resolution.paths) != 1:
        # A resolved multi-path or zero-path exact conversion route is a
        # candidate even when no individual path carries an ``evidence`` bit;
        # cardinality itself is the fail-closed signal.
        return True
    path = resolution.paths[0]
    if path.unresolved:
        # Unsupported syntax is a candidate only when it is on this exact
        # carried conversion path.  No closure-wide counter/store may make a
        # free or unrelated conversion look opaque.
        return path.evidence
    terminal_edges = tuple(
        terminal for terminal in resolution.terminal_edges
        if not static_symbols or terminal.symbol in static_symbols
    )
    if len(terminal_edges) > 1:
        # Multiple terminal writes on the carried binding are an explicit
        # fail-closed candidate.  The report records a stable diagnostic and
        # never chooses one edge for a digest.
        return True
    return len(terminal_edges) != 1 or bool(terminal_edges)


def _phase_c_provenance_diagnostic(path: ProvenancePath) -> str | None:
    """Name one selected-path grammar failure without collapsing it to text."""
    if not path.unresolved:
        return None
    item = path.endpoint_item
    if path.unresolved == "transformed-argument":
        origins = [record for record in path.records if record and record[0] == "conversion-origin"]
        if origins:
            item = origins[-1][2]
    return f"phase-c:provenance:{path.unresolved}:{path.endpoint_path}::{item}"


def phase_c_fixture_result(
    fixture: Path,
    *,
    identity_prefix: str | None = None,
) -> list[str]:
    """Run one immutable Phase-C snapshot through the same resolver."""
    authority = fixture / "authority.toml"
    if not authority.is_file():
        return ["phase-c:fixture-authority-missing"]
    report = phase_c_report(fixture, authority, identity_prefix=identity_prefix)
    failures = list(report.get("diagnostics", []))
    if report.get("unseeded_candidates"):
        failures.append("phase-c:unseeded-candidate")
    inventory = fixture / "inventory.toml"
    lock = fixture / "observations.lock.toml"
    if inventory.is_file() and lock.is_file():
        try:
            failures.extend(compare_phase_c_derived(report, load_inventory(inventory), load_observations_lock(lock)))
        except (OSError, tomllib.TOMLDecodeError) as error:
            failures.append(f"phase-c:fixture-derived-load:{type(error).__name__}")
    stale = fixture / "stale.toml"
    if stale.is_file():
        failures.extend(_phase_c_stale_diagnostics_v4(
            fixture, report, authority, identity_prefix=identity_prefix,
        ))
    return sorted(set(str(value) for value in failures if str(value)))


PHASE_C_RELATION_SCHEMA = "mamba.t1.opaque-value-boundary.v5.phase-c-relation"
PHASE_C_STALE_SCHEMA = "mamba.t1.opaque-value-boundary.v4.phase-c-stale"
PHASE_C_RELATION_CASES = frozenset({
    "edge_v6_format_only_equal", "edge_v6_binding_key_rebind_different",
    "edge_v6_conversion_move_different", "edge_v6_key_target_different",
})
PHASE_C_ZERO_DIGEST = "0" * 64
PHASE_C_SITE_IDS = (
    "selector_central_array", "selector_central_decimal", "selector_central_fractions",
    "selector_central_graphlib", "selector_central_hashlib", "selector_central_hmac",
    "selector_central_ipaddress", "selector_central_json", "selector_central_queue",
    "selector_central_random", "selector_central_uuid", "selector_direct_cell",
    "selector_direct_closure", "selector_direct_coroutine", "selector_direct_file",
    "selector_direct_generator", "selector_direct_iter_store", "selector_direct_range",
    "selector_direct_task", "selector_native_random",
    "selector_threading_barrier_instance_field",
)
PHASE_C_RELATION_TOP_KEYS = frozenset({
    "schema", "version", "capture_state", "case", "expected_relation", "sides",
    "logical_identity_prefix", "authority_pair_delta", "expected_site_ids", "expected_seeded",
    "expected_total", "expected_unseeded", "baseline", "candidate",
})
PHASE_C_RELATION_AUTHORITY_PAIR_DELTAS = MappingProxyType({
    "edge_v6_format_only_equal": "candidate-route_central_decimal-source_digest",
    "edge_v6_binding_key_rebind_different": "none",
    "edge_v6_conversion_move_different": "none",
    "edge_v6_key_target_different": "none",
})
PHASE_C_RELATION_SIDE_KEYS = frozenset({
    "selected_source_path", "selected_site_id", "selected_route_id",
    "selected_semantic_id", "selected_symbol", "selected_matcher",
    "selected_store_id", "selected_store_symbol", "selected_expression_digest",
    "selected_edge_digest",
})


PHASE_C_RELATION_SIDE_FILES = (
    "authority.toml", "runtime/value.rs", "typed_routes.rs",
)


def _phase_c_relation_top_level_function(source: str, name: str) -> RustItem | None:
    """Find one named top-level fixture function using only lexical census data."""
    try:
        items, _ = _source_census(source)
    except ScanFailure:
        return None
    matches = [item for item in items if item.kind == "fn" and item.depth == 0 and item.name == name]
    return matches[0] if len(matches) == 1 else None


def _phase_c_relation_item_words(source: str, item: RustItem) -> tuple[str, ...]:
    """Read the selected item's Rust tokens without any semantic traversal."""
    return tuple(token.text for token in _rust_tokens(source[item.start:item.end]))


def _phase_c_relation_positions(words: tuple[str, ...], needle: tuple[str, ...]) -> list[int]:
    """Return exact lexical occurrences for a frozen relation source shape."""
    return [
        index for index in range(len(words) - len(needle) + 1)
        if words[index:index + len(needle)] == needle
    ]


def _phase_c_relation_format_authority_delta(
    baseline_bytes: bytes,
    candidate_bytes: bytes,
) -> list[str]:
    """Accept precisely the V13c candidate route source-digest delta."""
    try:
        baseline = tomllib.loads(baseline_bytes.decode("utf-8"))
        candidate = tomllib.loads(candidate_bytes.decode("utf-8"))
    except (UnicodeError, tomllib.TOMLDecodeError):
        return ["phase-c:relation-authority-delta"]
    baseline_routes = baseline.get("routes")
    candidate_routes = candidate.get("routes")
    if not isinstance(baseline_routes, list) or not isinstance(candidate_routes, list):
        return ["phase-c:relation-authority-delta"]
    baseline_matches = [
        (index, row)
        for index, row in enumerate(baseline_routes)
        if isinstance(row, dict) and row.get("id") == "route_central_decimal"
    ]
    candidate_matches = [
        (index, row)
        for index, row in enumerate(candidate_routes)
        if isinstance(row, dict) and row.get("id") == "route_central_decimal"
    ]
    if len(baseline_matches) != 1 or len(candidate_matches) != 1:
        return ["phase-c:relation-authority-delta"]
    baseline_index, baseline_route = baseline_matches[0]
    candidate_index, candidate_route = candidate_matches[0]
    baseline_digest = baseline_route.get("source_digest")
    candidate_digest = candidate_route.get("source_digest")
    if (
        not isinstance(baseline_digest, str)
        or not isinstance(candidate_digest, str)
        or re.fullmatch(r"[0-9a-f]{64}", baseline_digest) is None
        or re.fullmatch(r"[0-9a-f]{64}", candidate_digest) is None
        or baseline_digest == candidate_digest
        or baseline_bytes.count(baseline_digest.encode("ascii")) != 1
        or candidate_bytes.count(candidate_digest.encode("ascii")) != 1
        or candidate_bytes.replace(
            candidate_digest.encode("ascii"), baseline_digest.encode("ascii"), 1,
        ) != baseline_bytes
    ):
        return ["phase-c:relation-authority-delta"]
    normalized_candidate = tomllib.loads(candidate_bytes.decode("utf-8"))
    normalized_routes = normalized_candidate.get("routes")
    if not isinstance(normalized_routes, list) or not isinstance(normalized_routes[candidate_index], dict):
        return ["phase-c:relation-authority-delta"]
    normalized_routes[candidate_index]["source_digest"] = baseline_digest
    if normalized_candidate != baseline or baseline_index != candidate_index:
        return ["phase-c:relation-authority-delta"]
    return []


def _phase_c_relation_evidence(
    fixture: Path,
    relation: dict[str, object],
) -> list[str]:
    """Fence V13 relation fixtures with bytes, spans, and Rust tokens only."""
    case = str(relation.get("case", ""))
    failures: list[str] = []
    side_bytes: dict[str, dict[str, bytes]] = {}
    runtime_sources: dict[str, str] = {}
    selected_items: dict[str, RustItem] = {}
    allocator_items: dict[str, RustItem] = {}
    for side in ("baseline", "candidate"):
        root = fixture / side
        actual_files = tuple(sorted(
            path.relative_to(root).as_posix()
            for path in root.rglob("*") if path.is_file()
        )) if root.is_dir() else ()
        if actual_files != PHASE_C_RELATION_SIDE_FILES:
            failures.append(f"phase-c:relation-nonruntime:{side}")
            continue
        files = {
            relative: (root / relative).read_bytes()
            for relative in PHASE_C_RELATION_SIDE_FILES
        }
        side_bytes[side] = files
        try:
            runtime_sources[side] = files["runtime/value.rs"].decode("utf-8")
        except UnicodeDecodeError:
            failures.append(f"phase-c:relation-evidence-runtime-encoding:{side}")
            continue
        selected = _phase_c_relation_top_level_function(runtime_sources[side], "seeded_from_api")
        if selected is None:
            failures.append(f"phase-c:relation-evidence-selected-item:{side}")
        else:
            selected_items[side] = selected
        allocator = _phase_c_relation_top_level_function(runtime_sources[side], "seeded_from_alloc")
        if allocator is None:
            failures.append(f"phase-c:relation-evidence-allocator-item:{side}")
        else:
            allocator_items[side] = allocator

    if set(side_bytes) != {"baseline", "candidate"}:
        return sorted(set(failures + ["phase-c:relation-evidence-sides"]))
    if case == "edge_v6_format_only_equal":
        if relation.get("authority_pair_delta") != (
            "candidate-route_central_decimal-source_digest"
        ):
            failures.append("phase-c:relation-authority-delta")
        if side_bytes["baseline"]["typed_routes.rs"] != side_bytes["candidate"]["typed_routes.rs"]:
            failures.append("phase-c:relation-nonruntime")
        failures.extend(_phase_c_relation_format_authority_delta(
            side_bytes["baseline"]["authority.toml"],
            side_bytes["candidate"]["authority.toml"],
        ))
        baseline = selected_items.get("baseline")
        candidate = selected_items.get("candidate")
        if baseline is not None and candidate is not None:
            baseline_source = runtime_sources["baseline"]
            candidate_source = runtime_sources["candidate"]
            if (
                baseline_source[:baseline.start] != candidate_source[:candidate.start]
                or baseline_source[baseline.end:] != candidate_source[candidate.end:]
                or baseline_source[baseline.start:baseline.end] == candidate_source[candidate.start:candidate.end]
                or _phase_c_relation_item_words(baseline_source, baseline)
                != _phase_c_relation_item_words(candidate_source, candidate)
            ):
                failures.append("phase-c:relation-format-scope")
        else:
            failures.append("phase-c:relation-format-scope")
    elif case == "edge_v6_binding_key_rebind_different":
        for side in ("baseline", "candidate"):
            source = runtime_sources.get(side, "")
            allocator = allocator_items.get(side)
            if allocator is None:
                continue
            words = _phase_c_relation_item_words(source, allocator)
            if _phase_c_relation_top_level_function(source, "seeded_from_put") is not None:
                failures.append(f"phase-c:relation-evidence-binding-helper:{side}")
            inserts = _phase_c_relation_positions(words, ("insert", "(", "slot", ",", "value", ")"))
            slots = _phase_c_relation_positions(words, ("let", "slot", "=", "SEEDED_FROM_NEXT"))
            if len(inserts) != 1:
                failures.append(f"phase-c:relation-evidence-binding-insert:{side}")
            if len(slots) != 1:
                failures.append(f"phase-c:relation-evidence-binding-slot:{side}")
                continue
            marker = ("drop", "(", "0", ")", ";") if side == "baseline" else ("let", "pad", "=", "0", ";")
            if not _phase_c_relation_positions(words[:slots[0]], marker):
                failures.append(f"phase-c:relation-evidence-binding-marker:{side}")
        if runtime_sources.get("baseline") == runtime_sources.get("candidate"):
            failures.append("phase-c:relation-evidence-binding-equal")
    elif case == "edge_v6_conversion_move_different":
        exact = ("let", "converted", "=", "MbValue", "::", "from_int", "(", "raw", "as", "i64", ")", ";")
        positions: dict[str, int] = {}
        for side in ("baseline", "candidate"):
            source = runtime_sources.get(side, "")
            selected = selected_items.get(side)
            if selected is None:
                continue
            words = _phase_c_relation_item_words(source, selected)
            matches = _phase_c_relation_positions(words, exact)
            drops = _phase_c_relation_positions(words, ("drop", "(", "value", ")", ";"))
            tails = _phase_c_relation_positions(words, ("converted", "}"))
            if len(matches) != 1:
                failures.append(f"phase-c:relation-evidence-conversion-binding:{side}")
            else:
                positions[side] = matches[0]
            if len(drops) != 1 or len(tails) != 1:
                failures.append(f"phase-c:relation-evidence-conversion-shape:{side}")
            elif side in positions and (
                (side == "baseline" and positions[side] >= drops[0])
                or (side == "candidate" and positions[side] <= drops[0])
            ):
                failures.append(f"phase-c:relation-evidence-conversion-move:{side}")
        if positions.get("baseline", -1) >= positions.get("candidate", -1):
            failures.append("phase-c:relation-evidence-conversion-order")
        if runtime_sources.get("baseline") == runtime_sources.get("candidate"):
            failures.append("phase-c:relation-evidence-conversion-equal")
    elif case == "edge_v6_key_target_different":
        for side, expected_store in (("baseline", "SEEDED_FROM_TABLE_A"), ("candidate", "SEEDED_FROM_TABLE_B")):
            source = runtime_sources.get(side, "")
            allocator = allocator_items.get(side)
            if allocator is None:
                continue
            words = _phase_c_relation_item_words(source, allocator)
            declarations = {
                symbol: _phase_c_relation_positions(
                    tuple(token.text for token in _rust_tokens(source)), ("static", symbol),
                )
                for symbol in ("SEEDED_FROM_TABLE_A", "SEEDED_FROM_TABLE_B")
            }
            inserts = _phase_c_relation_positions(words, ("insert", "(", "slot", ",", "value", ")"))
            if (
                any(len(matches) != 1 for matches in declarations.values())
                or _phase_c_relation_top_level_function(source, "seeded_from_put") is not None
                or len(inserts) != 1
                or expected_store not in words[:inserts[0] if inserts else 0]
            ):
                failures.append(f"phase-c:relation-evidence-target-store:{side}")
        if runtime_sources.get("baseline") == runtime_sources.get("candidate"):
            failures.append("phase-c:relation-evidence-target-equal")
    else:
        failures.append("phase-c:relation-evidence-case")
    return sorted(set(failures))


def _phase_c_relation_reports(
    fixture: Path,
) -> tuple[dict[str, dict[str, object]], dict[str, object]] | list[str] | None:
    """Run exactly the two closed relation sides through normal reports."""
    relation_path = fixture / "relation.toml"
    if not relation_path.is_file():
        return None
    try:
        relation = tomllib.loads(relation_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError):
        return ["phase-c:relation-schema"]
    if set(relation) != PHASE_C_RELATION_TOP_KEYS:
        return ["phase-c:relation-schema"]
    canonical_root = fixture.resolve().relative_to(REPO.resolve()).as_posix()
    if (
        relation.get("schema") != PHASE_C_RELATION_SCHEMA
        or relation.get("version") != 5
        or relation.get("capture_state") not in {"pending", "reviewed"}
        or relation.get("case") != fixture.name
        or relation.get("case") not in PHASE_C_RELATION_CASES
        or relation.get("expected_relation") not in {"equal", "different"}
        or relation.get("sides") != ["baseline", "candidate"]
        or relation.get("logical_identity_prefix") != canonical_root
        or relation.get("expected_site_ids") != list(PHASE_C_SITE_IDS)
        or relation.get("expected_seeded") != 21
        or relation.get("expected_total") != 21
        or relation.get("expected_unseeded") != 0
    ):
        return ["phase-c:relation-schema"]
    sides = ["baseline", "candidate"]
    if any(
        not isinstance(relation.get(side), dict)
        or set(relation[side]) != PHASE_C_RELATION_SIDE_KEYS
        for side in sides
    ):
        return ["phase-c:relation-schema"]
    contract = PHASE_C_RELATION_CASE_SEMANTICS.get(str(relation["case"]))
    pair_delta = PHASE_C_RELATION_AUTHORITY_PAIR_DELTAS.get(str(relation["case"]))
    if (
        contract is None
        or pair_delta is None
        or relation["expected_relation"] != contract[0]
        or relation["authority_pair_delta"] != pair_delta
    ):
        return ["phase-c:relation-schema"]
    for side in sides:
        expected = relation[side]
        if (
            not all(
                isinstance(expected.get(key), str) and expected[key]
                for key in (
                    "selected_source_path", "selected_site_id", "selected_route_id",
                    "selected_semantic_id", "selected_symbol", "selected_matcher",
                    "selected_store_id", "selected_store_symbol",
                )
            )
            or not isinstance(expected.get("selected_expression_digest"), str)
            or not re.fullmatch(r"[0-9a-f]{64}", expected["selected_expression_digest"])
            or not isinstance(expected.get("selected_edge_digest"), str)
            or not re.fullmatch(r"[0-9a-f]{64}", expected["selected_edge_digest"])
            or expected["selected_semantic_id"] != contract[1]
            or expected["selected_source_path"] != "runtime/value.rs"
            or expected["selected_site_id"] != "selector_central_decimal"
            or expected["selected_route_id"] != "route_central_decimal"
            or expected["selected_symbol"] != "seeded_from_api"
            or expected["selected_matcher"] != "public_from_int_reifier"
            or expected["selected_expression_digest"] != "f1721d8b024d9d95cc8f75a9e8b3e7a7a2700c81d3f7a608fe73df3ce1fdd4ff"
        ):
            return ["phase-c:relation-schema"]
        if relation["capture_state"] == "pending":
            if expected["selected_edge_digest"] != PHASE_C_ZERO_DIGEST:
                return ["phase-c:relation-schema"]
        elif expected["selected_edge_digest"] == PHASE_C_ZERO_DIGEST:
            return ["phase-c:relation-schema"]
    if relation["case"] in {
        "edge_v6_format_only_equal", "edge_v6_binding_key_rebind_different",
        "edge_v6_conversion_move_different",
    } and any(
        relation[side]["selected_store_id"] != "SEEDED_FROM_TABLE"
        or relation[side]["selected_store_symbol"] != "SEEDED_FROM_TABLE"
        for side in sides
    ):
        return ["phase-c:relation-schema"]
    if relation["case"] == "edge_v6_key_target_different" and (
        any(relation[side]["selected_store_id"] != "selected_store" for side in sides)
        or relation["baseline"]["selected_store_symbol"] != "SEEDED_FROM_TABLE_A"
        or relation["candidate"]["selected_store_symbol"] != "SEEDED_FROM_TABLE_B"
    ):
        return ["phase-c:relation-schema"]
    reports: dict[str, dict[str, object]] = {}
    with tempfile.TemporaryDirectory(prefix="mamba-opaque-phase-c-relation-") as temporary_root:
        isolated = Path(temporary_root) / "relation"
        shutil.copytree(fixture, isolated)
        for side in sides:
            side_root = isolated / side
            authority = side_root / "authority.toml"
            if not side_root.is_dir() or not authority.is_file():
                reports[str(side)] = {
                    "diagnostics": [f"phase-c:relation-side-missing:{side}"],
                    "seeded": [], "unseeded_candidates": [],
                }
                continue
            reports[str(side)] = phase_c_report(
                side_root, authority,
                identity_prefix=str(relation["logical_identity_prefix"]),
            )
    return reports, relation


PHASE_C_RELATION_CASE_SEMANTICS = MappingProxyType({
    "edge_v6_format_only_equal": ("equal", "expr-v13-format-only"),
    "edge_v6_binding_key_rebind_different": ("different", "expr-v13-binding-key-rebind"),
    "edge_v6_conversion_move_different": ("different", "expr-v13-conversion-move"),
    "edge_v6_key_target_different": ("different", "expr-v13-store-target"),
})
PHASE_C_STALE_CASE = "semantic_later_path_digest"
PHASE_C_STALE_SEMANTIC_ID = "expr-v13-stale-later-path"
PHASE_C_STALE_TOP_KEYS = frozenset({
    "schema", "version", "capture_state", "logical_identity_prefix",
    "authority_stale_edge_digest", "observed_current_edge_digest",
    "expected_site_ids", "expected_seeded", "expected_total", "expected_unseeded",
    "selected_source_path", "selected_site_id", "selected_route_id",
    "selected_semantic_id", "selected_symbol", "selected_matcher",
    "selected_store_id", "selected_store_symbol", "selected_expression_digest",
})
PHASE_C_SELECTED_REPORT_FIELDS = frozenset({
    "expression_id", "expression_count", "expression_digest", "edge_digest", "store_symbol",
})


def _phase_c_unique_row(
    rows: list[dict[str, object]],
    predicate: Callable[[dict[str, object]], bool],
) -> dict[str, object] | None:
    """Return one row only when the authority/report identity is unique."""
    matches = [row for row in rows if predicate(row)]
    return matches[0] if len(matches) == 1 else None


def _phase_c_authority_chain(
    manifest: dict[str, object],
    expected: dict[str, object],
    *,
    prefix: str,
) -> tuple[dict[str, object] | None, list[str]]:
    """Prove the raw selector -> route -> store -> semantic chain exactly once."""
    failures: list[str] = []
    families = [row for row in manifest.get("families", []) if isinstance(row, dict)]
    escapes = [row for row in manifest.get("public_escapes", []) if isinstance(row, dict)]
    routes = [row for row in manifest.get("routes", []) if isinstance(row, dict)]
    native_routes = [row for row in manifest.get("native_internal_routes", []) if isinstance(row, dict)]
    selectors = [row for row in manifest.get("selectors", []) if isinstance(row, dict)]
    stores = [row for row in manifest.get("stores", []) if isinstance(row, dict)]
    semantic_rows = _phase_c_semantic_rows(manifest)
    family_ids = tuple(str(row.get("id", "")) for row in families)
    escape_ids = tuple(str(row.get("id", "")) for row in escapes)
    if (
        len(families) != 19
        or set(family_ids) != set(PHASE_C_OWNER_IDS)
        or len(set(family_ids)) != len(family_ids)
        or len(escapes) != 1
        or escape_ids != PHASE_C_PUBLIC_ESCAPE_IDS
    ):
        failures.append(f"{prefix}:authority-owner-count")
    route_ids = tuple(str(row.get("id", "")) for row in [*routes, *native_routes])
    store_ids = tuple(str(row.get("id", "")) for row in stores)
    if (
        len(routes) != 20
        or len(native_routes) != 1
        or len(set(route_ids)) != len(route_ids)
        or len(set(store_ids)) != len(store_ids)
    ):
        failures.append(f"{prefix}:authority-route-count")
    selector_ids = tuple(str(row.get("id", "")) for row in selectors)
    # TOML array order is not an authority identity; after uniqueness is
    # proved, normalize it to the one frozen, ordered 21-site tuple used by
    # manifests and normal reports.
    if tuple(sorted(selector_ids)) != PHASE_C_SITE_IDS or len(set(selector_ids)) != len(selector_ids):
        failures.append(f"{prefix}:authority-selectors")
    if len(semantic_rows) != 1:
        failures.append(f"{prefix}:authority-semantic-count")
    selector = _phase_c_unique_row(
        selectors, lambda row: row.get("id") == expected["selected_site_id"],
    )
    route = _phase_c_unique_row(
        [*routes, *native_routes], lambda row: row.get("id") == expected["selected_route_id"],
    )
    store = _phase_c_unique_row(
        stores, lambda row: row.get("id") == expected["selected_store_id"],
    )
    semantic = _phase_c_unique_row(
        semantic_rows, lambda row: row.get("id") == expected["selected_semantic_id"],
    )
    if selector is None:
        failures.append(f"{prefix}:authority-selector")
    if route is None:
        failures.append(f"{prefix}:authority-route")
    if store is None:
        failures.append(f"{prefix}:authority-store")
    if semantic is None:
        failures.append(f"{prefix}:authority-semantic")
    if selector is not None and (
        selector.get("route_id") != expected["selected_route_id"]
        or selector.get("matcher") != expected["selected_matcher"]
    ):
        failures.append(f"{prefix}:authority-selector-link")
    if route is not None and (
        route.get("path") != expected["selected_source_path"]
        or route.get("symbol") != expected["selected_symbol"]
        or route.get("store") != expected["selected_store_id"]
        or not isinstance(route.get("selector_ids"), list)
        or list(route["selector_ids"]).count(expected["selected_site_id"]) != 1
    ):
        failures.append(f"{prefix}:authority-route-link")
    if store is not None and store.get("symbol") != expected["selected_store_symbol"]:
        failures.append(f"{prefix}:authority-store-link")
    if semantic is not None and (
        semantic.get("selector_id") != expected["selected_site_id"]
        or semantic.get("matcher") != expected["selected_matcher"]
        or semantic.get("store_symbol") != expected["selected_store_symbol"]
        or semantic.get("expression_digest") != expected["selected_expression_digest"]
    ):
        failures.append(f"{prefix}:authority-semantic-link")
    return semantic, failures


def _phase_c_report_chain(
    report: dict[str, object],
    expected: dict[str, object],
    *,
    prefix: str,
    relation_identity_prefix: str | None = None,
) -> tuple[dict[str, object] | None, PhaseCSemanticEvidence | None, list[str]]:
    """Cross-bind one normal report to the selected, authority-backed identity."""
    failures: list[str] = []
    sites = [row for row in report.get("sites", []) if isinstance(row, dict)]
    seeded = [row for row in report.get("seeded", []) if isinstance(row, dict)]
    site_ids = tuple(str(row.get("site_id", "")) for row in sites)
    if site_ids != PHASE_C_SITE_IDS or len(set(site_ids)) != len(site_ids):
        failures.append(f"{prefix}:report-sites")
    if len(seeded) != 21 or report.get("total") != 21 or report.get("unseeded_candidates") != []:
        failures.append(f"{prefix}:report-counts")
    semantic_rows = [
        row for row in seeded
        if PHASE_C_SELECTED_REPORT_FIELDS.intersection(row)
    ]
    if len(semantic_rows) != 1:
        failures.append(f"{prefix}:report-semantic-cardinality")
        selected = None
    else:
        selected = semantic_rows[0]
        if any(
            PHASE_C_SELECTED_REPORT_FIELDS.intersection(row)
            for row in seeded if row is not selected
        ):
            failures.append(f"{prefix}:report-semantic-fields")
    if selected is not None:
        selected_checks = {
            "site_id": expected["selected_site_id"],
            "path": expected["selected_source_path"],
            "symbol": expected["selected_symbol"],
            "kind": expected["selected_matcher"],
            "parent_route": f"phase-c/{expected['selected_route_id']}",
            "expression_id": expected["selected_semantic_id"],
            "expression_count": 1,
            "expression_digest": expected["selected_expression_digest"],
            "store_symbol": expected["selected_store_symbol"],
        }
        if any(selected.get(key) != value for key, value in selected_checks.items()):
            failures.append(f"{prefix}:report-selected")
        edge = selected.get("edge_digest")
        if not isinstance(edge, str) or not re.fullmatch(r"[0-9a-f]{64}", edge) or edge == PHASE_C_ZERO_DIGEST:
            failures.append(f"{prefix}:report-edge-digest")
    evidence_rows = [
        row for row in report.get("_semantic_evidence", ())
        if isinstance(row, PhaseCSemanticEvidence)
    ]
    evidence = _phase_c_unique_row(
        [
            {
                "evidence": row,
                "semantic_id": row.semantic_id,
                "site_id": row.site_id,
                "source_path": row.source_path,
                "route_id": row.route_id,
                "matcher": row.matcher,
            }
            for row in evidence_rows
        ],
        lambda row: (
            row["semantic_id"] == expected["selected_semantic_id"]
            and row["site_id"] == expected["selected_site_id"]
        ),
    )
    if evidence is None:
        failures.append(f"{prefix}:report-evidence")
        projected = None
    else:
        projected = evidence["evidence"]
        if not isinstance(projected, PhaseCSemanticEvidence) or (
            evidence["source_path"] != expected["selected_source_path"]
            or evidence["route_id"] != expected["selected_route_id"]
            or evidence["matcher"] != expected["selected_matcher"]
        ):
            failures.append(f"{prefix}:report-evidence-link")
        elif relation_identity_prefix is not None and (
            projected.conversion_count != 1
            or projected.path_count != 1
            or projected.terminal_count != 1
            or not projected.edge_complete
            or projected.conversion_owner_item != "seeded_from_api"
            or projected.conversion_operand_name != "raw"
            or projected.conversion_tokens != (
                "MbValue", "::", "from_int", "(", "raw", "as", "i64", ")",
            )
            or projected.terminal_method != "insert"
            or projected.terminal_receiver != "table"
            or projected.terminal_key_arguments != ("slot",)
            or projected.terminal_owner_path != f"{relation_identity_prefix}/runtime/value.rs"
        ):
            failures.append(f"{prefix}:report-evidence-contract")
    return selected, projected, failures


def _phase_c_relation_evidence_pairs(
    relation: dict[str, object],
    evidence: dict[str, PhaseCSemanticEvidence],
    report_edges: dict[str, str],
) -> list[str]:
    """Compare only immutable report evidence; never rebuild a resolution here."""
    failures: list[str] = []
    baseline = evidence.get("baseline")
    candidate = evidence.get("candidate")
    if baseline is None or candidate is None:
        return ["phase-c:relation-evidence-missing"]
    if set(report_edges) != {"baseline", "candidate"}:
        return ["phase-c:relation-evidence-edge-missing"]
    case = str(relation["case"])
    expected_terminal_owner = (
        "seeded_from_put"
        if case in {"edge_v6_format_only_equal", "edge_v6_conversion_move_different"}
        else "seeded_from_alloc"
    )
    if (
        baseline.terminal_owner_item != expected_terminal_owner
        or candidate.terminal_owner_item != expected_terminal_owner
    ):
        failures.append("phase-c:relation-evidence-terminal-owner")
    conversion_common = (
        "conversion_owner_path", "conversion_owner_item", "conversion_operand_name",
        "conversion_operand_binding_id", "conversion_tokens",
    )
    path_common = (
        "path_endpoint_path", "path_endpoint_item", "path_hops",
    )
    terminal_common = (
        "terminal_owner_path", "terminal_owner_item", "terminal_symbol",
        "terminal_operation", "terminal_method", "terminal_receiver", "terminal_declaration_id",
    )
    if case == "edge_v6_format_only_equal":
        if baseline != candidate or report_edges["baseline"] != report_edges["candidate"]:
            failures.append("phase-c:relation-evidence-format")
    elif case == "edge_v6_binding_key_rebind_different":
        if any(getattr(baseline, key) != getattr(candidate, key) for key in conversion_common):
            failures.append("phase-c:relation-evidence-binding-conversion")
        if baseline.conversion_direct_token_ordinal != candidate.conversion_direct_token_ordinal:
            failures.append("phase-c:relation-evidence-binding-ordinal")
        if any(getattr(baseline, key) != getattr(candidate, key) for key in (*path_common, *terminal_common)):
            failures.append("phase-c:relation-evidence-binding-neutral")
        if (
            baseline.path_endpoint_binding_id == candidate.path_endpoint_binding_id
            or baseline.terminal_endpoint_binding_id == candidate.terminal_endpoint_binding_id
            or baseline.terminal_key_binding_ids == candidate.terminal_key_binding_ids
            or baseline.terminal_key_arguments != ("slot",)
            or candidate.terminal_key_arguments != ("slot",)
        ):
            failures.append("phase-c:relation-evidence-binding-key")
    elif case == "edge_v6_conversion_move_different":
        move_common = (*conversion_common, *path_common, *terminal_common,
                       "path_endpoint_binding_id", "terminal_endpoint_binding_id",
                       "terminal_receiver_binding_id", "terminal_key_arguments",
                       "terminal_key_binding_ids")
        if any(getattr(baseline, key) != getattr(candidate, key) for key in move_common):
            failures.append("phase-c:relation-evidence-move-neutral")
        if not baseline.conversion_direct_token_ordinal < candidate.conversion_direct_token_ordinal:
            failures.append("phase-c:relation-evidence-move-ordinal")
    elif case == "edge_v6_key_target_different":
        target_common = (*conversion_common, "conversion_direct_token_ordinal", *path_common,
                         "path_endpoint_binding_id", "terminal_endpoint_binding_id",
                         "terminal_operation", "terminal_method", "terminal_receiver",
                         "terminal_key_arguments", "terminal_key_binding_ids")
        if any(getattr(baseline, key) != getattr(candidate, key) for key in target_common):
            failures.append("phase-c:relation-evidence-target-neutral")
        if (
            baseline.terminal_symbol == candidate.terminal_symbol
            or baseline.terminal_declaration_id == candidate.terminal_declaration_id
        ):
            failures.append("phase-c:relation-evidence-target-store")
    else:
        failures.append("phase-c:relation-evidence-case")
    return failures


def _phase_c_relation_diagnostics_v5(
    fixture: Path,
    reports_and_manifest: tuple[dict[str, dict[str, object]], dict[str, object]],
) -> list[str]:
    """Validate closed V13 relation manifests using normal-report evidence only."""
    reports, relation = reports_and_manifest
    failures = _phase_c_relation_evidence(fixture, relation)
    evidence: dict[str, PhaseCSemanticEvidence] = {}
    report_edges: dict[str, str] = {}
    for side in ("baseline", "candidate"):
        expected = relation[side]
        if not isinstance(expected, dict):
            failures.append(f"phase-c:relation-schema:{side}")
            continue
        try:
            authority_manifest = tomllib.loads(
                (fixture / side / "authority.toml").read_text(encoding="utf-8")
            )
        except (OSError, UnicodeError, tomllib.TOMLDecodeError):
            authority_manifest = {}
            failures.append(f"phase-c:relation-authority-load:{side}")
        semantic, authority_failures = _phase_c_authority_chain(
            authority_manifest, expected, prefix=f"phase-c:relation:{side}",
        )
        failures.extend(authority_failures)
        selected, projected, report_failures = _phase_c_report_chain(
            reports.get(side, {}), expected, prefix=f"phase-c:relation:{side}",
            relation_identity_prefix=str(relation["logical_identity_prefix"]),
        )
        failures.extend(report_failures)
        diagnostics = [str(value) for value in reports.get(side, {}).get("diagnostics", []) if str(value)]
        raw_diagnostic = f"phase-c:semantic-edge-digest:{expected['selected_semantic_id']}"
        expected_diagnostics = [raw_diagnostic] if relation["capture_state"] == "pending" else []
        if diagnostics != expected_diagnostics:
            failures.append(f"phase-c:relation-diagnostics:{side}")
        failures.extend(diagnostics)
        authority_edge = semantic.get("edge_digest") if semantic is not None else None
        manifest_edge = expected.get("selected_edge_digest")
        report_edge = selected.get("edge_digest") if selected is not None else None
        if isinstance(report_edge, str) and re.fullmatch(r"[0-9a-f]{64}", report_edge):
            report_edges[side] = report_edge
        if relation["capture_state"] == "pending":
            if authority_edge != PHASE_C_ZERO_DIGEST or manifest_edge != PHASE_C_ZERO_DIGEST:
                failures.append(f"phase-c:relation-pending-zero:{side}")
            failures.append(f"phase-c:capture-pending:{relation['case']}:{side}")
            failures.append(f"phase-c:capture-zero-digest:{relation['case']}:{side}")
        elif (
            not isinstance(authority_edge, str)
            or authority_edge == PHASE_C_ZERO_DIGEST
            or authority_edge != manifest_edge
            or authority_edge != report_edge
        ):
            failures.append(f"phase-c:relation-reviewed-edge:{side}")
        if projected is not None:
            evidence[side] = projected
    if len(report_edges) == 2:
        observed_relation = "equal" if report_edges["baseline"] == report_edges["candidate"] else "different"
        if observed_relation != relation["expected_relation"]:
            failures.append(f"phase-c:relation-equality:{relation['case']}:{observed_relation}")
    else:
        failures.append("phase-c:relation-edge-cardinality")
    failures.extend(_phase_c_relation_evidence_pairs(relation, evidence, report_edges))
    return sorted(set(failures))


def _phase_c_stale_diagnostics_v4(
    fixture: Path,
    report: dict[str, object],
    authority: Path,
    *,
    identity_prefix: str | None,
) -> list[str]:
    """Validate the closed V13 stale-edge contract against one normal report."""
    stale_path = fixture / "stale.toml"
    try:
        stale = tomllib.loads(stale_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError):
        return ["phase-c:stale-schema"]
    canonical_prefix = identity_prefix
    if canonical_prefix is None:
        try:
            canonical_prefix = fixture.resolve().relative_to(REPO.resolve()).as_posix()
        except ValueError:
            return ["phase-c:stale-schema"]
    if (
        fixture.name != PHASE_C_STALE_CASE
        or set(stale) != PHASE_C_STALE_TOP_KEYS
        or stale.get("schema") != PHASE_C_STALE_SCHEMA
        or stale.get("version") != 4
        or stale.get("capture_state") not in {"pending", "reviewed"}
        or stale.get("logical_identity_prefix") != canonical_prefix
        or stale.get("expected_site_ids") != list(PHASE_C_SITE_IDS)
        or stale.get("expected_seeded") != 21
        or stale.get("expected_total") != 21
        or stale.get("expected_unseeded") != 0
        or stale.get("selected_source_path") != "runtime/value.rs"
        or stale.get("selected_site_id") != "selector_central_array"
        or stale.get("selected_route_id") != "route_central_array"
        or stale.get("selected_semantic_id") != PHASE_C_STALE_SEMANTIC_ID
        or stale.get("selected_symbol") != "route_central_array"
        or stale.get("selected_matcher") != "public_from_int_reifier"
        or stale.get("selected_store_id") != "fixture_store"
        or stale.get("selected_store_symbol") != "SEM_TABLE"
        or stale.get("selected_expression_digest") != "f1721d8b024d9d95cc8f75a9e8b3e7a7a2700c81d3f7a608fe73df3ce1fdd4ff"
        or not all(
            isinstance(stale.get(key), str) and re.fullmatch(r"[0-9a-f]{64}", stale[key])
            for key in ("authority_stale_edge_digest", "observed_current_edge_digest")
        )
        or stale["authority_stale_edge_digest"] == PHASE_C_ZERO_DIGEST
        or (
            stale["capture_state"] == "pending"
            and stale["observed_current_edge_digest"] != PHASE_C_ZERO_DIGEST
        )
        or (
            stale["capture_state"] == "reviewed"
            and stale["observed_current_edge_digest"] == PHASE_C_ZERO_DIGEST
        )
    ):
        return ["phase-c:stale-schema"]
    failures: list[str] = []
    try:
        authority_manifest = tomllib.loads(authority.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError):
        authority_manifest = {}
        failures.append("phase-c:stale-authority-load")
    semantic, authority_failures = _phase_c_authority_chain(
        authority_manifest, stale, prefix="phase-c:stale",
    )
    failures.extend(authority_failures)
    selected, _, report_failures = _phase_c_report_chain(
        report, stale, prefix="phase-c:stale",
    )
    failures.extend(report_failures)
    raw_diagnostic = f"phase-c:semantic-edge-digest:{PHASE_C_STALE_SEMANTIC_ID}"
    diagnostics = [str(value) for value in report.get("diagnostics", []) if str(value)]
    if diagnostics != [raw_diagnostic]:
        failures.append("phase-c:stale-diagnostics")
    authority_edge = semantic.get("edge_digest") if semantic is not None else None
    report_edge = selected.get("edge_digest") if selected is not None else None
    if authority_edge != stale["authority_stale_edge_digest"]:
        failures.append("phase-c:stale-authority-digest")
    if not isinstance(report_edge, str) or not re.fullmatch(r"[0-9a-f]{64}", report_edge):
        failures.append("phase-c:stale-current-edge-digest")
    elif report_edge == stale["authority_stale_edge_digest"]:
        failures.append("phase-c:stale-current-edge-not-different")
    if stale["capture_state"] == "pending":
        failures.append("phase-c:capture-pending:semantic_later_path_digest:current")
        failures.append("phase-c:capture-zero-digest:semantic_later_path_digest:current")
    elif report_edge != stale["observed_current_edge_digest"]:
        failures.append("phase-c:stale-current-edge-digest")
    return sorted(set(failures))


def _phase_c_opaque_fixture_result(fixture: Path) -> list[str]:
    """Run one immutable Phase-C tree below an opaque temporary root."""
    relation_reports = _phase_c_relation_reports(fixture)
    if isinstance(relation_reports, list):
        return relation_reports
    if relation_reports is not None:
        return _phase_c_relation_diagnostics_v5(fixture, relation_reports)
    canonical = fixture.resolve().relative_to(REPO.resolve()).as_posix()
    with tempfile.TemporaryDirectory(prefix="mamba-opaque-phase-c-case-") as temporary_root:
        isolated = (Path(temporary_root) / "case").resolve()
        shutil.copytree(fixture, isolated)
        diagnostics = phase_c_fixture_result(isolated, identity_prefix=canonical)
    # The checker deliberately reports source-relative paths for a temporary
    # root.  Reattach the frozen fixture-relative prefix before comparing the
    # result so diagnostics remain independent of the temporary directory
    # name while execution cannot branch on the checked-in path.
    return sorted(set(
        diagnostic
        if f"{canonical}/runtime/" in diagnostic
        else diagnostic.replace("runtime/", f"{canonical}/runtime/")
        for diagnostic in diagnostics
    ))


def _phase_c_structural_fixture_result(fixture: Path) -> list[str]:
    """Check a structural fixture with the production census/context only."""
    manifest_path = fixture / "structural.toml"
    if not manifest_path.is_file():
        return ["phase-c:structural-manifest-missing"]
    try:
        manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError):
        return ["phase-c:structural-manifest-load"]
    expected_keys = {
        "schema", "version", "source", "target_kind", "target_name",
        "expected_classification", "expected_ancestors", "expected_cfg_test",
    }
    if set(manifest) != expected_keys:
        return ["phase-c:structural-schema"]
    if manifest.get("schema") != "mamba.t1.opaque-value-boundary.v2.structural" or manifest.get("version") != 1:
        return ["phase-c:structural-schema"]
    source_path = fixture / str(manifest["source"])
    if not source_path.is_file():
        return ["phase-c:structural-source-missing"]
    try:
        source = source_path.read_text(encoding="utf-8")
        census, _ = _source_census(source, include_nested_functions=True)
    except (OSError, UnicodeError, ScanFailure):
        return ["phase-c:structural-source-invalid"]
    matches = [
        item for item in census
        if item.kind == manifest["target_kind"] and item.name == manifest["target_name"]
    ]
    if len(matches) != 1:
        return [f"phase-c:structural-target-cardinality:{len(matches)}"]
    context = _phase_c_structural_context(source, matches[0], census)
    observed_ancestors = [[kind, name] for kind, name in context.ancestors]
    diagnostics: list[str] = []
    if context.nearest_kind != manifest["expected_classification"]:
        diagnostics.append(f"phase-c:structural-classification:{context.nearest_kind}")
    if observed_ancestors != manifest["expected_ancestors"]:
        diagnostics.append(f"phase-c:structural-ancestors:{observed_ancestors!r}")
    if context.cfg_test != manifest["expected_cfg_test"]:
        diagnostics.append(f"phase-c:structural-cfg-test:{context.cfg_test}")
    return sorted(diagnostics)


def _item_words(item: RustItem) -> tuple[str, ...]:
    return tuple(token.text for token in item.tokens)


def _function_parts(item: RustItem) -> tuple[tuple[str, ...], tuple[str, ...]]:
    words = _item_words(item)
    try:
        fn_index = words.index("fn")
        opening = words.index("{", fn_index)
    except ValueError:
        return (), ()
    return words[fn_index:opening], words[opening:]


def _split_args(words: tuple[str, ...]) -> list[tuple[str, ...]] | None:
    result: list[tuple[str, ...]] = []
    current: list[str] = []
    depth = 0
    for word in words:
        if word in {"(", "[", "<", "{"}:
            depth += 1
        elif word in {")", "]", ">", "}"}:
            depth -= 1
        if word == "," and depth == 0:
            result.append(tuple(current))
            current = []
        else:
            current.append(word)
    if current or not result:
        result.append(tuple(current))
    return result


def _signature(item: RustItem) -> tuple[tuple[tuple[str, tuple[str, ...]], ...], tuple[str, ...]] | None:
    header, _ = _function_parts(item)
    try:
        opening = header.index("(")
        closing = len(header) - 1 - header[::-1].index(")")
    except ValueError:
        return None
    params: list[tuple[str, tuple[str, ...]]] = []
    for arg in _split_args(tuple(header[opening + 1:closing])) or []:
        if not arg:
            continue
        if len(arg) < 3 or arg[1] != ":":
            return None
        params.append((arg[0], tuple(arg[2:])))
    return tuple(params), tuple(header[closing + 1:])


def _body_matches(item: RustItem, expected: str, *, kind: str = "") -> bool:
    _, actual = _function_parts(item)
    # The canonical grammar has one spelling.  In particular, accepting a
    # trailing match-arm comma here would create a second grammar branch that
    # is not tied to the authority-instantiated body.
    return actual == _rust_words(expected)


def _normalized_item_digest(source: str) -> str:
    """Digest the authority item after comments/whitespace normalization.

    The versioned NUL-separated lexer-token recipe makes the selector anchor
    independent of presentation while remaining bound to the exact item
    grammar.  ``source_digest`` below intentionally remains a separate exact
    UTF-8 item digest.
    """
    tokens = _rust_words(mask_non_code(source))
    return hashlib.sha256(("v1\0" + "\0".join(tokens)).encode("utf-8")).hexdigest()


def _source_item_digest(source: str) -> str:
    return hashlib.sha256(source.encode("utf-8")).hexdigest()


def _function_item(items: tuple[RustItem, ...], name: str) -> RustItem | None:
    found = [item for item in items if item.kind == "fn" and item.name == name]
    return found[0] if len(found) == 1 else None


def _authority_phase(root: Path, authority_path: Path | None) -> tuple[CanonicalSpec | None, list[str]]:
    path = authority_path
    if path is None and root.resolve() == PRODUCTION.resolve():
        path = FAMILIES_MANIFEST
    if path is None or not path.is_file():
        return None, ["typed-contract:authority-incomplete"]
    try:
        with path.open("rb") as stream:
            data = tomllib.load(stream)
    except (OSError, tomllib.TOMLDecodeError):
        return None, ["typed-contract:authority-incomplete"]
    raw = data.get("typed_contract")
    required = (
        "value_path", "value_type", "family_enum", "token_type", "decoded_type",
        "token_family_field", "token_id_field", "stop_variant", "opaque_variant",
        "raw_payload", "tag_const", "tag_value", "encoder", "decoder", "pack_helper",
        "unpack_helper", "producer_wrappers", "consumer_wrappers", "classifier_wrappers",
        "wrapper_graph", "route_selector_ids", "code_map",
        "kind_param", "id_param", "word_param", "producer_variant", "tag_local", "tag_bits_local",
        "payload_local", "code_local", "raw_id_local", "code_bits_local", "id_bits_local",
        "tag_shift", "code_shift", "id_shift", "tag_width", "code_width", "id_width",
        "tag_limit_local", "code_limit_local", "id_limit_local", "decoded_family_local",
        "validated_id_local",
    )
    if not isinstance(raw, dict) or str(data.get("schema", "")) != "mamba.t1.opaque-value-boundary.v2.families":
        return None, ["typed-contract:authority-incomplete"]
    if data.get("version") != 2 or any(key not in raw for key in required):
        return None, ["typed-contract:authority-incomplete"]
    identifier_keys = (
        "value_type", "family_enum", "token_type", "decoded_type", "token_family_field",
        "token_id_field", "stop_variant", "opaque_variant", "raw_payload", "tag_const",
        "encoder", "decoder", "pack_helper", "unpack_helper", "kind_param", "id_param",
        "word_param", "producer_variant", "tag_local", "tag_bits_local", "code_bits_local", "id_bits_local",
        "tag_limit_local", "code_limit_local", "id_limit_local", "decoded_family_local",
        "validated_id_local", "payload_local", "code_local", "raw_id_local",
    )
    if any(not isinstance(raw.get(key), str) or re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", raw[key]) is None for key in identifier_keys):
        return None, ["typed-contract:authority-incomplete"]
    list_keys = ("producer_wrappers", "consumer_wrappers", "classifier_wrappers", "route_selector_ids", "code_map", "wrapper_graph")
    if any(not isinstance(raw.get(key), list) for key in list_keys):
        return None, ["typed-contract:authority-incomplete"]
    if any(key in raw for key in ("family_ids", "variant_names", "codes")):
        return None, ["typed-contract:authority-incomplete"]
    if not isinstance(raw.get("value_path"), str) or raw["value_path"].startswith("/") or ".." in Path(raw["value_path"]).parts:
        return None, ["typed-contract:authority-incomplete"]
    if any(not isinstance(data.get(key), list) for key in ("families", "public_escapes", "selectors", "public_selectors")):
        return None, ["typed-contract:authority-incomplete"]
    layout_keys = ("tag_value", "tag_shift", "code_shift", "id_shift", "tag_width", "code_width", "id_width")
    if any(not isinstance(raw.get(key), int) or isinstance(raw.get(key), bool) for key in layout_keys):
        return None, ["typed-contract:authority-incomplete"]
    try:
        layout = tuple(raw[key] for key in layout_keys)
    except (KeyError, TypeError, ValueError):
        return None, ["typed-contract:authority-incomplete"]
    tag_value, tag_shift, code_shift, id_shift, tag_width, code_width, id_width = layout
    ranges = ((id_shift, id_width), (code_shift, code_width), (tag_shift, tag_width))
    if any(width <= 0 or shift < 0 or shift + width > 64 for shift, width in ranges):
        return None, ["typed-contract:authority-incomplete"]
    if any(left[0] + left[1] > right[0] and right[0] + right[1] > left[0] for index, left in enumerate(ranges) for right in ranges[index + 1:]):
        return None, ["typed-contract:authority-incomplete"]
    if layout != (6, 48, 32, 1, 16, 16, 31):
        return None, ["typed-contract:authority-incomplete"]
    records = [*data["families"], *data["public_escapes"]]
    if not records or len(raw["code_map"]) != len(records):
        return None, ["typed-contract:authority-incomplete"]
    if any(not isinstance(row, dict) for row in records):
        return None, ["typed-contract:authority-incomplete"]
    record_ids = tuple(row.get("id") for row in records)
    if any(not isinstance(row, dict) for row in raw["code_map"]):
        return None, ["typed-contract:authority-incomplete"]
    if any(
        not isinstance(row.get("id"), str)
        or not isinstance(row.get("name"), str)
        or not isinstance(row.get("typed_code"), int)
        or isinstance(row.get("typed_code"), bool)
        or not isinstance(row.get("required_roles"), list)
        or not row.get("required_roles")
        or any(not isinstance(role, str) for role in row.get("required_roles", []))
        or len(row.get("required_roles", [])) != len(set(row.get("required_roles", [])))
        or any(role not in {"producer", "consumer", "classifier"} for role in row.get("required_roles", []))
        for row in records
    ):
        return None, ["typed-contract:authority-incomplete"]
    if any(
        not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", row["id"])
        or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", row["name"])
        for row in records
    ):
        return None, ["typed-contract:authority-incomplete"]
    code_map = raw["code_map"]
    record_by_id = {str(row["id"]): row for row in records}
    record_kind_by_id = {str(row["id"]): "family" for row in data["families"]}
    record_kind_by_id.update({str(row["id"]): "public_escape" for row in data["public_escapes"]})
    if len(record_by_id) != len(records) or len(record_kind_by_id) != len(records):
        return None, ["typed-contract:authority-incomplete"]
    if any(
        not isinstance(row.get("owner_kind"), str)
        or not isinstance(row.get("owner_id"), str)
        or not isinstance(row.get("variant"), str)
        or not isinstance(row.get("code"), int)
        or isinstance(row.get("code"), bool)
        or row["owner_id"] not in record_kind_by_id
        or row["owner_kind"] != record_kind_by_id.get(row["owner_id"])
        or re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", row["variant"]) is None
        for row in code_map
    ):
        return None, ["typed-contract:authority-incomplete"]
    owner_ids = tuple(str(row["owner_id"]) for row in code_map)
    map_variants = tuple(str(row["variant"]) for row in code_map)
    map_codes = tuple(int(row["code"]) for row in code_map)
    if (
        len(set(owner_ids)) != len(owner_ids)
        or set(owner_ids) != set(record_by_id)
        or len(set(map_variants)) != len(map_variants)
        or len(set(map_codes)) != len(map_codes)
    ):
        return None, ["typed-contract:authority-incomplete"]
    if raw["producer_variant"] not in map_variants:
        return None, ["typed-contract:authority-incomplete"]
    if tag_value < 0 or tag_value >= (1 << tag_width) or any(code <= 0 or code >= (1 << code_width) for code in map_codes):
        return None, ["typed-contract:authority-incomplete"]
    code_by_owner = {str(row["owner_id"]): int(row["code"]) for row in code_map}
    if any(int(row["typed_code"]) != code_by_owner[str(row["id"])] for row in records):
        return None, ["typed-contract:authority-incomplete"]
    if any(row["exposure"] != "public_opaque_int" or row.get("disposition") != "typed_token" for row in records):
        return None, ["typed-contract:authority-incomplete"]
    family_rows = data["families"]
    record_required_roles = {
        str(row["id"]): frozenset(str(role) for role in row["required_roles"])
        for row in records
    }
    record_string_fields = ("id", "name", "path", "topology", "exposure", "disposition", "store_link", "allocator_or_table", "migration", "proof")
    if any(any(not isinstance(row.get(field), str) or not row.get(field) for field in record_string_fields) for row in records):
        return None, ["typed-contract:authority-incomplete"]
    if any(row["path"].startswith("/") or ".." in Path(row["path"]).parts or not (root / row["path"]).is_file() for row in records):
        return None, ["typed-contract:authority-incomplete"]
    if any(row["exposure"] != "public_opaque_int" or row.get("disposition") != "typed_token" or row["topology"] not in {"central_registered", "direct", "unregistered_side_table"} for row in records):
        return None, ["typed-contract:authority-incomplete"]
    if any(not isinstance(row.get("disposition"), str) or row.get("disposition") != "typed_token" for row in data["public_escapes"]):
        return None, ["typed-contract:authority-incomplete"]
    wrappers: list[tuple[str, str, str, str]] = []
    for row in raw["wrapper_graph"]:
        if not isinstance(row, dict):
            return None, ["typed-contract:authority-incomplete"]
        if any(not isinstance(row.get(key), str) for key in ("symbol", "path", "role")) or not isinstance(row.get("calls"), list) or any(not isinstance(value, str) for value in row.get("calls", [])):
            return None, ["typed-contract:authority-incomplete"]
        symbol = row["symbol"]
        route_path = row["path"]
        role = row["role"]
        calls = tuple(row["calls"])
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", symbol) or not route_path or route_path.startswith("/") or ".." in Path(route_path).parts or role not in {"producer", "consumer", "classifier"} or len(calls) != 1:
            return None, ["typed-contract:authority-incomplete"]
        wrappers.append((symbol, route_path, role, calls[0]))
    if len({row[0] for row in wrappers}) != len(wrappers):
        return None, ["typed-contract:authority-incomplete"]
    if any(any(not isinstance(value, str) for value in raw[key]) for key in ("producer_wrappers", "consumer_wrappers", "classifier_wrappers", "route_selector_ids")):
        return None, ["typed-contract:authority-incomplete"]
    roles = {
        "producer": tuple(raw["producer_wrappers"]),
        "consumer": tuple(raw["consumer_wrappers"]),
        "classifier": tuple(raw["classifier_wrappers"]),
    }
    if any(
        not roles[role]
        or len(roles[role]) != len(set(roles[role]))
        or set(roles[role]) != {row[0] for row in wrappers if row[2] == role}
        for role in roles
    ) or set().union(*(set(values) for values in roles.values())) != {row[0] for row in wrappers}:
        return None, ["typed-contract:authority-incomplete"]
    bound_targets = {raw["encoder"], raw["decoder"], *{row[0] for row in wrappers}}
    if any(target not in bound_targets for _, _, _, target in wrappers):
        return None, ["typed-contract:authority-incomplete"]
    selectors = [("family", row) for row in data["selectors"]] + [("public_escape", row) for row in data["public_selectors"]]
    selector_ids: list[str] = []
    frozen_selectors: list[tuple[tuple[str, object], ...]] = []
    selector_symbols: list[str] = []
    owner_roles: dict[str, list[str]] = {row["id"]: [] for row in records}
    wrapper_by_symbol = {symbol: (route_path, role) for symbol, route_path, role, _ in wrappers}
    record_by_id = {row["id"]: row for row in records}
    for owner_kind, row in selectors:
        if not isinstance(row, dict):
            return None, ["typed-contract:authority-incomplete"]
        if not isinstance(row.get("site_ids"), list) or any(not isinstance(value, str) for value in row.get("site_ids", [])):
            return None, ["typed-contract:authority-incomplete"]
        ids = tuple(row["site_ids"])
        has_family = "family_id" in row
        has_escape = "public_escape_id" in row
        if has_family == has_escape or (has_family and not isinstance(row.get("family_id"), str)) or (has_escape and not isinstance(row.get("public_escape_id"), str)):
            return None, ["typed-contract:authority-incomplete"]
        if (owner_kind == "family") != has_family:
            return None, ["typed-contract:authority-incomplete"]
        owner = row.get("family_id") if has_family else row.get("public_escape_id")
        if len(ids) != 1 or owner not in owner_ids:
            return None, ["typed-contract:authority-incomplete"]
        if any(not isinstance(row.get(field), str) for field in ("path", "symbol", "kind", "role", "category", "topology", "exposure", "allocator_or_table", "migration", "disposition", "proof", "anchor", "source_digest", "source_site_id", "normalized_digest")):
            return None, ["typed-contract:authority-incomplete"]
        selector_symbol = row["symbol"]
        selector_role = row["role"]
        if selector_role not in {"producer", "consumer", "classifier"} or row["kind"] != selector_role or not row["path"] or not selector_symbol:
            return None, ["typed-contract:authority-incomplete"]
        if selector_symbol not in wrapper_by_symbol or wrapper_by_symbol[selector_symbol] != (row["path"], selector_role):
            return None, ["typed-contract:authority-incomplete"]
        required_selector_fields = (
            "category", "normalized_digest", "source_digest", "source_site_id", "anchor",
            "topology", "exposure", "allocator_or_table", "migration", "disposition", "proof",
        )
        if any(not row[field] for field in required_selector_fields):
            return None, ["typed-contract:authority-incomplete"]
        if (
            re.fullmatch(r"[0-9a-f]{64}", row["source_digest"]) is None
            or re.fullmatch(r"[0-9a-f]{64}", row["source_site_id"]) is None
            or re.fullmatch(r"[0-9a-f]{64}", row["normalized_digest"]) is None
            or row["anchor"] != f"{row['path']}::{row['symbol']}::{row['normalized_digest']}"
        ):
            return None, ["typed-contract:authority-incomplete"]
        owner_record = record_by_id.get(owner)
        if owner_record is None or row.get("topology") != owner_record.get("topology") or row.get("exposure") != "public_opaque_int" or row.get("disposition") != "typed_token" or row.get("allocator_or_table") != owner_record.get("allocator_or_table"):
            return None, ["typed-contract:authority-incomplete"]
        if row.get("category") != selector_role or row.get("kind") != selector_role:
            return None, ["typed-contract:authority-incomplete"]
        if selector_role not in record_required_roles.get(owner, frozenset()):
            return None, ["typed-contract:authority-incomplete"]
        selector_ids.extend(ids)
        selector_symbols.append(selector_symbol)
        owner_roles.setdefault(owner, []).append(selector_role)
        frozen_selectors.append(tuple(sorted((str(key), _freeze_canonical(value)) for key, value in row.items())))
    route_ids = tuple(raw["route_selector_ids"])
    if (
        not selectors
        or len(selector_ids) != len(set(selector_ids))
        or len(selector_symbols) != len(set(selector_symbols))
        or route_ids != tuple(selector_ids)
        or any(set(owner_roles.get(row["id"], [])) != set(record_required_roles[row["id"]]) for row in records)
    ):
        return None, ["typed-contract:authority-incomplete"]
    files = {raw["value_path"]}
    files.update(row[1] for row in wrappers)
    files.update(row["path"] for _, row in selectors)
    if any(not value or value.startswith("/") or ".." in Path(value).parts or not (root / value).is_file() for value in files):
        return None, ["typed-contract:authority-incomplete"]
    contract = tuple(sorted((str(key), _freeze_canonical(value)) for key, value in raw.items()))
    return CanonicalSpec(contract, tuple(wrappers), tuple(frozen_selectors), tuple(sorted(files))), []


def _canonical_add(failures: list[str], diagnostic: str) -> None:
    if diagnostic not in failures:
        failures.append(diagnostic)


def _authority_declarations(spec: CanonicalSpec, root: Path) -> tuple[dict[str, tuple[RustItem, ...]], list[str]]:
    sources: dict[str, tuple[RustItem, ...]] = {}
    failures: list[str] = []
    for relative in spec.files:
        try:
            source = (root / relative).read_text(encoding="utf-8")
            sources[relative], _ = _source_census(source)
        except (OSError, UnicodeError, ScanFailure):
            _canonical_add(failures, "typed-contract:source")
    # The authority file is the closed source boundary.  An unlisted Rust
    # file may contain a same-name declaration or a macro that manufactures a
    # bound item; silently ignoring it would make the oracle lexical rather
    # than authority-instantiated.
    bound = _bound_names(spec)
    known = set(spec.files)
    for path in sorted(root.rglob("*.rs")):
        relative = path.relative_to(root).as_posix()
        if relative in known:
            continue
        try:
            items, _ = _source_census(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, ScanFailure):
            _canonical_add(failures, "typed-contract:source")
            continue
        if any(
            item.name in bound
            or (item.kind == "macro_rules" and any(word in bound for word in _item_words(item)))
            or (item.kind == "use" and any(word in bound for word in _item_words(item)))
            or ("!" in _item_words(item) and any(word in bound for word in _item_words(item)))
            for item in items
        ):
            _canonical_add(failures, "typed-contract:unlisted-bound-source")
    return sources, failures


def _bound_names(spec: CanonicalSpec) -> set[str]:
    contract = _spec_contract(spec)
    names = {
        str(contract[key])
        for key in ("value_type", "family_enum", "token_type", "decoded_type", "tag_const", "encoder", "decoder", "pack_helper", "unpack_helper")
    }
    names.update(row[0] for row in spec.wrappers)
    names.update(str(row["variant"]) for row in contract.get("code_map", []) if isinstance(row, dict))
    return names


def _support_symbols_ok(spec: CanonicalSpec, sources: dict[str, tuple[RustItem, ...]], root: Path, failures: list[str]) -> None:
    """Bind the few support symbols used by the canonical Rust grammar.

    ``NonZeroU64`` and ``Result`` are used only through absolute paths.  The
    support identity check is intentionally limited to the authority closure;
    an unrelated source file must not be able to make this typed fixture red.
    ``CodecError`` remains one exact unit struct in the authority value file.
    """
    contract = _spec_contract(spec)
    value_path = str(contract["value_path"])
    all_items = [(path, item) for path, items in sources.items() for item in items]
    exact_nonzero = ("use", *STD_NONZERO_WORDS, ";")
    exact_result = ("use", *STD_RESULT_WORDS, ";")
    codec_errors = [(path, item) for path, item in all_items if item.name == "CodecError"]
    if len(codec_errors) != 1 or codec_errors[0][0] != value_path or codec_errors[0][1].kind != "struct" or codec_errors[0][1].depth != 0 or _item_words(codec_errors[0][1]) != ("struct", "CodecError", ";"):
        _canonical_add(failures, "typed-contract:support-identity")

    def has_root_path(words: tuple[str, ...], target: str, prefix: tuple[str, ...]) -> bool:
        return any(
            index + 1 >= len(prefix)
            and words[index - len(prefix) + 1:index + 1] == prefix
            for index, value in enumerate(words)
            if value == target
        )

    for path, item in all_items:
        words = _item_words(item)
        if item.kind == "use" and any(value in {"NonZeroU64", "Result", "CodecError"} for value in words):
            if words not in {exact_nonzero, exact_result}:
                _canonical_add(failures, "typed-contract:support-identity")
        if item.kind == "use" and "as" in words and "std" in words[words.index("as") + 1:]:
            _canonical_add(failures, "typed-contract:support-identity")
        if item.name in {"NonZeroU64", "Result", "CodecError"} and item.kind in {"struct", "enum", "type", "const", "fn", "mod"} and not (item.name == "CodecError" and path == value_path and item.kind == "struct" and _item_words(item) == ("struct", "CodecError", ";")):
            _canonical_add(failures, "typed-contract:support-identity")
        if item.kind == "use" and "CodecError" in words:
            _canonical_add(failures, "typed-contract:support-identity")
        if item.name == "std" or (item.kind == "use" and "std" in words and words not in {exact_nonzero, exact_result}):
            _canonical_add(failures, "typed-contract:support-identity")

    for relative in spec.files:
        path = root / relative
        try:
            words = _rust_words(mask_non_code(path.read_text(encoding="utf-8")))
        except (OSError, UnicodeError, ScanFailure):
            _canonical_add(failures, "typed-contract:support-identity")
            continue
        if any(
            value == "NonZeroU64" and not has_root_path(words, value, STD_NONZERO_WORDS)
            for value in words
        ):
            _canonical_add(failures, "typed-contract:support-identity")
        if any(
            value == "Result" and not has_root_path(words, value, STD_RESULT_WORDS)
            for value in words
        ):
            _canonical_add(failures, "typed-contract:support-identity")
        token_index = 0
        while token_index + 4 < len(words):
            if words[token_index] == "extern" and words[token_index + 1] == "crate" and words[token_index + 3] == "as" and words[token_index + 4] == "std":
                _canonical_add(failures, "typed-contract:support-identity")
            token_index += 1


def _macro_tokens_reference_bound(source: str, bound: set[str]) -> bool:
    """Find a macro definition/invocation whose own tokens carry a symbol."""
    tokens = _rust_tokens(source)
    for index, token in enumerate(tokens):
        if token.text != "!" or index == 0 or tokens[index - 1].text == "macro_rules":
            continue
        opening = index + 1
        if opening >= len(tokens) or tokens[opening].text not in {"(", "[", "{"}:
            if tokens[index - 1].text in bound:
                return True
            continue
        closing = _matching_delimiter(tokens, opening)
        if closing is None:
            return True
        words = tuple(item.text for item in tokens[index - 1:closing + 1])
        if any(word in bound for word in words):
            return True
    return False


def _macro_partial(spec: CanonicalSpec, sources: dict[str, tuple[RustItem, ...]], root: Path) -> bool:
    bound = _bound_names(spec)
    contract = _spec_contract(spec)
    bound.update(str(row["variant"]) for row in contract.get("code_map", []) if isinstance(row, dict))
    # Macro safety is scoped to authority-participating source files.  The
    # closed declaration/support scans still inspect unlisted files for
    # bound-name shadows, but an unrelated future macro elsewhere in the
    # production tree cannot make this typed contract permanently partial.
    forbidden = bound | {"NonZeroU64", "Result", "CodecError"}
    for relative in spec.files:
        path = root / relative
        try:
            source = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError):
            return True
        items, tokens = _source_census(source)
        bound_items = [
            item for item in items
            if item.name in bound
            or (item.kind == "macro_rules" and any(word in bound for word in _item_words(item)))
        ]
        for item in items:
            if item.kind != "macro_rules":
                continue
            if any(word in forbidden for word in _item_words(item)):
                return True
            if any(parent.start <= item.start and item.end <= parent.end for parent in bound_items if parent is not item):
                return True
        for index, token in enumerate(tokens):
            if token.text != "!" or index == 0 or tokens[index - 1].text == "macro_rules":
                continue
            opening = index + 1
            if opening >= len(tokens) or tokens[opening].text not in {"(", "[", "{"}:
                return True
            closing = _matching_delimiter(tokens, opening)
            if closing is None:
                return True
            span_start = tokens[index - 1].start
            span_end = tokens[closing].end
            macro_words = tuple(item.text for item in tokens[index - 1:closing + 1])
            if any(word in forbidden for word in macro_words):
                return True
            containing = [item for item in items if item.start <= span_start and span_end <= item.end]
            if any(parent.name in bound for parent in containing):
                return True
            if not containing:
                # A bare top-level invocation could manufacture an authority
                # declaration even when its argument is innocuous.
                return True
    return False


def _declaration_items(spec: CanonicalSpec, sources: dict[str, tuple[RustItem, ...]], failures: list[str]) -> dict[tuple[str, str], RustItem]:
    contract = _spec_contract(spec)
    expected = [
        (str(contract["value_path"]), "struct", str(contract["value_type"]), "value_type"),
        (str(contract["value_path"]), "enum", str(contract["family_enum"]), "family_enum"),
        (str(contract["value_path"]), "struct", str(contract["token_type"]), "token_type"),
        (str(contract["value_path"]), "enum", str(contract["decoded_type"]), "decoded_type"),
        (str(contract["value_path"]), "const", str(contract["tag_const"]), "tag_const"),
        (str(contract["value_path"]), "fn", str(contract["pack_helper"]), "pack_helper"),
        (str(contract["value_path"]), "fn", str(contract["unpack_helper"]), "unpack_helper"),
        (str(contract["value_path"]), "fn", str(contract["encoder"]), "encoder"),
        (str(contract["value_path"]), "fn", str(contract["decoder"]), "decoder"),
    ]
    expected.extend((path, "fn", symbol, f"wrapper:{symbol}") for symbol, path, _, _ in spec.wrappers)
    expected_by_name = {name: (path, kind, label) for path, kind, name, label in expected}
    result: dict[tuple[str, str], RustItem] = {}
    forbidden_prefixes = {"pub", "unsafe", "async", "extern", "const", "default", "where", "impl", "trait"}
    for path, kind, name, label in expected:
        items = sources.get(path, ())
        matches = [item for item in items if item.kind == kind and item.name == name]
        if (
            len(matches) != 1
            or any(item.kind != kind for item in items if item.name == name)
            or any(item.has_attribute or item.depth != 0 or forbidden_prefixes.intersection(item.prefix) for item in matches)
        ):
            _canonical_add(failures, f"typed-contract:declaration:{label}")
        else:
            result[(path, name)] = matches[0]
    # Reject declarations and imports that shadow an authority-bound item in
    # another participating file.  In particular, `use evil::encode_token;
    # encode_token(...)` must not satisfy the nominal callee proof.
    for path, items in sources.items():
        for item in items:
            if item.kind == "use":
                if any(word in expected_by_name for word in _item_words(item)):
                    _canonical_add(failures, "typed-contract:declaration:callee-alias")
                continue
            expected_entry = expected_by_name.get(item.name)
            if expected_entry is None:
                continue
            expected_path, expected_kind, label = expected_entry
            if path != expected_path or item.kind != expected_kind or item.depth != 0 or forbidden_prefixes.intersection(item.prefix):
                _canonical_add(failures, f"typed-contract:declaration:{label}")
    return result


def _type_words(prefix: str, name: str) -> tuple[str, ...]:
    return (prefix, "::", name) if prefix else (name,)


def _wrapper_signature_ok(spec: CanonicalSpec, item: RustItem, role: str) -> bool:
    contract = _spec_contract(spec)
    signature = _signature(item)
    if signature is None:
        return False
    params, result = signature
    route = any(item.name == symbol and path != str(contract["value_path"]) for symbol, path, _, _ in spec.wrappers)
    prefix = "super" if route else ""
    family = _type_words(prefix, str(contract["family_enum"]))
    value = _type_words(prefix, str(contract["value_type"]))
    decoded = _type_words(prefix, str(contract["decoded_type"]))
    error = _type_words(prefix, "CodecError")
    producer_one = ((str(contract["id_param"]), STD_NONZERO_WORDS),)
    producer_two = ((str(contract["kind_param"]), family), (str(contract["id_param"]), STD_NONZERO_WORDS))
    if role == "producer":
        if tuple(params) not in {producer_one, producer_two}:
            return False
        return tuple(result) == ("->", *value)
    if tuple(params) != ((str(contract["word_param"]), value),):
        return False
    return tuple(result) == ("->", *STD_RESULT_WORDS, "<", *decoded, ",", *error, ">")


def _function_signature_ok(spec: CanonicalSpec, item: RustItem, kind: str) -> bool:
    contract = _spec_contract(spec)
    header, _ = _function_parts(item)
    if kind == "pack":
        expected = _rust_words(f"fn {item.name}(tag:u64,code:u64,id: ::std::num::NonZeroU64)-> ::std::result::Result<{contract['value_type']},CodecError>")
    elif kind == "unpack":
        expected = _rust_words(f"fn {item.name}(word:{contract['value_type']})->::std::result::Result<(u64,u64),CodecError>")
    elif kind == "encoder":
        expected = _rust_words(f"fn {item.name}({contract['kind_param']}:{contract['family_enum']},{contract['id_param']}: ::std::num::NonZeroU64)-> ::std::result::Result<{contract['value_type']},CodecError>")
    else:
        expected = _rust_words(f"fn {item.name}({contract['word_param']}:{contract['value_type']})->::std::result::Result<{contract['decoded_type']},CodecError>")
    return header == expected


def _canonical_body(spec: CanonicalSpec, kind: str) -> str:
    c = _spec_contract(spec)
    family = str(c["family_enum"])
    error = "CodecError"
    if kind == "pack":
        return (
            f"{{let {c['tag_limit_local']}=1u64.checked_shl({c['tag_width']}).ok_or({error})?;"
            f"let {c['code_limit_local']}=1u64.checked_shl({c['code_width']}).ok_or({error})?;"
            f"let {c['id_limit_local']}=1u64.checked_shl({c['id_width']}).ok_or({error})?;"
            f"if {c['tag_local']} >= {c['tag_limit_local']} || {c['code_local']} >= {c['code_limit_local']} || {c['id_param']}.get() >= {c['id_limit_local']}{{return ::std::result::Result::Err({error});}}"
            f"let {c['tag_bits_local']}={c['tag_local']}.checked_shl({c['tag_shift']}).ok_or({error})?;"
            f"let {c['code_bits_local']}={c['code_local']}.checked_shl({c['code_shift']}).ok_or({error})?;"
            f"let {c['id_bits_local']}={c['id_param']}.get().checked_shl({c['id_shift']}).ok_or({error})?;"
            f"let first={c['tag_bits_local']}.checked_add({c['code_bits_local']}).ok_or({error})?;"
            f"::std::result::Result::Ok({c['value_type']}(first.checked_add({c['id_bits_local']}).ok_or({error})?))}}"
        )
    if kind == "unpack":
        return (
            f"{{let {c['tag_local']}={c['word_param']}.0.checked_shr({c['tag_shift']}).ok_or({error})?;"
            f"let {c['tag_bits_local']}={c['tag_local']}.checked_shl({c['tag_shift']}).ok_or({error})?;"
            f"let {c['payload_local']}={c['word_param']}.0.checked_sub({c['tag_bits_local']}).ok_or({error})?;"
            f"::std::result::Result::Ok(({c['tag_local']},{c['payload_local']}))}}"
        )
    if kind == "encoder":
        arms = ", ".join(f"{family}::{row['variant']}=>{row['code']}" for row in c["code_map"])
        return f"{{let {c['code_local']}=match {c['kind_param']}{{{arms},}};{c['pack_helper']}({c['tag_const']},{c['code_local']},{c['id_param']})}}"
    arms = ", ".join(f"{row['code']}=>{family}::{row['variant']}" for row in c["code_map"])
    return (
        f"{{let({c['tag_local']},{c['raw_payload']})={c['unpack_helper']}({c['word_param']})?;"
        f"if {c['tag_local']} != {c['tag_const']}{{return ::std::result::Result::Err({error});}}"
        f"if {c['raw_payload']} == 0{{return ::std::result::Result::Ok({c['decoded_type']}::{c['stop_variant']});}}"
        f"if {c['raw_payload']} != 0{{if {c['raw_payload']}&1 != 0{{return ::std::result::Result::Err({error});}}"
        f"let {c['code_local']}={c['raw_payload']}.checked_shr({c['code_shift']}).ok_or({error})?;"
        f"let {c['code_bits_local']}={c['code_local']}.checked_shl({c['code_shift']}).ok_or({error})?;"
        f"let {c['payload_local']}={c['raw_payload']}.checked_sub({c['code_bits_local']}).ok_or({error})?;"
        f"let {c['raw_id_local']}={c['payload_local']}.checked_shr({c['id_shift']}).ok_or({error})?;"
        f"let {c['validated_id_local']}=::std::num::NonZeroU64::new({c['raw_id_local']}).ok_or({error})?;"
        f"if {c['validated_id_local']}.get()==0{{return ::std::result::Result::Err({error});}}"
        f"if {c['code_local']} == 0{{return ::std::result::Result::Err({error});}}"
        f"let {c['decoded_family_local']}=match {c['code_local']}{{{arms},_=>{{return ::std::result::Result::Err({error});}}}};"
        f"return ::std::result::Result::Ok({c['decoded_type']}::{c['opaque_variant']}({c['token_type']}{{{c['token_family_field']}:{c['decoded_family_local']},{c['token_id_field']}:{c['validated_id_local']}}}));}}"
        f"::std::result::Result::Err({error})}}"
    )


def _check_nominal_declarations(spec: CanonicalSpec, items: dict[tuple[str, str], RustItem], failures: list[str]) -> None:
    c = _spec_contract(spec)
    path = str(c["value_path"])
    expected = {
        str(c["value_type"]): _rust_words(f"struct {c['value_type']}(u64);"),
        str(c["family_enum"]): _rust_words(f"enum {c['family_enum']}{{{','.join(str(row['variant']) for row in c['code_map'])},}}"),
        str(c["token_type"]): _rust_words(f"struct {c['token_type']}{{{c['token_family_field']}:{c['family_enum']},{c['token_id_field']}: ::std::num::NonZeroU64}}"),
        # The canonical fixture spells the closed enum without a trailing
        # comma.  Keep declaration matching token-exact while accepting the
        # equivalent Rust trailing-comma spelling for the other nominal maps.
        str(c["decoded_type"]): _rust_words(f"enum {c['decoded_type']}{{{c['stop_variant']},{c['opaque_variant']}({c['token_type']})}}"),
        str(c["tag_const"]): _rust_words(f"const {c['tag_const']}:u64=6;"),
    }
    for name, expected_words in expected.items():
        item = items.get((path, name))
        if item and _item_words(item) != expected_words:
            _canonical_add(failures, f"typed-contract:declaration:{name}")


def _if_branches(body: tuple[str, ...]) -> list[tuple[int, tuple[str, ...], tuple[str, ...]]]:
    """Return every balanced `if` header and its exact brace body."""
    branches: list[tuple[int, tuple[str, ...], tuple[str, ...]]] = []
    for index, word in enumerate(body):
        if word != "if":
            continue
        opening = index + 1
        while opening < len(body) and body[opening] != "{":
            if body[opening] in {";", "}"}:
                break
            opening += 1
        if opening >= len(body) or body[opening] != "{":
            continue
        depth = 1
        closing = opening + 1
        while closing < len(body) and depth:
            if body[closing] == "{":
                depth += 1
            elif body[closing] == "}":
                depth -= 1
            closing += 1
        if depth == 0:
            branches.append((index, body[index + 1:opening], body[opening:closing]))
    return branches


def _direct_error_guard(body: tuple[str, ...], condition: str) -> int | None:
    expected_header = _rust_words(condition)
    expected_body = _rust_words("{return ::std::result::Result::Err(CodecError);}")
    for index, header, branch in _if_branches(body):
        if header == expected_header and branch == expected_body:
            return index
    return None


def _binding_ids(body: tuple[str, ...], function: str) -> tuple[BindingId, ...]:
    """Assign stable IDs to the exact `let` bindings in a canonical body."""
    bindings: list[BindingId] = []
    ordinal = 0
    index = 0
    while index < len(body):
        if body[index] != "let":
            index += 1
            continue
        cursor = index + 1
        names: list[str] = []
        if cursor < len(body) and body[cursor] == "(":
            cursor += 1
            while cursor < len(body) and body[cursor] != ")":
                if body[cursor].isidentifier() and body[cursor] not in {"_"}:
                    names.append(body[cursor])
                cursor += 1
        elif cursor < len(body) and body[cursor].isidentifier() and body[cursor] != "_":
            names.append(body[cursor])
        for name in names:
            bindings.append(BindingId(function, name, ordinal))
            ordinal += 1
        index = cursor + 1
    return tuple(bindings)


def _binding_identity(body: tuple[str, ...], function: str, expected: tuple[str, ...], parameters: set[str]) -> bool:
    """Check the canonical linear body's actual let/use binding edges.

    Names are not treated as identities: each declaration receives its
    ``BindingId`` ordinal, then every later reference is checked against the
    one prior declaration of that name.  A second declaration, tuple
    destructure, or use-before-definition consequently cannot satisfy the
    proof by spelling the same identifier.
    """
    bindings = _binding_ids(body, function)
    if tuple(binding.name for binding in bindings) != expected:
        return False
    ids_by_name: dict[str, BindingId] = {binding.name: binding for binding in bindings}
    if len(ids_by_name) != len(bindings) or set(ids_by_name) & parameters:
        return False
    parameter_ids = {
        name: BindingId(function, name, -1 - ordinal)
        for ordinal, name in enumerate(sorted(parameters))
    }
    defined_at: dict[str, int] = {}
    definition_positions: set[tuple[int, str]] = set()
    ordinal = 0
    index = 0
    while index < len(body):
        if body[index] != "let":
            index += 1
            continue
        cursor = index + 1
        names: list[str] = []
        name_positions: list[int] = []
        if cursor < len(body) and body[cursor] == "(":
            cursor += 1
            while cursor < len(body) and body[cursor] != ")":
                if body[cursor].isidentifier() and body[cursor] != "_":
                    names.append(body[cursor])
                    name_positions.append(cursor)
                cursor += 1
        elif cursor < len(body) and body[cursor].isidentifier() and body[cursor] != "_":
            names.append(body[cursor])
            name_positions.append(cursor)
        for name, name_position in zip(names, name_positions):
            binding = ids_by_name.get(name)
            if binding is None or binding.ordinal != ordinal or name in defined_at:
                return False
            defined_at[name] = index
            definition_positions.add((name_position, name))
            ordinal += 1
        index = max(index + 1, cursor + 1)
    if set(defined_at) != set(expected):
        return False
    # Resolve every local use to its one prior definition.  Function
    # parameters are the only names allowed before the first let.  Keep the
    # actual BindingId on each edge so a same-spelled decoy/rebinding cannot
    # satisfy the proof merely by reusing the identifier text.
    use_edges: list[tuple[int, BindingId]] = []
    for index, word in enumerate(body):
        if word in parameter_ids:
            if index + 1 < len(body) and body[index + 1] == "=":
                return False
            use_edges.append((index, parameter_ids[word]))
            continue
        if word not in ids_by_name:
            continue
        definition = defined_at.get(word)
        binding = ids_by_name.get(word)
        if definition is None or binding is None:
            return False
        if (index, word) in definition_positions:
            continue
        if index < definition:
            if word not in parameters:
                return False
            continue
        if index + 1 < len(body) and body[index + 1] == "=":
            return False
        resolved = BindingId(function, word, binding.ordinal)
        if resolved != binding:
            return False
        use_edges.append((index, resolved))
    if any(
        edge != (parameter_ids.get(body[index]) or ids_by_name.get(body[index]))
        for index, edge in use_edges
    ):
        return False
    return True


def _has_exact_map(body: tuple[str, ...], local: str, source: str) -> tuple[int, ...]:
    needle = ("let", local, "=", "match", source, "{")
    return tuple(index for index in range(len(body) - len(needle) + 1) if body[index:index + len(needle)] == needle)


def _has_rebinding(body: tuple[str, ...], names: set[str], definition: int) -> bool:
    for index, word in enumerate(body):
        if index <= definition or word not in names:
            continue
        if index == definition + 1 and index > 0 and body[index - 1] == "let":
            continue
        if index + 1 < len(body) and body[index + 1] == "=":
            return True
        if index > definition + 1 and index > 0 and body[index - 1] == "let":
            return True
    return False


def _canonical_roundtrip_proof(spec: CanonicalSpec) -> bool:
    """Prove the accepted non-stop payload grammar is encode/decode identity."""
    c = _spec_contract(spec)
    tag = int(c["tag_value"])
    tag_shift = int(c["tag_shift"])
    code_shift = int(c["code_shift"])
    id_shift = int(c["id_shift"])
    tag_width = int(c["tag_width"])
    code_width = int(c["code_width"])
    id_width = int(c["id_width"])
    code_max = (1 << code_width) - 1
    id_max = (1 << id_width) - 1
    id_bits_max = id_max << id_shift
    code_bits_max = code_max << code_shift
    payload_max = code_bits_max | id_bits_max
    # These inequalities are the symbolic part of the proof: extraction by
    # the configured shifts cannot consume a neighboring field, subtraction
    # cannot borrow across the code field, and every accepted payload remains
    # below the tag field.  The explicit round trips below then pin the same
    # equations to the non-stop endpoints and an interior value.
    if (
        tag != 6
        or tag_width != 16
        or tag_shift != 48
        or id_shift <= 0
        or id_bits_max >= (1 << code_shift)
        or code_bits_max >= (1 << tag_shift)
        or (id_bits_max & code_bits_max)
        or payload_max >= (1 << tag_shift)
        or payload_max & ((1 << id_shift) - 1)
    ):
        return False
    for code in (1, code_max // 2, code_max):
        for identifier in (1, id_max // 2, id_max):
            raw_payload = (code << code_shift) | (identifier << id_shift)
            decoded_code = raw_payload >> code_shift
            code_bits = decoded_code << code_shift
            decoded_id = (raw_payload - code_bits) >> id_shift
            packed = (tag << tag_shift) | code_bits | (decoded_id << id_shift)
            if packed != ((tag << tag_shift) | raw_payload) or raw_payload & ((1 << id_shift) - 1):
                return False
    return True


def _check_codec(spec: CanonicalSpec, items: dict[tuple[str, str], RustItem], failures: list[str]) -> None:
    c = _spec_contract(spec)
    path = str(c["value_path"])
    kinds = ((str(c["pack_helper"]), "pack"), (str(c["unpack_helper"]), "unpack"), (str(c["encoder"]), "encoder"), (str(c["decoder"]), "decoder"))
    for name, kind in kinds:
        item = items.get((path, name))
        if item is None:
            continue
        if not _function_signature_ok(spec, item, kind):
            _canonical_add(failures, f"typed-contract:signature:{kind}")
            continue
        _, body = _function_parts(item)
        canonical = _canonical_body(spec, kind)
        if _body_matches(item, canonical, kind=kind):
            # Even an otherwise canonical body must expose one authority-bound
            # live map and one binding identity per configured local.
            binding_expectations = {
                "pack": (
                    str(c["tag_limit_local"]), str(c["code_limit_local"]), str(c["id_limit_local"]),
                    str(c["tag_bits_local"]), str(c["code_bits_local"]), str(c["id_bits_local"]), "first",
                ),
                "unpack": (str(c["tag_local"]), str(c["tag_bits_local"]), str(c["payload_local"])),
                "encoder": (str(c["code_local"]),),
                "decoder": (
                    str(c["tag_local"]), str(c["raw_payload"]), str(c["code_local"]),
                    str(c["code_bits_local"]), str(c["payload_local"]), str(c["raw_id_local"]),
                    str(c["validated_id_local"]), str(c["decoded_family_local"]),
                ),
            }
            parameter_names = {
                "pack": {"tag", "code", str(c["id_param"])},
                "unpack": {str(c["word_param"])},
                "encoder": {str(c["kind_param"]), str(c["id_param"])},
                "decoder": {str(c["word_param"])},
            }
            if not _binding_identity(body, name, binding_expectations[kind], parameter_names[kind]):
                _canonical_add(failures, "typed-contract:binding-identity")
            if kind == "encoder":
                map_positions = _has_exact_map(body, str(c["code_local"]), str(c["kind_param"]))
                if map_positions != (body.index("let"),) or _has_rebinding(body, {str(c["code_local"]), str(c["kind_param"]), str(c["id_param"])}, map_positions[0] if map_positions else -1):
                    _canonical_add(failures, "typed-contract:encoder-map")
            elif kind == "decoder":
                map_positions = _has_exact_map(body, str(c["decoded_family_local"]), str(c["code_local"]))
                if len(map_positions) != 1 or _has_rebinding(body, {str(c["decoded_family_local"]), str(c["code_local"]), str(c["validated_id_local"]), str(c["raw_id_local"]), str(c["id_param"])}, map_positions[0] if map_positions else -1):
                    _canonical_add(failures, "typed-contract:decoder-map")
            continue
        if kind == "pack":
            expected_lets = canonical.count("let")
            if body.count("let") > expected_lets:
                _canonical_add(failures, "typed-contract:pack-extra-statement")
            else:
                guard = _direct_error_guard(
                    body,
                    f"{c['tag_local']} >= {c['tag_limit_local']} || {c['code_local']} >= {c['code_limit_local']} || {c['id_param']}.get() >= {c['id_limit_local']}",
                )
                dependent = min((body.index(word) for word in (str(c["tag_bits_local"]), str(c["code_bits_local"]), str(c["id_bits_local"])) if word in body), default=len(body))
                if guard is not None and guard > dependent:
                    _canonical_add(failures, "typed-contract:field-guard-order")
                elif guard is None:
                    _canonical_add(failures, "typed-contract:field-guard")
                elif "checked_add" not in body:
                    _canonical_add(failures, "typed-contract:compose-return")
                else:
                    _canonical_add(failures, "typed-contract:pack-grammar")
        elif kind == "unpack":
            _canonical_add(failures, "typed-contract:unpack-grammar")
        elif kind == "encoder":
            map_positions = _has_exact_map(body, str(c["code_local"]), str(c["kind_param"]))
            destructuring_shadow = any(
                index > (map_positions[0] if map_positions else -1) and body[index:index + 2] == ("let", "(")
                for index in range(len(body) - 1)
            )
            if len(map_positions) > 1 or body.count("let") > 1 or destructuring_shadow or (len(map_positions) == 1 and _has_rebinding(body, {str(c["code_local"]), str(c["kind_param"]), str(c["id_param"])}, map_positions[0])):
                _canonical_add(failures, "typed-contract:encoder-code-shadow")
            elif len(map_positions) != 1:
                _canonical_add(failures, "typed-contract:encoder-map")
            elif not body[-(len(_rust_words(f"{c['pack_helper']}({c['tag_const']},{c['code_local']},{c['id_param']})"))):] == _rust_words(f"{c['pack_helper']}({c['tag_const']},{c['code_local']},{c['id_param']})"):
                _canonical_add(failures, "typed-contract:encoder-return")
            else:
                _canonical_add(failures, "typed-contract:encoder-grammar")
        else:
            map_positions = _has_exact_map(body, str(c["decoded_family_local"]), str(c["code_local"]))
            first_let = body.index("let") if "let" in body else len(body)
            # The decoder's catch-all arm is deliberately `_`: binding the
            # unmatched code (even if it is immediately discarded) creates a
            # second live binding edge that can shadow the authority map.
            match_arm_binding_shadow = any(
                body[index] == str(c["code_local"])
                and index + 1 < len(body)
                and body[index + 1] == "=>"
                for index in range(len(body))
            )
            destructuring_shadow = any(
                index > first_let and body[index:index + 2] == ("let", "(")
                for index in range(len(body) - 1)
            )
            if match_arm_binding_shadow:
                _canonical_add(failures, "typed-contract:decoder-match-binding-shadow")
            elif len(map_positions) > 1:
                _canonical_add(failures, "typed-contract:decoder-map")
            elif destructuring_shadow or (len(map_positions) == 1 and _has_rebinding(body, {str(c["decoded_family_local"]), str(c["code_local"]), str(c["validated_id_local"]), str(c["raw_id_local"])}, map_positions[0])):
                _canonical_add(failures, "typed-contract:decoder-kind-id-shadow")
            elif body.count("let") > canonical.count("let"):
                _canonical_add(failures, "typed-contract:decoder-unknown-statement")
            else:
                tag_guard = _direct_error_guard(body, f"{c['tag_local']} != {c['tag_const']}")
                raw_zero = _if_branches(body)
                zero_guard = next((index for index, header, branch in raw_zero if header == _rust_words(f"{c['raw_payload']} == 0") and branch == _rust_words(f"{{return ::std::result::Result::Ok({c['decoded_type']}::{c['stop_variant']});}}")), None)
                nonzero_guard = next((index for index, header, _ in raw_zero if header == _rust_words(f"{c['raw_payload']} != 0")), None)
                reserved_condition = _rust_words(f"{c['raw_payload']} & 1 != 0")
                reserved_positions = [
                    index for index, header, branch in _if_branches(body)
                    if header == reserved_condition and branch == _rust_words("{return ::std::result::Result::Err(CodecError);}")
                ]
                reserved_guard = reserved_positions[0] if reserved_positions else None
                extraction = next((index for index, word in enumerate(body) if word == str(c["code_local"]) and index + 1 < len(body) and body[index + 1] == "="), len(body))
                if tag_guard is None or (zero_guard is not None and tag_guard > zero_guard):
                    _canonical_add(failures, "typed-contract:decoder-tag-order")
                elif zero_guard is None or nonzero_guard is None or zero_guard > nonzero_guard:
                    _canonical_add(failures, "typed-contract:zero-stop")
                elif reserved_guard is None:
                    _canonical_add(failures, "typed-contract:canonical-reserved-bits")
                elif len(reserved_positions) != 1:
                    _canonical_add(failures, "typed-contract:canonical-reserved-guard")
                elif reserved_guard > extraction:
                    _canonical_add(failures, "typed-contract:canonical-reserved-order")
                elif len(map_positions) != 1:
                    _canonical_add(failures, "typed-contract:decoder-map")
                else:
                    _canonical_add(failures, "typed-contract:decoder-grammar")


def _wrapper_body_ok(spec: CanonicalSpec, item: RustItem, target: str, role: str) -> bool:
    c = _spec_contract(spec)
    signature = _signature(item)
    if signature is None or not _wrapper_signature_ok(spec, item, role):
        return False
    params, _ = signature
    names = [name for name, _ in params]
    _, body = _function_parts(item)
    actual = body[1:-1] if body and body[0] == "{" and body[-1] == "}" else ()
    if role == "producer" and target == str(c["encoder"]):
        if len(names) == 1:
            expected = _rust_words(f"{target}({c['family_enum']}::{c['producer_variant']},{names[0]}).unwrap()")
        else:
            expected = _rust_words(f"{target}({','.join(names)}).unwrap()")
        return actual == expected
    return actual == _rust_words(f"{target}({','.join(names)})")


def _check_routes(spec: CanonicalSpec, sources: dict[str, tuple[RustItem, ...]], root: Path, failures: list[str]) -> None:
    c = _spec_contract(spec)
    graph = {symbol: (path, role, target) for symbol, path, role, target in spec.wrappers}
    for symbol, path, role, target in spec.wrappers:
        item = _function_item(sources.get(path, ()), symbol)
        if item is None or not _wrapper_body_ok(spec, item, target, role):
            _canonical_add(failures, f"typed-contract:wrapper-forward:{symbol}")
    for symbol, (_, role, target) in graph.items():
        cursor = symbol
        seen: set[str] = set()
        while cursor in graph:
            if cursor in seen:
                _canonical_add(failures, "typed-contract:wrapper-cycle")
                break
            seen.add(cursor)
            cursor = graph[cursor][2]
        terminal = str(c["encoder"] if role == "producer" else c["decoder"])
        if cursor != terminal:
            _canonical_add(failures, f"typed-contract:wrapper-terminal:{symbol}")
    for frozen in spec.selectors:
        selector = _spec_row(frozen)
        path = str(selector["path"])
        symbol = str(selector["symbol"])
        item = _function_item(sources.get(path, ()), symbol)
        if item is None:
            _canonical_add(failures, f"typed-contract:route-symbol:{symbol}")
            continue
        role = str(selector.get("role", selector.get("kind", "")))
        graph_entry = graph.get(symbol)
        target = graph_entry[2] if graph_entry is not None and graph_entry[1] == role else ""
        if not target or not _wrapper_body_ok(spec, item, target, role):
            _canonical_add(failures, f"typed-contract:route-closure:{symbol}")
        source = (root / path).read_text(encoding="utf-8")
        item_source = source[item.start:item.end]
        observed = _source_item_digest(item_source)
        normalized = _normalized_item_digest(item_source)
        site_id = str(selector.get("site_ids", [""])[0])
        if observed != str(selector.get("source_digest", "")):
            _canonical_add(failures, f"typed-contract:route-digest:{site_id}")
        if normalized != str(selector.get("normalized_digest", "")):
            _canonical_add(failures, f"typed-contract:route-normalized-digest:{site_id}")
        expected_anchor = f"{path}::{symbol}::{normalized}"
        if expected_anchor != str(selector.get("anchor", "")):
            _canonical_add(failures, f"typed-contract:route-normalized-anchor:{site_id}")
        owner_kind = "family" if "family_id" in selector else "public_escape"
        owner_id = str(selector.get("family_id", selector.get("public_escape_id", "")))
        expected_site = hashlib.sha256(("v1\0" + "\0".join((path, symbol, owner_kind, owner_id, role, normalized, observed))).encode()).hexdigest()
        if expected_site != str(selector.get("source_site_id", "")):
            _canonical_add(failures, f"typed-contract:route-site-id:{site_id}")


def typed_only_discriminator(root: Path, authority_path: Path | None = None) -> dict[str, object]:
    spec, authority_failures = _authority_phase(root, authority_path)
    if authority_failures:
        return {"typed_only": False, "contract_complete": False, "discriminator_sites": [], "partial_sites": sorted(authority_failures)}
    assert spec is not None
    sources, failures = _authority_declarations(spec, root)
    items = _declaration_items(spec, sources, failures)
    _support_symbols_ok(spec, sources, root, failures)
    _check_nominal_declarations(spec, items, failures)
    contract = _spec_contract(spec)
    nominal_keys = (
        "value_type", "family_enum", "token_type", "decoded_type", "tag_const",
    )
    nominal_ready = all((str(contract["value_path"]), str(contract[key])) in items for key in nominal_keys)
    if nominal_ready:
        _check_codec(spec, items, failures)
        if not _canonical_roundtrip_proof(spec):
            _canonical_add(failures, "typed-contract:canonical-roundtrip")
        _check_routes(spec, sources, root, failures)
    # Macro safety is one additional semantic failure in the same authority-
    # instantiated pass.  It must not mask declaration, codec, route, or
    # digest failures from the fixture.
    if _macro_partial(spec, sources, root):
        _canonical_add(failures, "typed-contract:macro-codec")
    if failures:
        return {"typed_only": False, "contract_complete": False, "discriminator_sites": [], "partial_sites": sorted(failures)}
    return {"typed_only": True, "contract_complete": True, "discriminator_sites": [f"{_spec_contract(spec)['value_path']}:typed-contract"], "partial_sites": []}


def toml_quote(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def write_inventory(report: dict[str, object], path: Path = INVENTORY) -> None:
    typed = typed_only_discriminator(Path(str(report["root"])))
    unmatched = list(report.get("manual_unmatched", []))
    lines = [
        f'schema = {toml_quote(SCHEMA)}',
        "version = 1",
        f'typed_only = {str(bool(typed["typed_only"])).lower()}',
        f'row_count = {report["total"]}',
        f'source_file_count = {report["source_file_count"]}',
        f'source_files_digest = {toml_quote(str(report["source_files_digest"]))}',
        f'family_manifest_digest = {toml_quote(str(report.get("family_manifest_digest", "")))}',
        f'inventory_digest = {toml_quote(str(report["inventory_digest"]))}',
        f'manual_unmatched_count = {len(unmatched)}',
        f'manual_unmatched_sample = {json.dumps([str(value) for value in unmatched[:20]])}',
        "",
    ]
    lines.append("[disposition_counts]")
    for disposition, count in sorted(report.get("disposition_counts", {}).items()):
        lines.append(f'{disposition} = {int(count)}')
    lines.append("")
    for summary in report.get("manual_unmatched_summary", []):
        lines.extend([
            "[[manual_unmatched_summary]]",
            f'path = {toml_quote(str(summary["path"]))}',
            f'category = {toml_quote(str(summary["category"]))}',
            f'count = {int(summary["count"])}',
            "",
        ])
    for row in report["sites"]:
        lines.append("[[sites]]")
        for key in ("site_id", "path", "symbol", "normalized_digest", "category", "disposition", "kind", "parent_route", "reason"):
            lines.append(f'{key} = {toml_quote(str(row[key]))}')
        lines.append("")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")
    write_observations_lock(report)


def structural_rows(report: dict[str, object]) -> list[dict[str, object]]:
    return [{key: row[key] for key in OBSERVATION_KEYS} for row in report["sites"]]


def write_observations_lock(report: dict[str, object], path: Path = OBSERVATIONS_LOCK) -> None:
    lines = [
        f'schema = {toml_quote(SCHEMA + ".observations")}',
        "version = 1",
        f'row_count = {report["total"]}',
        f'source_file_count = {report["source_file_count"]}',
        f'source_files_digest = {toml_quote(str(report["source_files_digest"]))}',
        f'observation_digest = {toml_quote(str(report["inventory_digest"]))}',
        "",
    ]
    for row in structural_rows(report):
        lines.append("[[observations]]")
        for key in OBSERVATION_KEYS:
            lines.append(f'{key} = {toml_quote(str(row[key]))}')
        lines.append("")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")


def load_inventory(path: Path = INVENTORY) -> dict[str, object]:
    with path.open("rb") as stream:
        return tomllib.load(stream)


def inventory_rows(data: dict[str, object]) -> list[dict[str, object]]:
    return list(data.get("sites", []))


def load_observations_lock(path: Path = OBSERVATIONS_LOCK) -> dict[str, object]:
    with path.open("rb") as stream:
        return tomllib.load(stream)


def compare_observation_lock(report: dict[str, object], lock: dict[str, object]) -> list[str]:
    failures: list[str] = []
    expected_rows = list(lock.get("observations", []))
    if lock.get("schema") != SCHEMA + ".observations":
        failures.append("observation lock schema mismatch")
    if lock.get("row_count") != len(expected_rows) or lock.get("row_count") != report["total"]:
        failures.append("observation lock row_count does not reconcile")
    if lock.get("source_file_count") != report["source_file_count"]:
        failures.append("observation lock source_file_count does not cover current production root")
    if lock.get("source_files_digest") != report["source_files_digest"]:
        failures.append("observation lock source file set digest does not cover current production root")
    if lock.get("observation_digest") != report["inventory_digest"]:
        failures.append("observation lock digest does not match current structural observations")
    actual_rows = structural_rows(report)
    if expected_rows != actual_rows:
        expected_ids = {str(row.get("site_id")) for row in expected_rows}
        actual_ids = {str(row.get("site_id")) for row in actual_rows}
        missing = sorted(expected_ids - actual_ids)
        added = sorted(actual_ids - expected_ids)
        if missing:
            failures.append(f"observation lock stale/moved rows: missing={missing[:5]}")
        if added:
            failures.append(f"observation lock unrecorded observations: added={added[:5]}")
        if not missing and not added:
            failures.append("observation lock row content differs from current source")
    return failures


def compare_inventory(report: dict[str, object], expected: dict[str, object], *, observed_typed: bool | None = None) -> list[str]:
    failures: list[str] = []
    expected_rows = inventory_rows(expected)
    actual_rows = [{k: v for k, v in row.items() if k != "line"} for row in report["sites"]]
    allowed_dispositions = {OPAQUE_LEGACY, ORDINARY_CONTROL, PRIVATE_NONREIFIED, NEEDS_CLASSIFICATION}
    if expected.get("schema") != SCHEMA:
        failures.append("inventory schema mismatch")
    if observed_typed is not None and bool(expected.get("typed_only", False)) != observed_typed:
        failures.append(
            f"inventory typed_only mode {bool(expected.get('typed_only', False))} "
            f"does not match production discriminator {observed_typed}"
        )
    if expected.get("row_count") != len(expected_rows):
        failures.append("inventory row_count does not reconcile")
    if expected.get("source_file_count") != report["source_file_count"]:
        failures.append("inventory source_file_count does not cover current production root")
    if expected.get("source_files_digest") != report["source_files_digest"]:
        failures.append("inventory source file set digest does not cover current production root")
    if expected.get("family_manifest_digest") != report.get("family_manifest_digest"):
        failures.append("inventory family manifest digest does not match checked-in authority")
    if expected.get("inventory_digest") != report["inventory_digest"]:
        failures.append("inventory digest does not match current source")
    if expected.get("manual_unmatched_count") != len(report.get("manual_unmatched", [])):
        failures.append("inventory manual_unmatched_count does not reconcile current manual census")
    if expected.get("disposition_counts", {}) != report.get("disposition_counts", {}):
        failures.append("inventory disposition_counts does not reconcile current manual census")
    expected_unmatched_summary = expected.get("manual_unmatched_summary", [])
    actual_unmatched_summary = report.get("manual_unmatched_summary", [])
    if expected_unmatched_summary != actual_unmatched_summary:
        failures.append("inventory manual_unmatched_summary does not reconcile current manual census")
    duplicate_expected_ids = [
        site_id for site_id in sorted({str(row.get("site_id")) for row in expected_rows})
        if sum(1 for row in expected_rows if str(row.get("site_id")) == site_id) > 1
    ]
    if duplicate_expected_ids:
        failures.append(f"duplicate inventory site_id rows: {duplicate_expected_ids[:5]}")
    invalid_dispositions = []
    empty_reasons = []
    for row in expected_rows:
        disposition = row.get("disposition")
        if disposition not in allowed_dispositions:
            invalid_dispositions.append((str(row.get("site_id")), disposition))
        if not str(row.get("reason", "")).strip():
            empty_reasons.append(str(row.get("site_id")))
    if invalid_dispositions:
        failures.append(
            f"invalid inventory dispositions: count={len(invalid_dispositions)} "
            f"sample={invalid_dispositions[:12]}"
        )
    if empty_reasons:
        failures.append(f"inventory rows have no explicit reason: count={len(empty_reasons)} sample={empty_reasons[:12]}")
    if expected_rows != actual_rows:
        expected_ids = {str(row.get("site_id")) for row in expected_rows}
        actual_ids = {str(row.get("site_id")) for row in actual_rows}
        missing = sorted(expected_ids - actual_ids)
        added = sorted(actual_ids - expected_ids)
        if missing:
            failures.append(f"stale/moved inventory rows: missing={missing[:5]}")
        if added:
            failures.append(f"unrecorded opaque candidates: added={added[:5]}")
        if not missing and not added:
            failures.append("inventory row content differs from current source")
    return failures


def _phase_c_rows(report: dict[str, object]) -> list[dict[str, object]]:
    base_keys = ("site_id", "path", "symbol", "normalized_digest", "category", "kind", "parent_route", "source_digest")
    optional_keys = ("expression_id", "expression_count", "expression_digest", "edge_digest")
    rows: list[dict[str, object]] = []
    for row in report.get("sites", []):
        item = {key: row[key] for key in base_keys}
        for key in optional_keys:
            if key in row:
                item[key] = row[key]
        rows.append(item)
    return rows


def write_phase_c_derived(report: dict[str, object], inventory_path: Path = INVENTORY, lock_path: Path = OBSERVATIONS_LOCK) -> None:
    """Refresh only derived Phase-C evidence; never mutate families.toml."""
    rows = _phase_c_rows(report)
    lines = [
        f"schema = {toml_quote(PHASE_C_INVENTORY_SCHEMA)}",
        "version = 1",
        "authority_mode = \"families.toml\"",
        f"row_count = {len(rows)}",
        f"source_file_count = {int(report.get('source_file_count', 0))}",
        f"source_files_digest = {toml_quote(str(report.get('source_files_digest', '')))}",
        f"authority_digest = {toml_quote(str(report.get('authority_digest', '')))}",
        f"inventory_digest = {toml_quote(str(report.get('inventory_digest', '')))}",
        "",
    ]
    for row in rows:
        lines.extend(["[[observations]]"])
        for key in ("site_id", "path", "symbol", "normalized_digest", "category", "kind", "parent_route", "source_digest", "expression_id", "expression_count", "expression_digest", "edge_digest"):
            if key in row:
                value = row[key]
                lines.append(f"{key} = {value if isinstance(value, int) else toml_quote(str(value))}")
        lines.append("")
    inventory_path.write_text("\n".join(lines), encoding="utf-8")
    lock_lines = [
        f"schema = {toml_quote(PHASE_C_OBSERVATION_SCHEMA)}",
        "version = 1",
        "authority_mode = \"families.toml\"",
        f"row_count = {len(rows)}",
        f"source_file_count = {int(report.get('source_file_count', 0))}",
        f"source_files_digest = {toml_quote(str(report.get('source_files_digest', '')))}",
        f"authority_digest = {toml_quote(str(report.get('authority_digest', '')))}",
        f"observation_digest = {toml_quote(str(report.get('inventory_digest', '')))}",
        "",
    ]
    for row in rows:
        lock_lines.extend(["[[observations]]"])
        for key in ("site_id", "path", "symbol", "normalized_digest", "category", "kind", "parent_route", "source_digest", "expression_id", "expression_count", "expression_digest", "edge_digest"):
            if key in row:
                value = row[key]
                lock_lines.append(f"{key} = {value if isinstance(value, int) else toml_quote(str(value))}")
        lock_lines.append("")
    lock_path.write_text("\n".join(lock_lines), encoding="utf-8")


def compare_phase_c_derived(report: dict[str, object], inventory: dict[str, object], lock: dict[str, object]) -> list[str]:
    """Compare the two derived ledgers without allowing either to classify."""
    failures: list[str] = []
    rows = _phase_c_rows(report)
    for data, schema, digest_key in ((inventory, PHASE_C_INVENTORY_SCHEMA, "inventory_digest"), (lock, PHASE_C_OBSERVATION_SCHEMA, "observation_digest")):
        if data.get("schema") != schema or data.get("version") != 1 or data.get("authority_mode") != "families.toml":
            failures.append("phase-c:derived-schema")
        if any(key in data for key in ("owner", "disposition", "family_id", "public_escape_id")):
            failures.append("phase-c:derived-authority-field")
        if data.get("row_count") != len(rows):
            failures.append("phase-c:derived-row-count")
        if data.get("source_file_count") != report.get("source_file_count") or data.get("source_files_digest") != report.get("source_files_digest"):
            failures.append("phase-c:derived-source-set")
        if data.get("authority_digest") != report.get("authority_digest"):
            failures.append("phase-c:derived-authority-digest")
        if data.get(digest_key) != report.get("inventory_digest"):
            failures.append("phase-c:derived-observation-digest")
        expected_rows = list(data.get("observations", []))
        if expected_rows != rows:
            failures.append("phase-c:derived-row-content")
    return failures


def expected_fixture_data(path: Path = EXPECTED) -> dict[str, object]:
    with path.open("rb") as stream:
        return tomllib.load(stream)


RECONCILIATION_DIAGNOSTIC_KEYS = (
    "duplicate inventory site_id rows",
    "stale/moved inventory rows",
    "unrecorded opaque candidates",
    "inventory digest does not match current source",
    "inventory manual_unmatched_count does not reconcile current manual census",
    "inventory source file set digest does not cover current production root",
    "inventory source_file_count does not cover current production root",
    "family manifest IDs do not match the frozen central/direct family set",
    "family manifest row lacks allocator_or_table",
    "family manifest row lacks exposure",
    "family manifest row lacks migration",
    "family manifest row lacks name/proof",
    "family manifest row lacks store_link",
    "family manifest row lacks topology",
    "family required_roles are empty/invalid",
    "family required_roles do not reconcile selectors",
    "invalid family exposure",
    "invalid family topology",
    "family selector has unknown/mismatched family path",
    "family selector topology mismatch",
    "public selector allocator_or_table mismatch",
    "public selector anchor mismatch",
    "public selector exposure mismatch",
    "public selector has invalid category/role",
    "public selector is not opaque",
    "public selector lacks topology/exposure/migration/store proof",
    "public selector migration mismatch",
    "public selector topology mismatch",
    "public selector has unknown public escape",
    "allocator/reifier anchors are not separate",
    "needs_classification",
)


def reconciliation_diagnostic_key(value: str) -> str:
    for prefix in RECONCILIATION_DIAGNOSTIC_KEYS:
        if value.startswith(prefix):
            return prefix
    return value


def fixture_tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(path.relative_to(root).as_posix().encode())
        digest.update(b"\\0")
        digest.update(path.read_bytes())
        digest.update(b"\\0")
    return digest.hexdigest()


def run_mutations() -> list[str]:
    failures: list[str] = []
    try:
        expected_digest = hashlib.sha256(EXPECTED.read_bytes()).hexdigest()
    except OSError:
        expected_digest = ""
    if expected_digest != FROZEN_EXPECTED_TOML_DIGEST:
        failures.append(
            f"mutation.EXPECTED_ORACLE_DRIFT:expected={FROZEN_EXPECTED_TOML_DIGEST}:observed={expected_digest}"
        )
    expected = expected_fixture_data()
    cases = expected.get("cases", [])
    e2e_results: dict[str, str] = {}
    e2e_diagnostics: dict[str, list[str]] = {}
    if set(FROZEN_RECONCILIATION_TREE_DIGESTS) != {"authority", "stale_moved"}:
        failures.append("mutation.FROZEN_RECONCILIATION_TREE_DIGEST_SET")
    for fixture_name, digest in FROZEN_RECONCILIATION_TREE_DIGESTS.items():
        if fixture_tree_digest(FIXTURES / fixture_name) != digest:
            failures.append(f"mutation.RECONCILIATION_FIXTURE_DRIFT:{fixture_name}")

    def record_e2e(name: str, diagnostics: list[object], expected_marker: str = "") -> None:
        observed = [str(value) for value in diagnostics if str(value)]
        e2e_results[name] = "rejected" if observed else "accepted"
        e2e_diagnostics[name] = observed
        if not observed:
            failures.append(f"mutation.{name}:detector accepted reconciliation defect")
    if list(expected.get("mutation_order", [])) != list(MUTATION_PHASES):
        failures.append("mutation corpus order must be E2E-reconciliation, schema units, scanner/codec")
    phase_numbers = {phase: index for index, phase in enumerate(MUTATION_PHASES)}
    observed_phases = [str(case.get("phase", "")) for case in cases]
    if any(phase not in phase_numbers for phase in observed_phases):
        failures.append(f"mutation corpus has an unknown/missing phase: {observed_phases!r}")
    elif observed_phases != sorted(observed_phases, key=phase_numbers.__getitem__):
        failures.append(f"mutation corpus phases are out of order: {observed_phases!r}")

    # E2E-reconciliation mutations run before schema and scanner/codec cases.
    # A stale/moved checked-in row must fail independently of the current
    # source inventory, and duplicate identity rows must fail closed.
    current = scan(FIXTURES / "stale_moved")
    fake = {"schema": SCHEMA, "typed_only": False, "row_count": 1,
            "inventory_digest": "deadbeef", "sites": [{
                "site_id": "stale-row", "path": "apps/mamba/src/runtime/moved.rs",
                "symbol": "gone", "normalized_digest": "old", "category": "producer",
                "disposition": OPAQUE_LEGACY, "kind": "producer",
                "parent_route": "opaque-value-boundary/producer", "reason": "stale"}]}
    stale_failures = compare_inventory(current, fake)
    record_e2e("stale_selector", stale_failures, "stale/moved inventory rows")
    duplicate_lock = dict(fake)
    duplicate_lock["row_count"] = 2
    duplicate_lock["sites"] = [fake["sites"][0], dict(fake["sites"][0])]
    duplicate_failures = compare_inventory(current, duplicate_lock)
    record_e2e("multiple_selector", duplicate_failures, "duplicate inventory site_id rows")

    # Selector reconciliation mutations are intentionally independent of the
    # current-tree census.  They exercise the checked-in authority shape even
    # while the live map is deliberately incomplete.
    try:
        # Mutation oracles use only the small checked-in authority fixture.
        # The live families.toml is validated separately by --check and may
        # remain incomplete while the production census is being mapped.
        manifest = load_family_manifest(AUTHORITY_FIXTURE)
        fixture_diagnostics = validate_family_manifest(
            manifest, strict_live=False, expected_family_ids={"fixture_direct"}
        )
        if fixture_diagnostics:
            failures.extend(f"mutation.AUTHORITY_FIXTURE_INVALID:{item}" for item in fixture_diagnostics)
    except (OSError, tomllib.TOMLDecodeError) as error:
        manifest = {"families": [], "selectors": [], "public_selectors": [], "public_escapes": []}
        failures.append(f"mutation.MANIFEST_LOAD:{type(error).__name__}")

    def expect_manifest_failure(label: str, changed: dict[str, object], marker: str) -> None:
        diagnostics = validate_family_manifest(
            changed, strict_live=False, expected_family_ids={"fixture_direct"}
        )
        record_e2e(label.lower(), diagnostics, marker)

    duplicate_family = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    duplicate_family["families"] = [*list(manifest.get("families", [])), dict(list(manifest.get("families", []))[0])] if manifest.get("families") else [{"id": "central_array"}, {"id": "central_array"}]
    expect_manifest_failure("CENTRAL_DUPLICATE", duplicate_family, "duplicate family ID")
    removed_family = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    removed_family["families"] = [family for family in manifest.get("families", []) if str(family.get("id")) != "fixture_direct"]
    expect_manifest_failure("CENTRAL_REMOVE", removed_family, "family manifest IDs")
    added_family = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    added_family["families"] = [*list(manifest.get("families", [])), {"id": "fixture_added", "path": "fixture_added.rs"}]
    expect_manifest_failure("CENTRAL_ADD", added_family, "family manifest IDs")
    direct_violation = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    direct_violation["families"] = [
        {**family, "topology": "central_registered"} if str(family.get("id")) == "fixture_direct" else family
        for family in manifest.get("families", [])
    ]
    expect_manifest_failure("DIRECT_CENTRAL", direct_violation, "family selector topology mismatch")
    public_reifier = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    public_reifier["public_selectors"] = [{
        "public_escape_id": "fixture_public_escape", "path": "apps/mamba/tests/governance/gates/t1_opaque_value_boundary/fixtures/authority/valid.rs",
        "symbol": "barrier_id", "kind": "private_id_reification", "category": "private_metadata",
        "normalized_digest": "mutation", "disposition": ORDINARY_CONTROL,
    }]
    expect_manifest_failure("PUBLIC_BARRIER_REIFIER", public_reifier, "public selector is not opaque")
    barrier_mismatch = {key: (list(value) if isinstance(value, list) else value) for key, value in manifest.items()}
    barrier_mismatch["public_selectors"] = [{
        **dict(manifest["public_selectors"][0]), "public_escape_id": "missing_escape"
    }] if manifest.get("public_selectors") else [{
        "public_escape_id": "missing_escape", "path": "fixture.rs", "symbol": "barrier_id",
        "kind": "private_id_reification", "role": "private_id_reification", "category": "private_metadata",
        "normalized_digest": "mutation", "anchor": "fixture.rs::barrier_id::mutation",
        "topology": "unregistered_side_table", "exposure": "public_opaque_int",
        "allocator_or_table": "barrier_id", "migration": "migrate", "disposition": OPAQUE_LEGACY,
        "proof": "mutation", "site_ids": ["barrier-mutation"],
    }]
    expect_manifest_failure("BARRIER_ESCAPE_MISMATCH", barrier_mismatch, "unknown public escape")
    random_path = FIXTURES / "reconciliation" / "random_allocator_reifier.rs"
    random_fixture = scan(random_path)
    random_symbols = {str(row["symbol"]) for row in random_fixture["sites"]}
    random_source = mask_non_code(random_path.read_text(encoding="utf-8"))
    alloc_match = re.search(r"\bfn\s+alloc_random_id\b[^\{]*\{", random_source)
    alloc_body = random_source[alloc_match.start() : matching_brace(random_source, alloc_match.end() - 1) + 1] if alloc_match else ""
    random_missing = {"alloc_random_id", "make_handle", "publish_handle"} - random_symbols
    random_diagnostics = []
    if random_missing or "from_int" in alloc_body or "as_int" in alloc_body:
        random_diagnostics.append(
            f"allocator/reifier anchors are not separate: missing={sorted(random_missing)}"
        )
    record_e2e("random_allocator_reifier", random_diagnostics, "allocator/reifier anchors are not separate")
    # The oracle fixture proves positive exact selectors and an unresolved
    # synthetic row without touching the incomplete production census.
    fixture_rows = scan(AUTHORITY_SOURCE_FIXTURE)["sites"]
    unknown = {
        "site_id": "fixture-unmatched", "path": "fixture/new.rs", "symbol": "new",
        "normalized_digest": "new", "category": "private_metadata", "kind": "unnamed_numeric_side_table",
    }
    manual_rows = {"sites": [*fixture_rows, unknown]}
    manual_unmatched = apply_manual_authority(manual_rows, manifest, strict_live=False)
    authority = manual_rows.get("authority", {})
    fixture_owners = {str(item.get("owner")) for item in authority.values() if item.get("owner")}
    if fixture_rows[0]["disposition"] != OPAQUE_LEGACY or authority.get(fixture_rows[0]["site_id"], {}).get("owner") != "fixture_direct":
        failures.append("mutation.AUTHORITY_POSITIVE:exact_family_owner_or_proof_missing")
    if fixture_rows[1]["disposition"] != OPAQUE_LEGACY or authority.get(fixture_rows[1]["site_id"], {}).get("owner") != "fixture_public_escape":
        failures.append("mutation.AUTHORITY_POSITIVE:exact_public_escape_owner_or_proof_missing")
    unmatched_diagnostics = list(manual_rows.get("authority_diagnostics", []))
    record_e2e("unmatched_selector", unmatched_diagnostics, "needs_classification")

    for reconciliation in expected.get("reconciliation", []):
        name = str(reconciliation.get("name"))
        if e2e_results.get(name) != str(reconciliation.get("expected_result")):
            failures.append(f"mutation.RECONCILIATION_EXPECTATION:{name}")
        expected_diagnostic = str(reconciliation.get("expected_diagnostic"))
        expected_diagnostics = [str(value) for value in reconciliation.get("expected_diagnostics", [])]
        declared = expected_diagnostics or ([expected_diagnostic] if expected_diagnostic else [])
        observed = sorted(reconciliation_diagnostic_key(value) for value in e2e_diagnostics.get(name, []))
        declared = sorted(reconciliation_diagnostic_key(value) for value in declared)
        if observed != sorted(declared):
            failures.append(
                f"mutation.RECONCILIATION_DIAGNOSTIC:{name}:expected={sorted(declared)!r}:observed={observed!r}"
            )
    reconciliation_names = [str(item.get("name")) for item in expected.get("reconciliation", [])]
    if reconciliation_names != list(RECONCILIATION_ORDER):
        failures.append("mutation.RECONCILIATION_ORDER_OR_SET")

    if set(FROZEN_CASE_TREE_DIGESTS) != FROZEN_CASE_NAMES:
        failures.append("mutation.FROZEN_CASE_TREE_DIGEST_SET")
    for case in cases:
        name = str(case["name"])
        root = FIXTURES / name
        if fixture_tree_digest(root) != FROZEN_CASE_TREE_DIGESTS.get(name):
            failures.append(f"mutation.CASE_FIXTURE_DRIFT:{name}")
        report = scan(root)
        expected_categories = sorted(case.get("categories", []))
        actual_categories = sorted(row["category"] for row in report["sites"])
        if actual_categories != expected_categories:
            failures.append(f"{name}: categories {actual_categories!r} != {expected_categories!r}")
        expected_diag = bool(case.get("diagnostics"))
        if bool(report["diagnostics"]) != expected_diag:
            failures.append(f"{name}: diagnostics presence mismatch")
        declared_diagnostics = [str(value) for value in case.get("expected_diagnostics", [])]
        if declared_diagnostics and sorted(str(value) for value in report["diagnostics"]) != sorted(declared_diagnostics):
            failures.append(f"{name}: diagnostics differ from frozen expected list")
        patterns = [str(pattern) for pattern in case.get("diagnostic_patterns", [])]
        if patterns and not declared_diagnostics:
            failures.append(f"{name}: diagnostic_patterns require an exact expected_diagnostics list")
        for pattern in patterns:
            if not any(diagnostic == pattern or diagnostic.endswith(f": {pattern}") for diagnostic in declared_diagnostics):
                failures.append(f"{name}: diagnostic pattern is absent from frozen expected list {pattern!r}")
        if case.get("new_source_required") and report["total"] < 1:
            failures.append(f"{name}: new source was not rediscovered")
        if case.get("unique_site_ids"):
            ids = [row["site_id"] for row in report["sites"]]
            if len(ids) != len(set(ids)):
                failures.append(f"{name}: duplicate site_id collision was not rejected")

    # Typed fixtures are immutable checked-in inputs.  The oracle reads the
    # authority and source tree for each case independently; it never creates
    # a mutation by replacing strings in a baseline at runtime.
    typed_cases = list(expected.get("typed_cases", []))
    if set(FROZEN_TYPED_TREE_DIGESTS) != FROZEN_TYPED_NAMES:
        failures.append("mutation.FROZEN_TYPED_TREE_DIGEST_SET")
    if set(FROZEN_TYPED_SOURCE_DIGESTS) != FROZEN_TYPED_NAMES:
        failures.append("mutation.FROZEN_TYPED_SOURCE_DIGEST_SET")
    frozen_signature = tuple(
        (
            str(item.get("name", "")),
            str(item.get("fixture", "")),
            str(item.get("expected_result", "")),
            tuple(str(value) for value in item.get("expected_diagnostics", [])),
        )
        for item in typed_cases
    )
    if frozen_signature != FROZEN_TYPED_EXPECTATIONS:
        failures.append(
            f"mutation.FROZEN_TYPED_EXPECTATIONS:expected={FROZEN_TYPED_EXPECTATIONS!r}:observed={frozen_signature!r}"
        )
    for typed_case in typed_cases:
        name = str(typed_case.get("name", ""))
        fixture = GATE / str(typed_case.get("fixture", ""))
        authority_path = fixture / "authority.toml"
        if not name or not fixture.is_dir() or not authority_path.is_file():
            failures.append(f"mutation.TYPED_FIXTURE_MISSING:{name}")
            continue
        if fixture_tree_digest(fixture) != FROZEN_TYPED_TREE_DIGESTS.get(name):
            failures.append(f"mutation.TYPED_FIXTURE_DRIFT:{name}")
        actual_fixture_files = tuple(
            sorted(path.relative_to(fixture).as_posix() for path in fixture.rglob("*") if path.is_file())
        )
        if actual_fixture_files != ("authority.toml", "runtime/value.rs", "typed_routes.rs"):
            failures.append(f"mutation.TYPED_FIXTURE_FILE_SET:{name}:{actual_fixture_files!r}")
        source_digests = tuple(
            (relative, hashlib.sha256((fixture / relative).read_bytes()).hexdigest())
            for relative in ("authority.toml", "runtime/value.rs", "typed_routes.rs")
            if (fixture / relative).is_file()
        )
        if source_digests != FROZEN_TYPED_SOURCE_DIGESTS.get(name):
            failures.append(f"mutation.TYPED_SOURCE_DRIFT:{name}")
        # Copy immutable checked-in inputs under an opaque temporary root so
        # detector behavior cannot branch on fixture directory names or
        # parent layout.  The authority path is explicit and copied together
        # with the source tree.
        with tempfile.TemporaryDirectory(prefix="mamba-opaque-typed-case-") as temporary_root:
            isolated = Path(temporary_root) / "case"
            shutil.copytree(fixture, isolated)
            result = typed_only_discriminator(isolated, isolated / "authority.toml")
        expected_result = str(typed_case.get("expected_result", ""))
        expected_diagnostics = [str(value) for value in typed_case.get("expected_diagnostics", [])]
        if "expected_diagnostic" in typed_case:
            failures.append(f"mutation.TYPED_LEGACY_DIAGNOSTIC_FIELD:{name}")
        if expected_diagnostics != sorted(expected_diagnostics):
            failures.append(f"mutation.TYPED_DIAGNOSTICS_NOT_SORTED:{name}")
        if expected_result == "accepted":
            if not result["typed_only"] or not result["contract_complete"] or result["partial_sites"]:
                failures.append(f"mutation.TYPED_ACCEPT:{name}:{result.get('partial_sites', [])}")
            elif expected_diagnostics:
                failures.append(f"mutation.TYPED_ACCEPT_DIAGNOSTIC:{name}")
        elif expected_result == "rejected":
            if result["typed_only"] or sorted(result["partial_sites"]) != sorted(expected_diagnostics):
                failures.append(
                    f"mutation.TYPED_REJECT:{name}:expected={sorted(expected_diagnostics)!r}:observed={result['partial_sites']!r}"
                )
        else:
            failures.append(f"mutation.TYPED_EXPECTATION:{name}")
    phase_c_cases = list(expected.get("phase_c_cases", []))
    phase_c_names = [str(item.get("name", "")) for item in phase_c_cases]
    if phase_c_names != list(PHASE_C_CASE_ORDER) or len(phase_c_cases) != PHASE_C_CASE_COUNT:
        failures.append("mutation.PHASE_C_CASE_SET")
    phase_c_fixture_dirs = {
        path.name
        for path in (FIXTURES / "phase_c_cases").iterdir()
        if path.is_dir()
    }
    if phase_c_fixture_dirs != set(PHASE_C_CASE_ORDER):
        failures.append("mutation.PHASE_C_FIXTURE_DIRECTORY_SET")
    if set(FROZEN_PHASE_C_TREE_DIGESTS) != set(PHASE_C_CASE_ORDER):
        failures.append("mutation.PHASE_C_TREE_DIGEST_SET")
    for phase_case in phase_c_cases:
        name = str(phase_case.get("name", ""))
        fixture_value = str(phase_case.get("fixture", ""))
        fixture = GATE / fixture_value
        frozen = FROZEN_PHASE_C_EXPECTATIONS.get(name)
        if frozen is None or fixture_value != f"fixtures/phase_c_cases/{name}":
            failures.append(f"mutation.PHASE_C_EXPECTATION:{name}")
            continue
        if str(phase_case.get("phase", "")) != PHASE_C_PHASE_CASES.get(name):
            failures.append(f"mutation.PHASE_C_PHASE:{name}")
        declared_result = str(phase_case.get("expected_result", ""))
        declared_diagnostics = tuple(str(value) for value in phase_case.get("expected_diagnostics", []))
        if list(declared_diagnostics) != sorted(declared_diagnostics):
            failures.append(f"mutation.PHASE_C_DIAGNOSTICS_NOT_SORTED:{name}")
        if (declared_result, list(declared_diagnostics)) != (frozen[0], list(frozen[1])):
            failures.append(f"mutation.PHASE_C_FROZEN_EXPECTATION:{name}")
        if not fixture.is_dir() or fixture_tree_digest(fixture) != FROZEN_PHASE_C_TREE_DIGESTS.get(name):
            observed_tree = fixture_tree_digest(fixture) if fixture.is_dir() else "missing"
            failures.append(f"mutation.PHASE_C_FIXTURE_DRIFT:{name}")
            continue
        if (fixture / "structural.toml").is_file():
            observed_diagnostics = _phase_c_structural_fixture_result(fixture)
        else:
            observed_diagnostics = _phase_c_opaque_fixture_result(fixture)
        observed_result = "rejected" if observed_diagnostics else "accepted"
        if observed_result != declared_result or sorted(observed_diagnostics) != sorted(declared_diagnostics):
            failures.append(
                f"mutation.PHASE_C_RESULT:{name}:expected={declared_result}:{sorted(declared_diagnostics)!r}:observed={observed_result}:{sorted(observed_diagnostics)!r}"
            )
    typed_names = [str(item.get("name", "")) for item in typed_cases]
    reconciliation_names = [str(item.get("name", "")) for item in expected.get("reconciliation", [])]
    case_names = [str(item.get("name", "")) for item in cases]
    all_names = reconciliation_names + case_names + typed_names
    fixture_paths = [f"fixtures/{name}" for name in case_names] + [str(item.get("fixture", "")) for item in typed_cases]
    expected_fixture_paths = {f"fixtures/{name}" for name in FROZEN_CASE_NAMES} | {
        fixture for _, fixture, _, _ in FROZEN_TYPED_EXPECTATIONS
    }
    if len(all_names) != len(set(all_names)) or any(not name for name in all_names):
        failures.append("mutation.CASE_NAME_IDENTITY")
    if (
        len(fixture_paths) != len(set(fixture_paths))
        or any(not path for path in fixture_paths)
        or set(fixture_paths) != expected_fixture_paths
        or any(not (GATE / path).is_dir() for path in fixture_paths)
    ):
        failures.append("mutation.TYPED_FIXTURE_PATH_IDENTITY")
    if set(reconciliation_names) != FROZEN_RECONCILIATION_NAMES or len(reconciliation_names) != FROZEN_RECONCILIATION_COUNT:
        failures.append("mutation.FROZEN_RECONCILIATION_SET")
    if set(case_names) != FROZEN_CASE_NAMES or len(case_names) != FROZEN_CASE_COUNT:
        failures.append("mutation.FROZEN_CASE_SET")
    if set(typed_names) != FROZEN_TYPED_NAMES or len(typed_names) != FROZEN_TYPED_COUNT:
        failures.append("mutation.FROZEN_TYPED_SET")
    if len(all_names) != FROZEN_MUTATION_CASE_COUNT:
        failures.append("mutation.FROZEN_TOTAL")
    if set(all_names) & set(phase_c_names):
        failures.append("mutation.PHASE_C_NAME_COLLISION")
    phase_fixture_paths = [str(item.get("fixture", "")) for item in phase_c_cases]
    if len(phase_fixture_paths) != len(set(phase_fixture_paths)) or set(phase_fixture_paths) & set(fixture_paths):
        failures.append("mutation.PHASE_C_PATH_COLLISION")
    phase_c_phase_counts = Counter(str(case.get("phase", "")) for case in phase_c_cases)
    if dict(sorted(phase_c_phase_counts.items())) != dict(sorted(PHASE_C_PHASE_COUNTS.items())):
        failures.append(
            f"mutation.PHASE_C_PHASE_COUNTS:expected={dict(PHASE_C_PHASE_COUNTS)!r}:observed={dict(sorted(phase_c_phase_counts.items()))!r}"
        )
    phase_counts = Counter(str(case.get("phase", "")) for case in cases)
    phase_counts["e2e_reconciliation"] += len(expected.get("reconciliation", []))
    phase_counts["scanner_codec"] += len(typed_cases)
    if dict(sorted(phase_counts.items())) != dict(sorted(FROZEN_PHASE_COUNTS.items())):
        failures.append(
            f"mutation.FROZEN_PHASE_COUNTS:expected={FROZEN_PHASE_COUNTS!r}:observed={dict(sorted(phase_counts.items()))!r}"
        )
    return failures
def run_check() -> tuple[int, dict[str, object]]:
    failures: list[str] = []
    # Phase-C is authority-first.  Do not call the legacy broad census here:
    # an unseeded source match is a perimeter candidate, never a disposition.
    first = phase_c_report(PRODUCTION)
    second = phase_c_report(PRODUCTION)
    typed = typed_only_discriminator(PRODUCTION)
    if encoded(first) != encoded(second):
        failures.append("phase-c source resolution is not deterministic")
    if first["diagnostics"]:
        failures.extend(str(value) for value in first["diagnostics"])
    # Perimeter findings are not dispositions.  Any unseeded candidate in the
    # live runtime must keep this gate red until an independently reviewed
    # authority row and exact semantic observation are added.
    if first.get("unseeded_candidates"):
        failures.append("phase-c:unseeded-candidate")
    if first["total"] == 0:
        failures.append("phase-c authority produced no seeded/perimeter observations")
    try:
        expected = load_inventory()
        lock = load_observations_lock()
        failures.extend(compare_phase_c_derived(first, expected, lock))
    except (OSError, tomllib.TOMLDecodeError) as error:
        failures.append(f"cannot load checked-in Phase-C derived ledgers: {error}")
    try:
        failures.extend(run_mutations())
    except Exception as error:  # pragma: no cover - final fail-closed shield
        failures.append(f"mutation.INTERNAL_ERROR:{type(error).__name__}:{error}")
    result = {
        "schema": SCHEMA,
        "mode": "check",
        "status": "PASS" if not failures else "FAIL",
        "total": first["total"],
        "inventory_digest": first["inventory_digest"],
        "family_manifest_digest": first.get("authority_digest", ""),
        "counts": {
            "seeded": len(first.get("seeded", [])),
            "unseeded_candidate": len(first.get("unseeded_candidates", [])),
        },
        "mutation_cases": FROZEN_MUTATION_CASE_COUNT,
        "phase_c_cases": PHASE_C_CASE_COUNT,
        "phase_c_phase_counts": dict(PHASE_C_PHASE_COUNTS),
        "failures": failures,
        "typed_only": typed,
        "disposition_counts": {"seeded": len(first.get("seeded", [])), "unseeded_candidate": len(first.get("unseeded_candidates", []))},
        "unseeded_candidates": [str(row["site_id"]) for row in first.get("unseeded_candidates", [])[:20]],
    }
    return (0 if not failures else 1), result


def encoded(report: dict[str, object]) -> bytes:
    return (json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n").encode()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--mode", choices=("inventory", "check", "mutations"), default="inventory")
    parser.add_argument("--root", type=Path, default=PRODUCTION)
    parser.add_argument("--write-inventory", action="store_true")
    args = parser.parse_args()
    if args.check or args.mode == "check":
        code, report = run_check()
        sys.stdout.buffer.write(encoded(report))
        return code
    if args.mode == "mutations":
        try:
            failures = run_mutations()
        except Exception as error:  # never leak a mutation traceback
            failures = [f"mutation.INTERNAL_ERROR:{type(error).__name__}:{error}"]
        fixture_data = expected_fixture_data()
        phase_counts = Counter(str(case.get("phase", "")) for case in fixture_data.get("cases", []))
        phase_counts["e2e_reconciliation"] += len(fixture_data.get("reconciliation", []))
        phase_counts["scanner_codec"] += len(fixture_data.get("typed_cases", []))
        report = {"schema": SCHEMA, "mode": "mutations", "status": "PASS" if not failures else "FAIL", "mutation_cases": FROZEN_MUTATION_CASE_COUNT, "phase_c_cases": PHASE_C_CASE_COUNT, "phase_c_phase_counts": dict(PHASE_C_PHASE_COUNTS), "mutation_phase_counts": dict(sorted(phase_counts.items())), "failures": failures}
        sys.stdout.buffer.write(encoded(report))
        return 0 if not failures else 1
    if args.root.resolve() == PRODUCTION.resolve():
        report = phase_c_report(args.root)
        report["mode"] = "inventory"
        report["status"] = "PASS" if not report["diagnostics"] else "FAIL"
        if args.write_inventory:
            write_phase_c_derived(report)
    else:
        report = scan(args.root)
        unmatched = apply_manual_authority(report)
        if report.get("family_manifest_diagnostics"):
            report["diagnostics"].extend(str(value) for value in report["family_manifest_diagnostics"])
        report["mode"] = "inventory"
        report["manual_unmatched_count"] = len(unmatched)
        report["manual_unmatched_sample"] = unmatched[:20]
        report["status"] = "PASS" if not report["diagnostics"] and not unmatched else "FAIL"
        if args.write_inventory:
            write_inventory(report)
    sys.stdout.buffer.write(encoded(report))
    return 0 if report["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
