# MCP Servers and Tool Integrations

## Standards
- **Credentials**: Never embed API tokens or secrets in source files or configuration files checked into source control. Always read credentials from environment variables.
- **Error Handling**: Gracefully handle network timeouts and rate limits with exponential backoff and jitter.
- **Scope Verification**: Confirm required tool scopes with a read-only probe before attempting write operations.
