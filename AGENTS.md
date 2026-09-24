# Agent Routing Index

> Agent doc file nay dau tien de biet minh co vu khi gi.
> Chi doc SKILL.md tuong ung khi task thuc su can den skill do — tiet kiem token toi da.

## Core Rules (Always Active)

- `.agents/rules/01-core-philosophy.md` — Triet ly code: KISS, YAGNI, Surgical, Goal-Driven
- `.agents/rules/02-security-baseline.md` — Baseline bao mat toi thieu cho moi project
- `.agents/rules/03-migration-traceability.md` — Quy chuan chuyen dich Kotlin -> Rust & ma tran truy vet 1:1

---

## Skills (Load khi can — On-Demand)

| Skill ID | Kich hoat khi... | File |
|----------|-----------------|------|
| `spec-brainstormer` | Khai thac intent, lam ro spec, phan loai Spike/Bounded/Architecture | `.agents/skills/spec-brainstormer/SKILL.md` |
| `subagent-orchestrator` | Dieu phoi multi-agent, chay task doc lap, continuous execution | `.agents/skills/subagent-orchestrator/SKILL.md` |
| `systematic-debugging` | Go loi logic phuc tap, crash, memory leak theo First-Principles | `.agents/skills/systematic-debugging/SKILL.md` |
| `unit-testing-tdd` | Viet Unit test & Integration test theo chuan TDD | `.agents/skills/unit-testing-tdd/SKILL.md` |
| `adversarial-review` | Peer code review phan bien, ra soat regression, edge cases, bao mat | `.agents/skills/adversarial-review/SKILL.md` |
| `engineering-workflows` | Refactoring phuc tap, API design, database migration | `.agents/skills/engineering-workflows/SKILL.md` |
| `deep-skill-search` | Can skill rat dac thu khong co trong hub nay | `.agents/skills/deep-skill-search/SKILL.md` |
| `database-mastery` | Toi uu SQL Query, Indexing, Locking, Redis Caching | `.agents/skills/database-mastery/SKILL.md` |
| `security-auditor` | Review bao mat, kiem tra OWASP, pentest, phan tich lo hong | `.agents/skills/security-auditor/SKILL.md` |
| `diagram-designer` | Ve kien truc, workflow, sequence, ERD, C4, HTML+SVG | `.agents/skills/diagram-designer/SKILL.md` |
| `framework-standards` | Code theo dung convention cua framework (Next.js, NestJS...) | `.agents/skills/framework-standards/SKILL.md` |

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **flowspeed.link** (17899 symbols, 38976 relationships, 703 execution flows).

> Index stale? Run `node .gitnexus/run.cjs analyze --index-only` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? Bootstrap with `npx`, `bunx`, or `pnpm dlx` — e.g. `bunx gitnexus@latest analyze` (npm 11 npx crash; #1939).

## Always Do

- **MUST run impact before editing.** Use `impact({target: "symbolName", direction: "upstream"})` or `node .gitnexus/run.cjs impact "symbolName" --direction upstream --repo .`; report callers, processes, and risk. Never substitute grep for graph analysis.
- **MUST analyze graph changes before committing.** Use `detect_changes({scope: "all"})` (MCP) or `node .gitnexus/run.cjs detect-changes --scope all --repo .` (CLI fallback). `partial: true` or `truncated: true` is not a clean check — a zero means unseen, not unaffected; re-run it. For regression review: `detect_changes({scope: "compare", base_ref: "main"})` or `node .gitnexus/run.cjs detect-changes --scope compare --base-ref "main" --repo .`.
- MUST warn on HIGH/CRITICAL `risk` pre-edit; never use `riskSharedAxes` to waive a HIGH/CRITICAL `risk` warning. Compare File/symbol: MCP File omits axes; Graph-RAG expands File.
- **MUST treat `risk: UNKNOWN` as unresolved, not as low.** An empty caller set is not evidence the symbol is unused — it can also mean the callers are not resolvable by the index (plain-object property access, dynamic dispatch, cross-language calls). `impact` pairs `UNKNOWN` with a `riskNote` saying so. Confirm with a text search before treating the symbol as safe to change or delete; do not proceed on the strength of a zero.
- **MUST use `query({search_query: "concept"})` for concepts/flows, `context({name: "symbolName"})` for a named symbol, or `impact` for blast radius, on read-only callers, dependencies, imports, or execution flow.** Graph first; text search only for empty/`UNKNOWN`/literals.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method before MCP/CLI impact analysis.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis, and never read `UNKNOWN` as an all-clear — it means the walk could not answer, which is the one verdict that requires confirming by other means.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit before MCP/CLI graph change analysis.

## Resources

| Resource | Use for |
| --- | --- |
| `gitnexus://repo/flowspeed.link/context` | Codebase overview, check index freshness |
| `gitnexus://repo/flowspeed.link/clusters` | All functional areas |
| `gitnexus://repo/flowspeed.link/processes` | All execution flows |
| `gitnexus://repo/flowspeed.link/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
| --- | --- |
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
