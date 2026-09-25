# Reference-System Acceptance Matrix

This matrix links each of the seven ordered reference-system milestones (spec
issue #1, ADR 0015) to its concrete externally observable behavior, the named
acceptance tests that assert it, the stable log artifact a reviewer reads, and
the known limitations and non-claims that bound the result. Test names below are
verbatim from `tests/ledger_issue_*.rs`; log names follow the campaign naming
scheme (`logs/<suite>_<feature>.log`) produced by
`scripts/run_reference_evidence.sh` (see [README.md](README.md)).

A row is *admitted* only when its tests pass in a recorded campaign run; the
mechanically derived verdict lives in each run's `manifest.json` and
`summary.md`.

## Milestone 1 — Durable admission authority (Issue #2, `ledger_issue_2`)

**External behavior:** one complete canonical `AdmittedBlockEnvelope` per
commit; journal append plus fsync is the sole commit point; projections rebuild
through verified replay only.

| Case class | Evidence (test → artifact) |
|---|---|
| Success | `ordinary_candidate_commits_one_canonical_envelope_and_replays_projections` → `logs/ledger_issue_2_default.log` |
| Rejection | `deterministic_rejections_leave_the_authoritative_journal_unchanged` |
| Tamper | `noncanonical_and_tampered_envelopes_are_rejected`; `journal_corruption_is_detected_without_rebuilding_from_unverified_bytes` |
| Crash/restart | `torn_tail_requires_explicit_recovery_without_duplicate_commit` |
| Concurrency | `journal_has_one_process_owner_at_a_time` |
| Canonical state | `post_state_commitment_is_for_the_full_public_dataset_not_turtle_order`; `post_state_commitment_is_invariant_to_blank_node_labels`; `blank_nodes_are_scoped_per_payload_before_state_union` |

**Known limitations:** single-process journal authority; three-node behavior is
Milestone 3. No throughput claim.

## Milestone 2 — Universal Final Admission (Issue #3, `ledger_issue_3`)

**External behavior:** local, API, batch, PoA, replication, synchronization,
privacy, and bridge ingresses produce identical outcomes for the same candidate
and context; pre-commit rejection mutates nothing; one commit maximum under
retry or post-commit failure.

| Case class | Evidence (test → artifact) |
|---|---|
| Success (ingress universality) | `every_supported_adapter_commits_the_same_candidate_as_identical_envelope_bytes` → `logs/ledger_issue_3_default.log` |
| Rejection | `every_precommit_rejection_is_identical_and_preserves_all_authoritative_and_projected_state` |
| Response loss / retry | `exact_candidate_retry_across_all_adapters_has_one_commit_and_one_response_identity` |

**Known limitations:** adapters exercised are the ones implemented at this
seam; web-server ingress remains outside the reference-system claim boundary.

## Milestone 3 — Authenticated membership and three-node PoA (Issues #4–#6, `ledger_issue_4/5/6`)

**External behavior:** governance-signed manifest as sole membership source;
mutual Ed25519 peer authentication with fresh transcripts; deterministic
scheduled authority; crash-safe signing fence; exact envelope replication with
per-node commit receipts; three independent processes converge.

| Case class | Evidence (test → artifact) |
|---|---|
| Success (membership) | `valid_governance_manifest_activates_reference_node_before_ledger_admission` → `logs/ledger_issue_4_default.log` |
| Rejection (contracts) | `invalid_startup_contracts_fail_before_the_ledger_or_network_can_activate`; `handshake_rejects_replay_stale_modified_identity_and_manifest_attempts` |
| Authentication | `two_authorized_nodes_complete_fresh_mutual_identity_proof`; `session_capabilities_are_transport_bound_single_use_and_identity_unique`; `unauthenticated_transports_are_quarantined_from_ordinary_peer_messages`; `peer_authentication_does_not_grant_validator_signer_authorization` |
| Resource bounds | `abandoned_handshakes_are_ttl_pruned_and_globally_bounded` |
| Scheduling | `every_node_derives_the_same_scheduled_authority_from_height_and_manifest_bound_order`; `out_of_turn_validator_is_rejected_and_a_missed_turn_does_not_take_over` |
| Concurrency/equivocation | `concurrent_identical_requests_coalesce_and_conflicts_cannot_replace_the_selected_body`; `distinct_signed_proposals_for_one_turn_are_equivocation_not_timestamp_fork_choice` |
| Crash / response loss | `lost_response_restart_recovers_the_exact_fenced_proposal_without_a_second_commit` → `logs/ledger_issue_5_default.log` |
| Bounds / incapacity | `deterministic_envelope_size_failure_does_not_begin_selection_or_signing`; `block_interval_is_preflighted_for_local_and_received_signed_proposals` |
| Replication / receipts | `follower_final_admits_the_exact_producer_envelope_and_receipts_follow_local_fsync`; `three_identity_signed_matching_receipts_are_evidence_of_network_convergence` → `logs/ledger_issue_6_default.log` |
| Partition / rejoin | `partitioned_node_rejoins_with_bounded_exact_prefix_sync_and_retry_is_idempotent` |
| Restart | `producer_and_follower_restart_recover_exact_records_and_lost_receipts` |
| Multi-process | `three_independent_reference_node_processes_commit_byte_identical_journals` (subprocess entrypoint; one environment-conditional ignore exists in earlier integrations) |

**Known limitations:** sequential authenticated exact-envelope replication
across three reference processes, not concurrent long-lived P2P operation;
PBFT, threshold finality, dynamic membership, and production transport
hardening are out of scope (spec issue #1; ADR 0019/0020).

## Milestone 4 — Package-declared semantic admission (Issue #7, `ledger_issue_7`)

**External behavior:** the network binds one exact ontology package and
semantic execution profile; Final Admission validates the complete staged
post-block public state under the bounded SHACL/SPACL contract with
non-vacuous candidate focus.

| Case class | Evidence (test → artifact) |
|---|---|
| Success (activation) | `checked_in_default_package_activates_under_its_declared_profile`; `package_identity_is_path_independent_and_activation_binds_executable_assets` → `logs/ledger_issue_7_default.log` |
| Rejection (activation) | `unsupported_validating_construct_fails_package_activation`; `missing_empty_malformed_or_wrong_digest_assets_fail_activation`; `custom_constraints_and_ill_typed_profile_parameters_fail_activation` |
| Semantic failure | `semantic_rejection_issues_neither_journal_frame_nor_commit_receipt`; `unknown_asserted_classes_are_deterministic_nonconformance_not_engine_failure`; `datatype_rejects_ill_typed_sparql_supported_literals` |
| Non-vacuous focus | `final_admission_validates_full_staged_union_and_requires_candidate_focus`; `subclass_entailment_selects_focus_and_evaluates_class_constraints` |
| Restart | `restart_reconstructs_semantic_parent_state_from_verified_journal_history` |
| Multi-process | `three_reference_nodes_agree_after_independent_admission_and_exact_catch_up` |
| Ingress universality | `every_ordinary_ingress_observes_identical_semantic_verdict_and_envelope` |
| Resource incapacity | `deterministic_focus_work_bound_rejects_without_node_incapacity` inverted with `a_nonconforming_committed_parent_is_node_incapacity_not_candidate_rejection` |
| State hygiene | `semantic_data_graph_excludes_package_artifacts_and_inferred_triples`; `document_local_blank_nodes_do_not_merge_across_staged_graphs` |
| Constraint matrix | `supported_constraint_matrix_uses_rdf_terms_and_pinned_comparisons`; `integer_range_comparison_does_not_lose_precision`; `in_and_has_value_compare_complete_rdf_terms`; `temporal_range_comparison_is_pinned_and_complete` |

**Known limitations:** bounded locked SHACL construct set only (ADR 0025/0026);
no generic OWL2 reasoning or broad W3C SHACL conformance claim.

## Milestone 5 — Durable privacy lifecycle (Issues #8–#10, `ledger_issue_8/9/10`)

**External behavior:** closed `PrivacyControlV1` transitions; participant
principal/key lifecycle; one-ciphertext per-object-DEK protected objects;
client-only custody; converged live release; terminal prospective revocation;
no server-held plaintext or private keys anywhere.

| Case class | Evidence (test → artifact) |
|---|---|
| Success (registration) | `registration_verifies_both_proofs_and_stages_one_atomic_effective_state_delta` → `logs/ledger_issue_8_privacy-conformance.log` |
| Vectors (independent proof) | `checked_in_cross_implementation_vectors_match_active_lifecycle_surface` + `tests/vectors/privacy_lifecycle_v1.json` |
| Rejection | `canonical_decoder_rejects_ambiguous_unknown_malformed_and_unimplemented_forms`; `version_gaps_wrong_purpose_authorizers_and_global_key_reuse_are_rejected`; `invalid_privacy_proof_rejects_and_exact_retry_is_idempotent_without_mutation` |
| Privacy denial (revocation) | `wrapping_rotation_preserves_only_exact_historical_release_and_revocation_is_terminal`; `authorization_rotation_is_contiguous_and_self_revocation_freezes_participant_control` |
| Restart/replay | `final_admission_commits_control_only_bytes_and_replay_rebuilds_identical_privacy_state` |
| Trust separation | `privacy_bootstrap_key_cannot_reuse_consensus_or_membership_trust_roles` |
| Multi-process | `poa_replicates_one_exact_privacy_envelope_and_three_nodes_rebuild_identical_state` |
| Protected objects | `public_final_admission_accepts_exact_ciphertext_without_any_private_key`; `independent_protected_object_vectors_self_open_and_reject_context_tampering` (vectors: `tests/vectors/protected_object_v1.json`) → `logs/ledger_issue_9_privacy-conformance.log` |
| No-secret boundary | `exact_replication_artifacts_exclude_controlled_client_secrets` |
| Crash / custody | `custody_crash_boundaries_require_deterministic_recovery_before_disclosure`; `generated_custody_is_durable_before_disclosure_and_constructs_real_objects` |
| Release barrier | `converged_grant_release_is_opaque_and_revocation_is_prefix_bound` |
| Grants (vectors) | `independent_issue10_vectors_match_canonical_grant_and_release_surface` + `tests/vectors/privacy_grant_v1.json` → `logs/ledger_issue_10_default.log` |

**Known limitations:** no server-side decryption, key escrow, or generic
recovery (ADR 0028/0035/0036); production privacy activation remains fail
closed for unimplemented lifecycle variants.

## Milestone 6 — Bounded bridge (Issues #11–#12, `ledger_issue_11/12`)

**External behavior:** one-hop exact public `OrdinaryProvenanceV1` copy between
bound ProvChain instances; source export requires exactly three pinned durable
receipts; target imports only through its own PoA and Final Admission;
journal-derived replay state with idempotent replay.

| Case class | Evidence (test → artifact) |
|---|---|
| Success (source) | `source_final_admission_commits_one_exact_target_bound_declaration` (bridge-conformance); `export_requires_exactly_three_pinned_durable_receipts_and_reproduces_source_bytes` → `logs/ledger_issue_11_bridge-conformance.log` |
| Source crash/response loss | `source_declaration_obeys_journal_commit_crash_and_response_loss_boundaries` (bridge-conformance) |
| Source rejection | `source_bridge_activation_rejects_a_different_live_profile`; `closed_bridge_codec_rejects_tag_count_order_transfer_scheme_and_bound_mutations` |
| Vectors | `tests/vectors/bridge_export_v1.json` regenerated byte-identical by `scripts/generate_bridge_export_vectors.py` |
| Success (target import) | `valid_vector_import_commits_once_and_replay_returns_already_imported` → `logs/ledger_issue_12_default.log` |
| Bridge conflict | `invalid_evidence_consumes_no_id_and_changed_same_id_is_replay_conflict`; `active_target_trust_substitution_and_out_of_turn_signer_fail_closed`; `target_activation_rejects_source_or_semantic_trust_substitution` |
| Bounds / codec | `target_origin_codec_rejects_noncanonical_and_oversized_evidence` |
| Partition / convergence | `exact_target_envelope_replication_converges_bridge_state_after_partition` |
| Target crash / response loss / projection failure | `target_import_obeys_commit_projection_and_response_failure_boundaries` (bridge-conformance) → `logs/ledger_issue_12_bridge-conformance.log` |

**Known limitations:** bounded one-hop exact public-payload copying with
three-node reference evidence; genesis-position source prefix under ADR 0038
until a prefix-bound export version exists; no asset movement, exactly-once
delivery, cross-ledger atomicity, freshness, rollback protection, mapping,
private-data transfer, or multihop (ADR 0037).

## Milestone 7 — Corrected reproducible evidence (Issue #13, this workflow)

**External behavior:** an independent reviewer reruns the complete workflow
from a clean checkout and obtains stable canonical artifacts plus a mechanically
derived manifest and claim map.

| Case class | Evidence (test → artifact) |
|---|---|
| Success | campaign verdict `pass` in `manifest.json`; per-command exit codes and parsed counts in `manifest.json` |
| Provenance | `environment.env`, `provenance/` (revision, Cargo.lock SHA-256, `rustc -Vv`, OS, filesystem types, git state) |
| Raw artifacts | `logs/*.log`, regenerated `vectors/*.json` with SHA-256 comparisons |
| Rerun stability | `rerun_comparison.json` from `--compare` across two runs (including after machine restart) |
| Claim mapping | [`docs/paper_submission/PAPER_EVIDENCE_INDEX.md`](../paper_submission/PAPER_EVIDENCE_INDEX.md) traces every paper table, figure, and claim to an artifact or an explicit unsupported/descriptive status |

**Known limitations:** campaign time is dominated by the deterministic privacy
custody suite; host-specific storage-profile qualification (ADR 0036 host gate)
is recorded per run, and a host exposing duplicate ext4 mount records fails
that gate for environmental, not implementation, reasons.

## Cross-cutting case-class coverage (issue #13 criterion 3)

| Required case | Named evidence |
|---|---|
| success | every suite's primary commit test (see rows above) |
| rejection | `deterministic_rejections_leave_the_authoritative_journal_unchanged`; `every_precommit_rejection_is_identical_and_preserves_all_authoritative_and_projected_state`; activation rejections in #4/#7 |
| tamper | `noncanonical_and_tampered_envelopes_are_rejected`; `independent_protected_object_vectors_self_open_and_reject_context_tampering`; `invalid_evidence_consumes_no_id_and_changed_same_id_is_replay_conflict` |
| bounds | `deterministic_envelope_size_failure_does_not_begin_selection_or_signing`; `target_origin_codec_rejects_noncanonical_and_oversized_evidence`; `closed_bridge_codec_rejects_tag_count_order_transfer_scheme_and_bound_mutations` |
| crash | `lost_response_restart_recovers_the_exact_fenced_proposal_without_a_second_commit`; `custody_crash_boundaries_require_deterministic_recovery_before_disclosure`; `source_declaration_obeys_journal_commit_crash_and_response_loss_boundaries`; `target_import_obeys_commit_projection_and_response_failure_boundaries` |
| response loss | same crash-family tests plus `producer_and_follower_restart_recover_exact_records_and_lost_receipts` |
| projection failure | `journal_corruption_is_detected_without_rebuilding_from_unverified_bytes`; `target_import_obeys_commit_projection_and_response_failure_boundaries` |
| restart | `torn_tail_requires_explicit_recovery_without_duplicate_commit`; `restart_reconstructs_semantic_parent_state_from_verified_journal_history`; `final_admission_commits_control_only_bytes_and_replay_rebuilds_identical_privacy_state` |
| partition | `partitioned_node_rejoins_with_bounded_exact_prefix_sync_and_retry_is_idempotent`; `exact_target_envelope_replication_converges_bridge_state_after_partition` |
| rejoin | `partitioned_node_rejoins_with_bounded_exact_prefix_sync_and_retry_is_idempotent`; `three_reference_nodes_agree_after_independent_admission_and_exact_catch_up` |
| semantic failure | `semantic_rejection_issues_neither_journal_frame_nor_commit_receipt`; `unknown_asserted_classes_are_deterministic_nonconformance_not_engine_failure` |
| privacy denial | `wrapping_rotation_preserves_only_exact_historical_release_and_revocation_is_terminal`; `converged_grant_release_is_opaque_and_revocation_is_prefix_bound`; `authorization_rotation_is_contiguous_and_self_revocation_freezes_participant_control` |
| bridge conflict | `invalid_evidence_consumes_no_id_and_changed_same_id_is_replay_conflict`; `active_target_trust_substitution_and_out_of_turn_signer_fail_closed` |
| resource incapacity | `deterministic_focus_work_bound_rejects_without_node_incapacity`; `a_nonconforming_committed_parent_is_node_incapacity_not_candidate_rejection`; `deterministic_envelope_size_failure_does_not_begin_selection_or_signing` |
