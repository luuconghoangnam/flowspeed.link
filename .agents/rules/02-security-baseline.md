# Security Baseline — Always-On Rule
# Distilled from: strix (OWASP Top 10 AI pentest agent)
# Token budget: ~200 tokens | Always injected into every conversation

---

## Non-negotiable Security Rules (apply to ALL code)

### Input & Auth
- NEVER trust user input without validation/sanitization
- NEVER store passwords in plaintext — bcrypt/argon2 minimum
- NEVER put secrets/API keys in code or git — use env vars / secret manager
- ALWAYS use parameterized queries — never string-concatenate SQL

### API & HTTP
- ALWAYS validate and sanitize request body, query params, headers
- ALWAYS set security headers: CSP, HSTS, X-Frame-Options, X-Content-Type-Options
- ALWAYS rate-limit authentication endpoints
- NEVER expose stack traces or internal errors to client

### File & Data
- NEVER allow arbitrary file path from user input (path traversal)
- ALWAYS validate file types server-side, not just by extension
- ALWAYS encrypt sensitive data at rest

### OWASP Top 10 Quick Check
Before finalizing any feature, mentally check:
- [ ] Injection (SQL, NoSQL, Command, LDAP)
- [ ] Broken Auth / Session management
- [ ] Sensitive data exposure
- [ ] Insecure Direct Object Reference (IDOR)
- [ ] Security misconfiguration
- [ ] XSS (reflected, stored, DOM)
- [ ] CSRF on state-changing endpoints
- [ ] Using components with known vulnerabilities

> For deep security audit, use: `.agents/skills/security-auditor/SKILL.md`
