# Issue tracker: GitHub

Issues and PRDs for this repo live in GitHub Issues for `anusornc/prov-chain`. Use the `gh` CLI for all operations and pass `--repo anusornc/prov-chain` when the repository is ambiguous.

## Conventions

- **Create an issue**: `gh issue create --repo anusornc/prov-chain --title "..." --body "..."`
- **Read an issue**: `gh issue view <number> --repo anusornc/prov-chain --comments`
- **List issues**: `gh issue list --repo anusornc/prov-chain --state open --json number,title,body,labels,comments`
- **Comment**: `gh issue comment <number> --repo anusornc/prov-chain --body "..."`
- **Apply or remove labels**: use `gh issue edit` with `--add-label` or `--remove-label`
- **Close**: `gh issue close <number> --repo anusornc/prov-chain --comment "..."`

## Pull requests as a triage surface

**PRs as a request surface: no.**

GitHub shares one number space across issues and pull requests. If a bare issue number is ambiguous, try `gh pr view` and fall back to `gh issue view`.

## Skill operations

- When a skill says **publish to the issue tracker**, create a GitHub issue.
- When a skill says **fetch the relevant ticket**, run `gh issue view <number> --repo anusornc/prov-chain --comments`.

## Wayfinding operations

The wayfinding map is one GitHub issue whose child issues represent tickets.

- **Map**: label with `wayfinder:map`.
- **Child ticket**: use a GitHub sub-issue when available; otherwise link it through a task list and add `Part of #<map>` to the child.
- **Ticket type**: label with `wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`, or `wayfinder:task`.
- **Blocking**: use native GitHub issue dependencies when available; otherwise add `Blocked by: #<n>` to the child.
- **Claim**: assign the ticket to the active developer.
- **Resolve**: record the answer in a comment, close the ticket, and update the map's Decisions-so-far.
