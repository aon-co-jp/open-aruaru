# aruaru-ai TOP100 Programming Languages

## Problem

The aruaru-ai development menu should not only list programming languages. It should show a daily-updated TOP100 popularity view with:

- characteristics
- merits
- demerits
- author / governance
- author policy, philosophy, or public claim when available
- aruaru-ai recommendation
- evidence source type

## Cause

A single popularity ranking is dangerous. TIOBE is a popularity indicator, not a best-language ranking. GitHub Octoverse measures repository activity. Stack Overflow measures usage and developer sentiment. PYPL measures search interest. These signals must be separated.

## Fix policy

aruaru-ai now treats TOP100 as a composite report:

1. crawl public allowlisted sources daily
2. keep TIOBE, GitHub, Stack Overflow, PYPL, official documentation, and aruaru policy as separate fields
3. summarize each language into features / merits / demerits / author policy
4. never show popularity as technical superiority
5. keep daily history and rollback

## Ruby policy

Ruby is visible because it is historically important, created by Yukihiro Matsumoto, and has many Japanese resources. However, aruaru-ai marks Ruby as `NotRecommendedByDefault` for new standard development.

Reason:

- expressive dynamic code can strongly reflect the first developer's style
- later handoff can become difficult if conventions, tests, and documentation are weak
- Ruby/Rails maintenance remains supported, but new default projects should prefer Rust + TypeScript unless there is a clear reason

## Output files

- `data/language-top100/latest.json`
- `data/language-top100/history/YYYY-MM-DD.json`
- `data/language-top100/diff/YYYY-MM-DD.md`
- `ARUARU_AI_TOP100_PROGRAMMING_LANGUAGES.md`

## UI

Each TOP100 row should show:

- rank
- language
- popularity basis
- characteristics
- merits
- demerits
- author / governance
- author policy / claim
- aruaru-ai recommendation
- source freshness
- confidence

## Quality gate

```powershell
cargo test language_top100
cargo check
cargo clippy --all-targets -- -D warnings
```
