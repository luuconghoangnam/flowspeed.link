---
name: deep-skill-search
description: >
  Use when you need a very specific skill not covered by other skills in this hub:
  Kubernetes, gRPC, WebAssembly, GraphQL, Terraform, Docker advanced, ML/AI ops,
  specific cloud services (AWS Lambda, GCP, Azure), niche languages/frameworks.
  Triggers: any specialized technical topic not covered by other skills.
  Source: agentic-awesome-skills (local catalog of 2100+ skills).
---

# Deep Skill Search

This skill taps into the local catalog of 2100+ skills from `agentic-awesome-skills`.
Do NOT load this catalog into context — search it on-demand via CLI.

## Xac dinh thu muc Skills (Auto-Discovery)

Thu muc `skills/` nam cung cap voi thu muc Hub. Agent tu xac dinh bang cach:
```powershell
# Tu dong tim thu muc skills tu vi tri Hub hien tai
$hubDir = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$skillsDir = if ($env:SKILLS_PATH -and (Test-Path $env:SKILLS_PATH)) { $env:SKILLS_PATH } else { Join-Path (Split-Path $hubDir -Parent) "skills" }
```

> Neu da set bien moi truong `$env:SKILLS_PATH`, Agent se dung gia tri do. Neu khong, mac dinh la thu muc `skills/` nam cung cap voi Hub.

## Search the Catalog

```powershell
# Search for any skill topic (thay $skillsDir bang duong dan thuc te)
$catalog = Join-Path $skillsDir "agentic-awesome-skills"

# List all available skills
Get-ChildItem $catalog -Recurse -Filter "SKILL.md" | Select-Object FullName

# Search by keyword
Get-ChildItem $catalog -Recurse -Filter "SKILL.md" |
  Select-String -Pattern "<your-keyword>" |
  Select-Object Path -Unique
```

## Read a Specific Skill

```powershell
# Once you find the relevant skill file, read it
Get-Content (Join-Path $skillsDir "agentic-awesome-skills\<path>\SKILL.md") -Raw
```

## Browse awesome-cursorrules for Framework Rules

```powershell
# Find framework-specific .mdc rules
Get-ChildItem (Join-Path $skillsDir "awesome-cursorrules") -Recurse -Filter "*.mdc" |
  Select-String -Pattern "<framework>" |
  Select-Object Path -Unique
```

## Browse awesome-agent-skills for Workflow Patterns

```powershell
# Find agent workflow patterns
Get-ChildItem (Join-Path $skillsDir "awesome-agent-skills") -Recurse -Filter "*.md" |
  Select-String -Pattern "<topic>" |
  Select-Object Path -Unique
```

## Browse tech-leads-club for Audited Agent Skills

> Registry có cật gác bảo mật, skills đã qua review cho Cursor, Claude Code, Antigravity, Copilot.

```powershell
# Search catalog online
Start-Process "https://github.com/tech-leads-club/agent-skills"

# Hoặc nếu đã clone local:
$tl = Join-Path $skillsDir "tech-leads-club-agent-skills"
if (Test-Path $tl) {
  Get-ChildItem $tl -Recurse -Filter "*.md" |
    Select-String -Pattern "<topic>" |
    Select-Object Path -Unique
}
```

## MCP Official Tools (mcp-servers)

For tools requiring live connection (Postgres, Git, Memory, Fetch):

```powershell
# Browse available MCP server implementations
Get-ChildItem (Join-Path $skillsDir "mcp-servers\src") -Directory | Select-Object Name
```

Available official MCP servers:
- `postgres` — Query PostgreSQL databases directly
- `git` — Git repository operations
- `memory` — Persistent key-value memory across sessions
- `fetch` — HTTP fetch with content extraction
- `sequential-thinking` — Multi-step reasoning tool
- `filesystem` — Secure file system access

> After finding a relevant skill, read ONLY that skill file — never dump the whole catalog.
