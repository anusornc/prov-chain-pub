# Structural Audit: Node Startup, Config, Consensus, and Ontology Runtime

**Date:** 2026-03-09
**Status:** Completed
**Purpose:** Establish the current execution truth before any network-profile or ontology-package implementation

## Scope

This audit covers:

- node startup path,
- runtime configuration sources,
- ontology initialization path,
- semantic validation path,
- and current network consistency enforcement.

## Findings

### 1. `start-node` now conditionally uses the ontology-aware blockchain path and local semantic-contract preflight

The `start-node` command now loads `NodeConfig` via `load_config` and creates the blockchain through a node-specific helper that selects:

- `Blockchain::new_persistent_with_config_and_ontology` when ontology startup is enabled
- `Blockchain::new_persistent_with_config` otherwise

Evidence:

- [main.rs](/home/cit/provchain-org/src/main.rs#L637)
- [main.rs](/home/cit/provchain-org/src/main.rs#L339)
- [blockchain.rs](/home/cit/provchain-org/src/core/blockchain.rs#L540)
- [blockchain.rs](/home/cit/provchain-org/src/core/blockchain.rs#L550)
- [blockchain.rs](/home/cit/provchain-org/src/core/blockchain.rs#L230)

Impact:

- networked node startup can now initialize ontology manager and SHACL validator at startup,
- it can reject startup when the local node config does not match a declared network profile or ontology package manifest,
- but ontology hash participation is still local-only and not enforced in network bootstrapping.

### 2. The repository still uses two configuration entrypoints, but they now share the same core runtime types

There is a split between:

- `src/config/mod.rs` used by general application paths
- `src/utils/config.rs` used by node runtime startup

Evidence:

- [config/mod.rs](/home/cit/provchain-org/src/config/mod.rs#L7)
- [utils/config.rs](/home/cit/provchain-org/src/utils/config.rs#L15)
- [main.rs](/home/cit/provchain-org/src/main.rs#L314)
- [main.rs](/home/cit/provchain-org/src/main.rs#L639)

Impact:

- duplicate network, consensus, storage, and logging struct definitions have been removed from `src/config/mod.rs`,
- but there is still no single canonical config entrypoint for all runtime paths,
- and ontology-aware CLI flows and node startup flows still diverge in how they consume config.

### 3. The node config model is still split, but both config entrypoints now support defaults and the default config file is aligned with `NodeConfig`

`config/config.toml` now defines `consensus_type` and ontology startup fields, `NodeConfig` supports serde defaults for missing fields, and `config::Config` now also supports partial TOML defaults for its own fields.

Evidence:

- [utils/config.rs](/home/cit/provchain-org/src/utils/config.rs#L64)
- [utils/config.rs](/home/cit/provchain-org/src/utils/config.rs#L335)
- [config.toml](/home/cit/provchain-org/config/config.toml#L30)

Impact:

- the default startup config path is now usable,
- the shared type drift between `Config` and `NodeConfig` is lower than before,
- but the repository still has two different config entrypoints and no single canonical runtime contract.

### 4. Ontology consistency support exists and local startup can now use it, but network runtime still does not enforce it

The ontology path already supports:

- ontology hash generation,
- ontology hash access,
- and explicit ontology consistency checking.

Evidence:

- [ontology/mod.rs](/home/cit/provchain-org/src/ontology/mod.rs#L66)
- [domain_manager.rs](/home/cit/provchain-org/src/ontology/domain_manager.rs#L408)
- [domain_manager.rs](/home/cit/provchain-org/src/ontology/domain_manager.rs#L529)

The discovery path now checks `network_id` and semantic contract metadata during peer discovery:

- [discovery.rs](/home/cit/provchain-org/src/network/discovery.rs#L203)

Additional evidence:

- [profile.rs](/home/cit/provchain-org/src/network/profile.rs)
- [package.rs](/home/cit/provchain-org/src/ontology/package.rs)
- [config.toml](/home/cit/provchain-org/config/config.toml#L6)
- [network_profile.toml](/home/cit/provchain-org/config/network_profile.toml)
- [ontology_package.toml](/home/cit/provchain-org/config/ontology_package.toml)

Impact:

- semantic consistency is now enforceable during local startup,
- is enforceable in the discovery path during runtime message handling,
- is now tied to a stable transport-vs-logical peer identity split in the runtime path,
- but is still not tied to consensus admission.

### 5. Peer handshake carries no semantic contract metadata

Current P2P discovery messages include:

- node ID,
- listen port,
- network ID,
- timestamp

Evidence:

- [messages.rs](/home/cit/provchain-org/src/network/messages.rs#L20)
- [messages.rs](/home/cit/provchain-org/src/network/messages.rs#L123)

Missing fields include:

- ontology package ID,
- ontology hash,
- SHACL hash,
- consensus profile hash,
- package version.

Impact:

- a node cannot verify shared ontology-package compatibility during handshake.

Update on 2026-03-09:

- the standalone discovery/message layer now carries semantic contract metadata and can reject mismatched peers,
- and the main `NetworkManager` runtime now routes discovery messages through that semantic-aware path,
- and the runtime now keeps a stable transport ID per connection while updating discovered logical node identity separately,
- and logical-node routing now resolves through a logical-to-transport mapping,
- and duplicate logical node IDs now default to rejecting the newer transport connection.

### 6. Consensus authority identity is now derived deterministically from authority public keys

PoA and PBFT now derive authority IDs from the configured authority public keys instead of generating random UUIDs per node.

Additional behavior now in place:

- authority nodes load or generate their configured authority key consistently across PoA and PBFT,
- authority nodes fail fast when their local authority key is not part of the configured authority set,
- PBFT primary selection is now based on the deterministic authority set,
- and PoA single-authority startup now seeds the authority set and rotation order from the local authority key when no authority list is configured.

Impact:

- nodes now derive the same in-memory authority ordering from the same public key list,
- PBFT primary selection is no longer tied to local random IDs,
- and consensus startup now rejects one important class of silent authority misconfiguration.

### 7. The blockchain admission path now routes through the ontology-manager validation path, and class-constraint reasoning coverage is stronger than before

When the ontology-aware blockchain path is used, `initialize_ontology_system` now creates an `OntologyManager` and stores a clone of its validator instead of constructing a second validator with `reasoner: None`.

- `OntologyManager`
- `ShaclValidator`

Impact:

- runtime transaction admission now calls `OntologyManager::validate_transaction(...)` first,
- the stored blockchain validator now preserves the SPACL-backed reasoner cloned from `OntologyManager`,
- `sh:class` validation now enforces exact RDF type matches even when no reasoner is available,
- subclass acceptance is now recorded explicitly when the SPACL-backed reasoner satisfies a class constraint through ontology hierarchy,
- validation results now expose reasoning metadata for class constraints,
- invalid admission failures now emit deterministic summaries with constraint and shape breakdowns,
- and a focused Criterion benchmark now exists for exact-class fallback, subclass reasoning, and failure-summary generation,
- but the current admission path still relies on validator-centered reasoning hooks rather than a broader inference or explanation pipeline.

### 8. `start-node` now consumes the ontology section from `NodeConfig` and can derive runtime ontology config from an ontology package manifest

`NodeConfig` includes an `ontology` field:

- [utils/config.rs](/home/cit/provchain-org/src/utils/config.rs#L33)

`start-node` now converts that into `crate::ontology::OntologyConfig` and uses it during blockchain initialization:

- [main.rs](/home/cit/provchain-org/src/main.rs#L637)
- [main.rs](/home/cit/provchain-org/src/main.rs#L339)

Impact:

- the node config model now participates in ontology-aware startup,
- the node can load a formal ontology package manifest as a first-class artifact,
- but semantic contract enforcement still stops at local node initialization and has not yet been extended to peer handshake or network admission.

### 9. The repository now has first-class models for the intended network contract artifacts

Formal models now exist for:

- a network-wide profile object,
- a deployable ontology package manifest,
- local startup validation against both artifacts,
- and runtime ontology initialization directly from the package manifest.

Evidence:

- [profile.rs](/home/cit/provchain-org/src/network/profile.rs)
- [package.rs](/home/cit/provchain-org/src/ontology/package.rs)
- [main.rs](/home/cit/provchain-org/src/main.rs#L369)
- [main.rs](/home/cit/provchain-org/src/main.rs#L652)

Impact:

- the intended architecture now has concrete runtime artifacts instead of only documentation,
- and the network runtime now partially enforces the semantic contract through discovery,
- but the transport identity model is still transitional.

### 10. The discovery protocol now supports semantic contract exchange and mismatch rejection

The discovery/message layer now includes:

- semantic contract metadata in `PeerDiscovery` messages,
- semantic contract metadata in `PeerInfo`,
- explicit `SemanticMismatch` errors,
- and rejection of incompatible peers inside `PeerDiscovery::handle_peer_discovery`.

Evidence:

- [messages.rs](/home/cit/provchain-org/src/network/messages.rs)
- [discovery.rs](/home/cit/provchain-org/src/network/discovery.rs)
- [profile.rs](/home/cit/provchain-org/src/network/profile.rs)

Impact:

- the intended handshake fields now exist in code,
- semantic mismatch can be detected and rejected in the discovery layer,
- and `NetworkManager` now uses that discovery path during live message processing,
- and logical peer identity is updated after successful discovery,
- but provisional transport identity still exists before discovery completes.

## Safe Conclusions

The current repository already contains much of the semantic-network foundation:

- ontology hashing,
- ontology manager construction,
- SHACL transaction validation,
- consensus selection by config,
- and network ID checking.

However, those capabilities are not yet joined into a single network-semantic startup model.

## Safe Next Steps

1. Make the node runtime config path structurally coherent
2. Choose one canonical config model for node startup
3. Define reconnect or takeover semantics beyond the default duplicate-logical-node rejection policy
4. Unify the node runtime config story without creating a third competing config system
5. Expand ontology-aware runtime admission beyond the current validator-centered class-reasoning, failure-summary, and focused benchmark coverage

## Update Note

This audit was updated after the first remediation pass on 2026-03-09:

- node startup now supports ontology-aware blockchain initialization
- node config parsing now supports missing-field defaults
- `src/config/mod.rs` now reuses runtime config types from `src/utils/config.rs`
- default `config/config.toml` is aligned with the node runtime model
- startup now validates `config/network_profile.toml` and `config/ontology_package.toml` when configured
- consensus authority identity is now derived deterministically from authority public keys across PoA and PBFT
- discovery messages now carry semantic contract metadata and reject semantic mismatches in the discovery module
- `NetworkManager` now initializes discovery state and processes semantic-aware discovery messages in the runtime path
- transport connection IDs are now separated from discovered logical node IDs in the runtime peer map
- duplicate logical node IDs now default to rejecting the newer transport connection
- blockchain ontology-aware admission now routes through `OntologyManager::validate_transaction(...)`
