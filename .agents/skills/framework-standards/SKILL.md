---
name: framework-standards
description: >
  Use when writing code that must follow a specific framework's conventions,
  file structure, naming patterns, or best practices. Triggers: framework name
  mentioned (Next.js, NestJS, FastAPI, Rust, Go, Django, Astro, Vue, Svelte),
  "how to structure", "folder layout", "naming convention", "best practice for X".
  Source: PatrickJS/awesome-cursorrules.
---

# Framework Standards Skill

## How to Find Framework-Specific Rules

The `awesome-cursorrules` repo has .mdc rule files for 100+ frameworks.
Look up the exact rules for any tech stack:

```powershell
# $skillsDir = thu muc skills nam cung cap voi Hub (hoac $env:SKILLS_PATH)
# List available frameworks
Get-ChildItem (Join-Path $skillsDir "awesome-cursorrules\rules") -Directory | Select-Object Name

# Read specific framework rules
Get-Content (Join-Path $skillsDir "awesome-cursorrules\rules\nextjs\*.mdc") -Raw
```

---

## Universal Conventions (Apply to ALL frameworks)

### File & Folder Naming
- **Components**: PascalCase → `UserCard.tsx`, `AuthModal.vue`
- **Utils / Hooks**: camelCase → `useAuth.ts`, `formatDate.ts`
- **Routes / Pages**: kebab-case → `user-profile.tsx`, `api/auth/callback`
- **Constants**: UPPER_SNAKE → `MAX_RETRY_COUNT`, `API_BASE_URL`
- **Tests**: co-locate or mirror structure → `UserCard.test.tsx`

### Code Conventions
- Exports: named exports preferred over default (easier refactoring)
- Async: async/await over raw Promises
- Error handling: always handle, never swallow silently
- Types: explicit return types on public functions
- Comments: explain WHY, not WHAT

---

## Quick Reference — Key Frameworks

### Next.js 14/15 (App Router)
- `app/` directory, `page.tsx`, `layout.tsx`, `loading.tsx`, `error.tsx`
- Server Components by default. Client: `'use client'` at top.
- Data: `fetch()` with `cache: 'force-cache'` | `revalidate: N` | `cache: 'no-store'`
- API Routes → `app/api/[route]/route.ts` with `GET`, `POST` exports
- Images → always `next/image`. Links → always `next/link`.

### NestJS
- Feature modules: `module`, `controller`, `service`, `dto`, `entity`
- DTOs: class-validator decorators for all inputs
- Guards for auth, Interceptors for logging/transform
- Never put business logic in Controllers

### FastAPI (Python)
- Pydantic models for all request/response schemas
- Dependency injection via `Depends()`
- Route prefix groups with `APIRouter`
- Async endpoints when doing I/O

### Go
- Package names: single word, lowercase
- Error handling: always check, `if err != nil { return err }`
- Interfaces: define where consumed, not where implemented
- No OOP inheritance — composition via embedding

### Rust
- `Result<T, E>` for fallible operations, `Option<T>` for optional
- `?` operator for error propagation
- Ownership: clone only when necessary
- `cargo clippy` before committing

---

## Find Deeper Rules On-Demand

```powershell
# Search for any framework you need
Get-ChildItem (Join-Path $skillsDir "awesome-cursorrules\rules") -Recurse -Filter "*.mdc" |
  Where-Object { $_.Name -like "*<framework>*" } |
  Select-Object FullName
```
