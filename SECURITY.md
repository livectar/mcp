# Security policy

Please report suspected vulnerabilities privately to the repository
maintainers before opening a public issue. Do not include provider tokens,
OAuth secrets, phone codes, 2FA passwords, or session material in reports.

Credentials are host-injected and must never be placed in MCP tool arguments,
schemas, model prompts, audit payloads, logs, or routing configuration.
