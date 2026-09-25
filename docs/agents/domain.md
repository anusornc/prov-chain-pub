# Domain Docs

ProvChainOrg is configured as a single-context repository.

These rules supplement the mandatory startup context in `AGENTS.md`; when that file requires additional architecture or publication documents, read those as well.

## Before exploring

- Read the root `CONTEXT.md`.
- Read relevant ADRs under `docs/architecture/ADR/`.
- If either resource does not exist, proceed silently.

## Layout

```text
/
├── CONTEXT.md
├── docs/
│   └── architecture/
│       └── ADR/
└── src/
```

## Vocabulary

Use the terminology defined in `CONTEXT.md` in issue titles, specifications, tests, and implementation discussions. Avoid synonyms explicitly rejected by the glossary.

If a required domain concept is missing, record it through `/domain-modeling` when the concept is resolved.

## ADR conflicts

If proposed work contradicts an existing ADR, surface the conflict explicitly instead of silently overriding the decision.
