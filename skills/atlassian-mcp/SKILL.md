---
name: atlassian-mcp
description: Comprehensive enterprise integration with Atlassian Jira and Confluence via MCP protocol. Manages Jira issues, projects, agile boards, sprint workflows, and custom fields, alongside Confluence spaces and pages. Use when asked to create, search, update, transition, or batch-manage Jira tickets, query with JQL or CQL, discover custom fields, manage sprints, backlogs, and epics, configure Atlassian MCP authentication (OAuth 2.1 or API tokens), or sync documentation between Confluence and codebase.
license: MIT
metadata:
  author: https://github.com/Jeffallan
  company: https://synergetic.solutions
  version: "2.0.0"
  domain: platform
  triggers: Jira, Confluence, Atlassian, MCP, tickets, issues, wiki, JQL, CQL, sprint, backlog, project management, create Jira ticket, search Jira, update Jira issue, transition issue, sprint planning, epic management, custom field discovery
  role: master-expert
  scope: implementation-and-management
  output-format: code-and-reports
  related-skills: mcp-developer, api-designer, security-reviewer, devops-engineer
---

# Atlassian & Jira Master Suite (MCP Edition)

Comprehensive Claude & Antigravity skill for enterprise Jira and Confluence management using Atlassian Model Context Protocol (MCP) servers.

---

## When to Use This Skill

- Creating and validating Jira issues with proper project keys and required fields.
- Querying Jira issues with optimized JQL filters and pagination.
- Managing issue lifecycle transitions (e.g., In Progress, Review, Done).
- Managing Agile/Scrum operations: boards, sprints, backlogs, epics, and story points.
- Discovering and handling Jira custom fields dynamically.
- Searching, authoring, and updating Confluence pages and spaces with CQL.
- Setting up and securing Atlassian MCP servers (OAuth 2.1, API tokens, PATs).
- Syncing specifications, meeting notes, and architecture docs to Jira/Confluence.

---

## Core Workflows

### 1. Jira Issue Creation Workflow

When creating Jira issues, always follow this strict sequence:

1. **Gather Requirements**:
   - Project key (e.g. `PROJ`, `DEV`, `OPS`) — **NEVER ASSUME**.
   - Issue type (`Story`, `Task`, `Bug`, `Epic`, `Subtask`).
   - Summary and descriptive Markdown body.
   - Assignee, priority, labels, and target sprint/epic.
2. **Validate Project & Fields**:
   - Verify project key exists.
   - For custom fields (story points, acceptance criteria, epic links), consult `references/custom_field_discovery.md` or run a discovery probe.
3. **Execute Creation with Confirmation**:
   - Confirm payload with user before modifying production boards.
   - Create issue with structured fields.
4. **Follow-up Links & Attachments**:
   - Link subtasks to parent or stories to epics.
   - Add initial implementation plan as a comment if requested.

### 2. Issue Search & Querying (JQL)

- Always test queries with `maxResults=1` or `limit: 1` first to validate syntax before executing large fetches.
- Consult `references/jql_guide.md` for exhaustive syntax rules.
- Common patterns:
  ```jql
  # Active sprint work for current user
  project = PROJ AND sprint in openSprints() AND assignee = currentUser() ORDER BY priority DESC

  # Unresolved high-priority bugs
  project = PROJ AND issuetype = Bug AND priority in (High, Highest, Critical) AND status != Done
  ```

### 3. Workflow & Status Transitions

1. Retrieve available transitions for current issue key to determine valid target states.
2. Provide transition ID, resolution status (e.g. `Fixed`, `Won't Fix`), and explanatory comment.

### 4. Confluence Documentation Operations (CQL)

1. Use CQL to locate parent spaces or existing documentation:
   ```cql
   space = "ENG" AND type = page AND title ~ "Architecture Decision"
   ```
2. Create or update pages with clear hierarchy, macros, and version notes.

---

## Reference Guide

| Topic | Reference | Load When |
|---|---|---|
| Custom Field Discovery | `references/custom_field_discovery.md` | Finding custom field names, IDs, schema structures |
| Exhaustive JQL Guide | `references/jql_guide.md` | Advanced JQL syntax, operators, functions, performance |
| Jira Queries & CRUD | `references/jira-queries.md` | Issue creation, editing, linking, transitions |
| Confluence Operations | `references/confluence-operations.md` | CQL search, page creation, space management |
| Authentication & Scopes | `references/authentication-patterns.md` | OAuth 2.0, API tokens, least-privilege scopes |
| MCP Server Setup | `references/mcp-server-setup.md` | Server setup, Docker/npx configs, connection debugging |
| Common Workflows | `references/common-workflows.md` | Sprint automation, triage, doc sync |

---

## MCP Server Configuration

Recommended MCP configuration for local and cloud instances:

```json
{
  "mcpServers": {
    "atlassian": {
      "command": "npx",
      "args": ["-y", "@sooperset/mcp-atlassian"],
      "env": {
        "JIRA_URL": "https://your-domain.atlassian.net",
        "JIRA_EMAIL": "user@example.com",
        "JIRA_API_TOKEN": "${JIRA_API_TOKEN}",
        "CONFLUENCE_URL": "https://your-domain.atlassian.net/wiki",
        "CONFLUENCE_EMAIL": "user@example.com",
        "CONFLUENCE_API_TOKEN": "${CONFLUENCE_API_TOKEN}"
      }
    }
  }
}
```

---

## Safety & Operating Guardrails

### MUST DO
- Always validate project keys and required fields prior to creation.
- Test unfamiliar JQL queries with `maxResults=1` probe before full query.
- Ask for user confirmation before bulk edits, ticket deletion, or status transitions.
- Store `JIRA_API_TOKEN` and credentials exclusively in environment variables.
- Handle pagination (standard chunks of 20-50 items) for large backlogs.

### MUST NOT DO
- Never hardcode API tokens or credentials in files or tool calls.
- Never make assumptions about project keys (e.g. guessing `PROJ` without checking).
- Never execute destructive bulk deletions without explicit user confirmation.
- Never expose sensitive ticket contents or credentials in public logs.
