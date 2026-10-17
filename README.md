# Master Agent Skills & Rules Repository

A curated, enterprise-grade repository uniting **139 production skills** and **universal operating rules** for AI coding assistants.

Consolidates:
1. **[Harness Skills](https://github.com/harness/harness-skills)**: 71 enterprise DevOps, CI/CD, FME, and governance skills.
2. **[Quicknode Blockchain Skills](https://github.com/quiknode-labs/blockchain-skills)**: Multi-chain Web3 infrastructure across 80+ blockchains.
3. **[Claude Skills](https://github.com/jeffallan/claude-skills)**: 67 expert fullstack, language, testing, security, and architectural skills.

---

## Key Features

- **Multi-Platform Support**: Ready out of the box for Google Antigravity, Claude Code, Cursor, Codex, and OpenAI/Anthropic tool protocols.
- **Unified Discovery Layout**:
  - `skills/<skill_name>/SKILL.md` (Standard Agent Skills spec & Claude Code)
  - `.agents/skills/` (Antigravity customization discovery)
- **Superior Merged Implementations**:
  - **`vue-expert`**: Merged modern TypeScript & Vanilla JavaScript + JSDoc modes into a single adaptive skill.
  - **`atlassian-mcp`**: Merged deep Jira issue lifecycles, JQL queries, and agile workflows with Confluence CQL and MCP setups.
  - **Cross-Linked Synergies**: Integrated general DevOps/Chaos/Security skills with Harness enterprise platform skills.
- **Universal Operating Rules (`AGENTS.md`)**:
  - **The 1% Rule**: Mandatory skill activation when any relevant workflow is detected.
  - **Verification Discipline**: Evidence-based completion claims with fresh execution proofs.
  - **Change Discipline**: Minimal blast radius and root cause resolution.
  - **Safety & Permissions**: Strict read-only defaults, zero private-key exposure, and explicit confirmation before destructive/financial actions.

---

## Structure

```text
├── .agents/
│   ├── rules/            # Modular rule definitions (activation, verification, safety)
│   └── skills/           # Discovery link for Antigravity
├── skills/               # 139 individual production skills
├── commands/             # Workflow commands (planning, execution, discovery)
├── references/           # Shared technical references
├── scripts/              # Validation and catalog scripts
├── AGENTS.md             # Universal agent instructions
├── CLAUDE.md             # Claude Code entrypoint
├── GEMINI.md             # Antigravity/Gemini entrypoint
└── skills.json           # Master catalog
```

---

## Getting Started

1. Open this repository in your AI coding assistant (Antigravity, Claude Code, Cursor).
2. The agent will automatically detect `AGENTS.md` (or `CLAUDE.md` / `GEMINI.md`) and discover all 139 skills.
3. Invoke any skill via slash-commands (e.g. `/react-expert`, `/quicknode-skill`, `/create-pipeline`, `/atlassian-mcp`) or let the agent auto-activate skills following the 1% Rule.
