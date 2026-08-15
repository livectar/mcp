# Compatibility policy

The initial MCP protocol revision is `2025-06-18`.

Changes to wire-level types, public traits, or required host services are
breaking changes and require a new minor release note. Additive tool fields
must remain optional when possible. Provider-specific behavior belongs in the
provider package and must not leak into the generic protocol contracts.
