# ProvChain Ledger Profile Comparison - 2026-04-29

This reference note compares ProvChain-only profiling campaigns used for
R002 write-path remediation analysis.

This is not primary paper comparison evidence. It supports engineering and
methods/limitations discussion only.

## Campaigns

| Role | Campaign | Curated Export |
|---|---|---|
| Before scoped optimization | `20260429_profile_ledger_supply1000_provchain-only_n3_fix1` | `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_n3_20260429/` |
| After RDF snapshot flush batching | `20260429_profile_ledger_supply1000_provchain-only_flush100_n3` | `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_flush100_n3_20260429/` |
| After RDF snapshot flush batching + state-root cache | `20260429_profile_ledger_supply1000_provchain-only_staterootcache_n3` | `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_staterootcache_n3_20260429/` |
| After RDF snapshot flush batching + state-root cache + WAL/index fsync batching | `20260429_profile_ledger_supply1000_provchain-only_walsync100_n3` | `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_walsync100_n3_20260429/` |
| Batch-block diagnostic after the same relaxed fsync settings | `20260429_profile_ledger_supply1000_provchain-only_batchblock_n3` | `docs/benchmarking/data/reference/profiling_ledger_supply1000_provchain_only_batchblock_n3_20260429/` |
| Cold-load vs steady-state append profile with conservative fsync | `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3` | `docs/benchmarking/data/reference/20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3/` |

## Before And After

| Metric | Before mean | Flush batching mean | State-root cache mean | WAL/index fsync batching mean | Cumulative change |
|---|---:|---:|---:|---:|---:|
| Batch total, 100 tx | `11967.556 ms` | `10695.444 ms` | `6011.111 ms` | `5926.444 ms` | `50.48%` faster |
| Client submit loop, 100 tx | `10306.832 ms` | `9044.055 ms` | `4344.425 ms` | `4279.226 ms` | `58.48%` faster |
| Server handler average | `101.547 ms/tx` | `88.971 ms/tx` | `41.992 ms/tx` | `41.349 ms/tx` | `59.28%` faster |
| Server block admission average | `97.731 ms/tx` | `85.207 ms/tx` | `38.197 ms/tx` | `37.618 ms/tx` | `61.51%` faster |
| Block admission share of handler time | `96.24%` | `95.77%` | `90.96%` | `90.98%` | still dominant |

## Interpretation

RDF snapshot flush batching reduces repeated full-store `store.nq` serialization
cost. State-root hash caching removes much of the repeated full-store hashing and
sorting cost from each block proposal. Together they cut ProvChain-only batch
latency by about half in this profiling workload. WAL/index fsync batching adds
a small incremental improvement after state-root caching, but it changes the
durability profile to relaxed batched fsync mode for profiling only.

`block_admission` remains the dominant measured server stage even after caching,
which means the next R002 work should target remaining native admission internals
such as block batching semantics and reducing per-block admission work.

## Batch-Block Diagnostic

The follow-up batch-block campaign added a diagnostic row named
`Batched Write (100 triples, 1 block)` while keeping the existing
`Single-threaded Write (100 tx)` row. In the same `n3` campaign, the existing
`100 tx` path had mean `6434.444 ms`, while the diagnostic batch-block path had
mean `1646.265 ms`, p95 `1678.490 ms`, and p99 `1687.068 ms` for `100` triples
in one block.

This is a `74.41%` lower client-observed mean latency within that diagnostic
campaign, but it is a semantics change rather than a direct optimization of the
existing `100 tx` metric. The batch-block row should be used to discuss
per-block overhead and future batch-ingest API design, not as a replacement for
ledger/write comparison tables.

## Cold-Load Vs Steady-State Profile

The cold/steady campaign uses conservative WAL/index fsync settings and
`--include-load` so the artifacts separate:

- `Turtle RDF Import`: cold-load cost on a fresh managed ProvChain runtime.
- `Steady-state Append After Cold Load (100 tx)`: append cost after the dataset
  has already been admitted.

In campaign `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3`,
the cold-load mean was `50736.000 ms` for `632` triples and the post-load
append mean was `17972.444 ms` per `100 tx`. Server-side post-load
`block_admission` averaged `161.195 ms/tx`, which confirms that state size
materially affects per-block admission work.

## Claim Boundary

- This evidence can support the statement that scoped persistence snapshot
  batching, state-root caching, and relaxed WAL/index fsync batching improved
  ProvChain-only profiling latency by about `50.48%`.
- The batch-block diagnostic can support the statement that collapsing `100`
  triples into one block reduces measured per-block overhead, under explicitly
  different semantics.
- The cold/steady profile can support a methods statement that cold-load and
  post-load append costs are now separated and should not be mixed in paper
  tables.
- This evidence cannot support a cross-system ledger/write performance claim.
- The WAL/index fsync batching result is relaxed-durability profiling evidence;
  production/default storage remains conservative sync-every-block.
- The batch-block diagnostic is also relaxed-durability profiling evidence and
  cannot be compared directly with Fabric batch commit unless equivalent batch
  semantics and finality boundaries are defined.
- Full comparative campaigns must be rerun after deeper admission optimization.
