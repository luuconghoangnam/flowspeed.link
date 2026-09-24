---
name: security-auditor
description: >
  Use when reviewing code for security issues, running security scans,
  checking OWASP Top 10, analyzing vulnerabilities, pentesting web endpoints,
  checking authentication/authorization flaws, or generating security reports.
  Triggers: "security review", "audit", "pentest", "vulnerability", "OWASP",
  "injection", "XSS", "CSRF", "auth bypass", "is this secure?".
  Source: strix (autonomous AI pentest agent).
---

# Security Auditor Skill

> ⚡ **TỐI ƯU TOKEN (TOKEN OPTIMIZATION)**:
> Không tự động nạp các file trong `references/` vào bộ nhớ lúc ban đầu.
> Chỉ mở từng file tương ứng khi người dùng yêu cầu audit mảng cụ thể:
> - Checklist OWASP Top 10 chi tiết: Tra cứu [owasp-top-10-testing.md](references/owasp-top-10-testing.md)
> - Kiểm tra bảo mật API: Tra cứu [api-security-testing.md](references/api-security-testing.md)
> - Pentest ứng dụng web: Tra cứu [web-app-penetration-testing.md](references/web-app-penetration-testing.md)
> - Quét lỗ hổng trong code: Tra cứu [code-vulnerabilities.md](references/code-vulnerabilities.md)

## Quick Scan with Strix

Strix is an autonomous AI pentest agent. Run it against a target:

```bash
# Install (one-time)
pip install strix-agent

# Quick scan
strix scan --target http://localhost:3000 --checks owasp-top10

# Deep scan with report
strix scan --target http://localhost:3000 --checks all --output report.md

# Specific check
strix scan --target http://localhost:3000 --checks injection,auth,xss
```

---

## Manual Security Audit Checklist

### A01 — Broken Access Control
- [ ] All API endpoints check authorization, not just authentication
- [ ] IDOR: Can user A access user B's resources by changing IDs?
- [ ] Admin routes protected beyond just login check
- [ ] File downloads validate ownership

### A02 — Cryptographic Failures
- [ ] Passwords hashed with bcrypt/argon2 (cost factor ≥ 12)
- [ ] Sensitive data encrypted at rest (AES-256)
- [ ] TLS enforced everywhere, no HTTP fallback
- [ ] JWT secrets are strong random values, not "secret123"

### A03 — Injection
- [ ] All DB queries use parameterized statements / ORM
- [ ] No eval(), exec() with user input
- [ ] Command injection: no shell exec with user data
- [ ] NoSQL injection: validate MongoDB/Redis query operators

### A05 — Security Misconfiguration
- [ ] Debug mode OFF in production
- [ ] Default credentials changed
- [ ] Error messages don't leak stack traces to client
- [ ] Security headers set (CSP, HSTS, X-Frame-Options)
- [ ] CORS configured precisely, not `*`

### A07 — Auth & Session
- [ ] Session tokens: cryptographically random, ≥ 128 bits
- [ ] Session invalidated on logout
- [ ] Brute-force protection on login (rate limit + lockout)
- [ ] Password reset tokens: single-use, expiring (15 min max)
- [ ] MFA available for sensitive operations

### A10 — SSRF
- [ ] Validate/allowlist URLs before server-side fetch
- [ ] Block internal IP ranges (10.x, 172.16.x, 192.168.x, 127.x, 169.254.x)

---

## Severity Classification

| Severity | Examples | Action |
|----------|---------|--------|
| 🔴 Critical | RCE, SQLi, Auth bypass, Leaked secrets | Fix before deploy |
| 🟠 High | XSS stored, IDOR, SSRF | Fix in current sprint |
| 🟡 Medium | CSRF, Info disclosure, Weak crypto | Fix in next sprint |
| 🟢 Low | Security headers missing, verbose errors | Track in backlog |
