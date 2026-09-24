# Agent Routing Index

> Agent doc file nay dau tien de biet minh co vu khi gi.
> Chi doc SKILL.md tuong ung khi task thuc su can den skill do â€” tiet kiem token toi da.

## Core Rules (Always Active)

- `.agents/rules/01-core-philosophy.md` â€” Triet ly code: KISS, YAGNI, Surgical, Goal-Driven
- `.agents/rules/02-security-baseline.md` â€” Baseline bao mat toi thieu cho moi project

---

## Skills (Load khi can â€” On-Demand)

| Skill ID | Kich hoat khi... | File |
|----------|-----------------|------|| `spec-brainstormer` | Khai thac intent, lam ro spec, phan loai Spike/Bounded/Architecture | `.agents/skills/spec-brainstormer/SKILL.md` |
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
