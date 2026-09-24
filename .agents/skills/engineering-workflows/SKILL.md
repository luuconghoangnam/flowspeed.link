---
name: engineering-workflows
description: >
  Use for complex software engineering tasks: large-scale refactoring,
  API design, database schema design/migration, architecture decisions,
  design patterns selection, code review workflows, performance optimization.
  Triggers: "refactor", "redesign", "architecture", "API design", "schema",
  "migration", "design pattern", "performance", "optimize", "how to structure".
  Source: VoltAgent/awesome-agent-skills.
---

# Engineering Workflows Skill

## 1. Refactoring Workflow

```
Step 1: Understand before touching
  → Read ALL files the change touches. Trace end-to-end flow.
  → Identify all callers of functions you'll modify.

Step 2: Add tests FIRST (if none exist)
  → Write tests that capture current behavior.
  → They'll protect you during refactor.

Step 3: Refactor incrementally
  → One concern at a time. Commit after each green test run.
  → Never mix refactor + feature in same commit.

Step 4: Verify
  → Run full test suite. Check no callers broke.
  → Review diff: every changed line traces to goal.
```

---

## 2. API Design Checklist (REST)

```
Resource naming:  Nouns not verbs. Plural. /users, /orders, /products
HTTP methods:     GET (read), POST (create), PUT (replace), PATCH (update), DELETE
Status codes:     200 OK, 201 Created, 204 No Content, 400 Bad Request,
                  401 Unauthorized, 403 Forbidden, 404 Not Found, 422 Validation Error,
                  429 Rate Limited, 500 Internal Error
Versioning:       /api/v1/ prefix from day 1
Pagination:       cursor-based for large datasets, offset for small
Response envelope:
  { "data": {...}, "meta": { "page": 1, "total": 100 } }
  { "error": { "code": "VALIDATION_ERROR", "message": "...", "details": [...] } }
Auth:             Bearer token in Authorization header, never in URL
Rate limiting:    X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset headers
```

---

## 3. Database Schema Design Rules

```
- UUID vs Int PK: UUID for distributed/public APIs, Int for internal perf
- Always: created_at, updated_at timestamps on every table
- Soft delete: deleted_at nullable column (don't hard delete business data)
- Indexes: on foreign keys, frequently filtered/sorted columns
- Constraints: use DB-level constraints (NOT NULL, UNIQUE, FK) as last defense
- Migrations: additive first (add column), then migrate data, then drop old
- Never: store comma-separated values in a column (normalize it)
- Naming: snake_case tables/columns, singular table names (user not users in some ORMs)
```

---

## 4. Design Pattern Quick Selector

| Problem | Pattern | When |
|---------|---------|------|
| Object creation complex | Factory / Builder | Multiple configs, same interface |
| Behavior varies at runtime | Strategy | e.g., payment methods, sort algorithms |
| Add behavior without subclassing | Decorator | Logging, caching wrappers |
| Event broadcasting | Observer/EventEmitter | Loose coupling, 1-to-many |
| Expensive object creation | Singleton | DB connection pool, config |
| Simplify complex subsystem | Facade | Legacy code wrapping |
| Async pipeline steps | Chain of Responsibility | Middleware, request pipeline |

---

## 5. Performance Optimization Approach

```
Rule #1: MEASURE FIRST. Never optimize without profiling.
Rule #2: Optimize the bottleneck, not the prettiest code.

Common wins (in order of impact):
1. N+1 queries → add eager loading / joins
2. Missing DB indexes → EXPLAIN ANALYZE the slow query
3. No caching → Redis for repeated expensive reads
4. Synchronous I/O → make it async/parallel
5. Large payload → paginate, compress, lazy load
6. Re-renders → memoize, virtualize lists
```

---

## 6. Code Review Mental Model

Ask before approving:
- [ ] Does this solve the stated problem? (not more, not less)
- [ ] Are edge cases handled? (null, empty, concurrent)
- [ ] Is error handling meaningful to the caller?
- [ ] Is this testable? Are tests present?
- [ ] Will this scale to 10x load?
- [ ] Are secrets/PII handled correctly?
- [ ] Is the naming clear to someone new?
