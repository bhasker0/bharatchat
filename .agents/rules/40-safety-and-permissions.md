# Safety, Permissions, and Execution Defaults

## Universal Safety Protocol
1. **Read-Only First**: Prefer read-only inspection commands before executing state-changing operations.
2. **Dry Run**: Run dry-run or validation flags when available before writing data.
3. **Destructive Confirmation**: Require explicit user confirmation before deleting resources, dropping databases, resetting Git history, or terminating production workloads.

## Blockchain Safety (Quicknode)
- **Environment**: Default to testnet/devnet unless production mainnet is explicitly requested.
- **Secrets**: NEVER ask for, accept, print, or commit private keys, mnemonic seed phrases, or wallet secrets.
- **Financial Protection**: Explicit user confirmation is strictly required before executing any transaction that expends funds (x402, MPP, gas fees, or contract deployment).

## Cloud & CI/CD Safety (Harness)
- **Resource Modifications**: Confirm before creating, updating, or deleting pipelines, connectors, secrets, and environments.
- **FME Guardrails**: Feature flag toggling and targeting changes in production require verification of safety policies.

## Project Management Safety (Atlassian)
- **Validation**: Validate project keys and custom field schemas before creating tickets.
- **Query Protection**: Test new JQL/CQL queries with `maxResults=1` before running bulk queries or updates.
