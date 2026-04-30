# ProvChain Ledger Cold-Load Vs Steady-State Profile - 2026-04-29

This note records the R002 conservative-durability profile that separates
cold-load cost from steady-state append cost.

This is reference/profiling evidence only. It is not primary paper comparison
evidence and must not be used as a cross-system ledger/write win table.

## Campaign

- campaign: `20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3`
- curated export: `docs/benchmarking/data/reference/20260429_profile_ledger_supply1000_provchain_only_coldsteady_conservative_n3/`
- status: `passed`, `3/3` epochs
- append iterations per epoch: `3`
- cold-load samples: `3`
- steady-state append samples: `9`
- durability mode: `conservative_sync_every_block`
- WAL sync interval: `1`
- chain-index sync interval: `1`
- RDF snapshot flush interval: `100`

## Result Summary

| Metric | Samples | Mean | p95 | p99 |
|---|---:|---:|---:|---:|
| Cold Turtle RDF import | 3 | `50736.000 ms` | `51061.600 ms` | `51107.520 ms` |
| Cold-load client submit loop | 3 | `48162.729 ms` | `48488.438 ms` | `48534.370 ms` |
| Steady-state append, `100 tx` | 9 | `17972.444 ms` | `20141.800 ms` | `20193.960 ms` |
| Steady-state client submit loop | 9 | `16713.070 ms` | `18873.838 ms` | `18917.555 ms` |
| Diagnostic batch-block, `100 triples, 1 block` | 9 | `1766.770 ms` | `1803.113 ms` | `1808.423 ms` |

## Server Stage Summary

| Phase | Stage | Samples | Mean |
|---|---|---:|---:|
| Cold load | `handler_total` | 3 | `74.080 ms/tx` |
| Cold load | `block_admission` | 3 | `69.342 ms/tx` |
| Steady-state append | `handler_total` | 9 | `164.899 ms/tx` |
| Steady-state append | `block_admission` | 9 | `161.195 ms/tx` |

## Interpretation

The cold-load and steady-state append paths are now separated in artifacts:

- `Turtle RDF Import` rows measure loading the dataset into a fresh managed
  ProvChain runtime.
- `Steady-state Append After Cold Load (100 tx)` rows measure appending `100`
  new blocks after the dataset has already been loaded.
- The post-load append path is slower than the earlier no-load profiling rows,
  which confirms that state size materially affects per-block admission work.
- `block_admission` remains dominant in both phases and accounts for `97.75%`
  of mean steady-state handler time in this campaign.

The batch-block diagnostic is still much faster because it admits `100` triples
as one block, but that is different ledger semantics from `100` independent
transactions and should remain a diagnostic row.

## Reproduction

```bash
./benchmark-toolkit/scripts/provchain-ledger-r002-campaign.sh profile \
  --durability conservative \
  --id 20260429_profile_ledger_supply1000_provchain-only_coldsteady_conservative_n3 \
  --skip-preflight
```

The wrapper runs a managed ProvChain runtime per epoch, enables cold-load rows
with `--include-load`, skips Fabric for ProvChain-only profiling, exports a
curated evidence bundle, and regenerates `provchain_profile_summary.*`.
