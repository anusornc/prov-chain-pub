# Benchmark Datasets

This directory contains benchmark input datasets mounted into the Docker
benchmark stack.

## Current Slices

| Dataset | Role | Notes |
|---|---|---|
| `supply_chain_1000.ttl` | primary/reference input | Original benchmark slice used by current paper-facing campaigns. |
| `supply_chain_5000.ttl` | scale-up confidence input | Generated deterministic larger slice; not primary paper evidence until a campaign passes validity gates. |
| `supply_chain_10000.ttl` | scale-up confidence input | Generated deterministic larger slice; not primary paper evidence until a campaign passes validity gates. |

Each generated scale-up slice has a sibling `*.manifest.json` containing the
generator, logical batch count, SHA-256 checksum, byte size, and evidence role.

## Regeneration

```bash
./benchmark-toolkit/scripts/generate-scale-dataset.py \
  --target-triples 5000 \
  --output benchmark-toolkit/datasets/supply_chain_5000.ttl

./benchmark-toolkit/scripts/generate-scale-dataset.py \
  --target-triples 10000 \
  --output benchmark-toolkit/datasets/supply_chain_10000.ttl
```

Scale-up campaign results should be exported under
`docs/benchmarking/data/reference/` or another explicitly reference-only
location unless and until they are promoted into the paper evidence index.
