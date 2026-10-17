# Universal Agent Operating Guidelines

**Scope**: This repository contains the unified agent configuration and complete skills library (139 production skills across Blockchain/Web3, Full-Stack Engineering, Cloud/DevOps, Testing, Security, and Harness Enterprise Automation). This file configures agents operating across Antigravity, Claude Code, Cursor, Codex, and other AI pair-programming platforms.

---

## 1. Core Operating Doctrine

### 1.1 Skill Activation: The 1% Rule
If there is even a **1% chance** an available skill applies to what you are doing, you **ABSOLUTELY MUST** read the skill.
This is non-negotiable. Reject rationalizations such as:
- *"This is just a simple question."*
- *"I remember what this skill says."*
- *"This seems like overkill."*
- *"I need more context first."*

When these rationalizations arise, pause immediately, identify which skill applies, and load it.

### 1.2 Subagent Delegation Discipline
**The 1% Rule does not survive delegation unless the brief carries it explicitly.**
When dispatching any subagent:
1. Review the catalog below for relevant skills.
2. Formulate instructions referencing the exact skill name: `Use the <exact skill name> skill to <task>.`
3. Instruct the subagent to report the loaded skills in its final response.

### 1.3 Verification Discipline: Evidence-Based Proof
**NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE.**
Before asserting that any task, bug fix, or refactor is complete:
1. **Identify**: Determine the exact test, command, or log check that proves the claim.
2. **Execute**: Run it fresh (never rely on past memory).
3. **Examine**: Inspect exit codes, stderr, stdout, and error traces.
4. **Confirm**: Validate that outputs satisfy the requirement without unwanted regressions.
5. **State**: Present the conclusion alongside the fresh evidence.

### 1.4 Change Discipline: Surgical Interventions
1. **Surgical Scope**: Modify only what is necessary to achieve the goal. Avoid gratuitous refactorings or stylistic overrides.
2. **Root Cause Resolution**: Solve underlying bugs directly; do not apply defensive duct tape.
3. **Preserve Context**: Maintain documentation, comments, and type signatures unless explicitly modified.
4. **Surface Assumptions**: Explicitly state operational assumptions when dealing with ambiguous APIs or constraints.

---

## 2. Safety, Permissions, and Execution Defaults

### 2.1 Universal Guardrails
- **Read-Only Inspection First**: Run dry-runs and non-mutating inspections before modifying state.
- **Explicit Confirmation for Destructive Actions**: Never delete files, tables, branches, or production workloads without explicit confirmation.

### 2.2 Blockchain & Web3 Safety (Quicknode)
- **Environment**: Default to testnet/devnet environments unless mainnet is explicitly requested.
- **Private Keys**: NEVER request, accept, log, or commit private keys, mnemonic phrases, or wallet secrets.
- **Financial Actions**: Require explicit confirmation before deploying contracts, provisioning paid streams, or expending funds via x402 / MPP.

### 2.3 Cloud & CI/CD Safety (Harness)
- **Resource Protection**: Require user confirmation prior to creating, modifying, or deleting pipelines, secrets, connectors, or environments.
- **FME Guardrails**: Feature flag rollout changes must follow write-safety protocols and verification steps.

### 2.4 Project Management Safety (Atlassian / Jira)
- **Validation**: Verify project keys and custom field schemas before ticket creation.
- **Query Safeguard**: Probe new JQL and CQL queries with `maxResults=1` before executing bulk fetches or batch operations.

---

## 3. Communication & Style Guidelines

- **High-Signal Technical Output**: Be direct, factual, and concise. Avoid AI conversational fluff and apologies.
- **Clickable File Links**: Reference files and symbols using clickable markdown links (`[filename.ext](file:///path/to/filename.ext)`).
- **GitHub Alert Callouts**: Use `> [!NOTE]`, `> [!TIP]`, `> [!IMPORTANT]`, `> [!WARNING]`, and `> [!CAUTION]` to highlight key information.

---

## 4. Master Skills Directory (139 Skills)

All skills reside under `skills/<skill-name>/SKILL.md` (and are mirrored in `.agents/skills/` for Antigravity discovery).

### Blockchain & Web3

| Skill | Description |
|---|---|
| `/quicknode-skill` | Quicknode blockchain infrastructure for 80+ chains: Core RPC API, Streams, Webhooks, Blazar WSS, Solana gRPC, HyperCore gRPC, S... |

### Architecture, Systems & Protocols

| Skill | Description |
|---|---|
| `/architecture-designer` | Use when designing new high-level system architecture, reviewing existing designs, or making architectural decisions. Invoke to... |
| `/microservices-architect` | Designs distributed system architectures, decomposes monoliths into bounded-context services, recommends communication patterns... |
| `/api-designer` | Use when designing REST or GraphQL APIs, creating OpenAPI specifications, or planning API architecture. Invoke for resource mod... |
| `/graphql-architect` | Use when designing GraphQL schemas, implementing Apollo Federation, or building real-time subscriptions. Invoke for schema desi... |
| `/websocket-engineer` | Use when building real-time communication systems with WebSockets or Socket.IO. Invoke for bidirectional messaging, horizontal ... |
| `/mcp-developer` | Use when building, debugging, or extending MCP servers or clients that connect AI systems with external tools and data sources.... |
| `/atlassian-mcp` | Comprehensive enterprise integration with Atlassian Jira and Confluence via MCP protocol. Manages Jira issues, projects, agile ... |
| `/fullstack-guardian` | Builds security-focused full-stack web applications by implementing integrated frontend and backend components with layered sec... |
| `/the-fool` | Use when challenging ideas, plans, decisions, or proposals using structured critical reasoning. Invoke to play devil's advocate... |
| `/legacy-modernizer` | Designs incremental migration strategies, identifies service boundaries, produces dependency maps and migration roadmaps, and g... |
| `/feature-forge` | Conducts structured requirements workshops to produce feature specifications, user stories, EARS-format functional requirements... |
| `/spec-miner` | Reverse-engineering specialist that extracts specifications from existing codebases. Use when working with legacy or undocument... |
| `/code-documenter` | Generates, formats, and validates technical documentation — including docstrings, OpenAPI/Swagger specs, JSDoc annotations, doc... |
| `/cli-developer` | Use when building CLI tools, implementing argument parsing, or adding interactive prompts. Invoke for parsing flags and subcomm... |
| `/embedded-systems` | Use when developing firmware for microcontrollers, implementing RTOS applications, or optimizing power consumption. Invoke for ... |
| `/game-developer` | Use when building game systems, implementing Unity/Unreal Engine features, or optimizing game performance. Invoke to implement ... |

### Frontend Engineering

| Skill | Description |
|---|---|
| `/react-expert` | Use when building React 18+ applications in .jsx or .tsx files, Next.js App Router projects, or create-react-app setups. Create... |
| `/nextjs-developer` | Use when building Next.js 14+ applications with App Router, server components, or server actions. Invoke to configure route han... |
| `/vue-expert` | Master Vue 3 engineering skill supporting both TypeScript and Vanilla JavaScript with JSDoc typing. Builds Vue 3 components wit... |
| `/vue-expert-js` | Creates Vue 3 components, builds vanilla JS composables, configures Vite projects, and sets up routing and state management usi... |
| `/angular-architect` | Generates Angular 17+ standalone components, configures advanced routing with lazy loading and guards, implements NgRx state ma... |
| `/flutter-expert` | Use when building cross-platform applications with Flutter 3+ and Dart. Invoke for widget development, Riverpod/Bloc state mana... |
| `/react-native-expert` | Builds, optimizes, and debugs cross-platform mobile applications with React Native and Expo. Implements navigation hierarchies ... |
| `/shopify-expert` | Builds and debugs Shopify themes (.liquid files, theme.json, sections), develops custom Shopify apps (shopify.app.toml, OAuth, ... |
| `/wordpress-pro` | Develops custom WordPress themes and plugins, creates and registers Gutenberg blocks and block patterns, configures WooCommerce... |

### Backend Languages & Frameworks

| Skill | Description |
|---|---|
| `/python-pro` | Use when building Python 3.11+ applications requiring type safety, async programming, or robust error handling. Generates type-... |
| `/fastapi-expert` | Use when building high-performance async Python APIs with FastAPI and Pydantic V2. Invoke to create REST endpoints, define Pyda... |
| `/django-expert` | Use when building Django web applications or REST APIs with Django REST Framework. Invoke when working with settings.py, models... |
| `/django-storages-s3` | Use when configuring Django to store static and media files on AWS S3 with django-storages. Invoke when working with the STORAG... |
| `/golang-pro` | Implements concurrent Go patterns using goroutines and channels, designs and builds microservices with gRPC or REST, optimizes ... |
| `/rust-engineer` | Writes, reviews, and debugs idiomatic Rust code with memory safety and zero-cost abstractions. Implements ownership patterns, m... |
| `/java-architect` | Use when building, configuring, or debugging enterprise Java applications with Spring Boot 3.x, microservices, or reactive prog... |
| `/spring-boot-engineer` | Generates Spring Boot 3.x configurations, creates REST controllers, implements Spring Security 6 authentication flows, sets up ... |
| `/dotnet-core-expert` | Use when building .NET 8 applications with minimal APIs, clean architecture, or cloud-native microservices. Invoke for Entity F... |
| `/csharp-developer` | Use when building C# applications with .NET 8+, ASP.NET Core APIs, or Blazor web apps. Builds REST APIs using minimal or contro... |
| `/cpp-pro` | Writes, optimizes, and debugs C++ applications using modern C++20/23 features, template metaprogramming, and high-performance s... |
| `/php-pro` | Use when building PHP applications with modern PHP 8.3+ features, Laravel, or Symfony frameworks. Invokes strict typing, PHPSta... |
| `/laravel-specialist` | Build and configure Laravel 10+ applications, including creating Eloquent models and relationships, implementing Sanctum authen... |
| `/rails-expert` | Rails 7+ specialist that optimizes Active Record queries with includes/eager_load, implements Turbo Frames and Turbo Streams fo... |
| `/kotlin-specialist` | Provides idiomatic Kotlin implementation patterns including coroutine concurrency, Flow stream handling, multiplatform architec... |
| `/swift-expert` | Builds iOS/macOS/watchOS/tvOS applications, implements SwiftUI views and state management, designs protocol-oriented architectu... |
| `/typescript-pro` | Implements advanced TypeScript type systems, creates custom type guards, utility types, and branded types, and configures tRPC ... |
| `/javascript-pro` | Writes, debugs, and refactors JavaScript code using modern ES2023+ features, async/await patterns, ESM module systems, and Node... |
| `/nestjs-expert` | Creates and configures NestJS modules, controllers, services, DTOs, guards, and interceptors for enterprise-grade TypeScript ba... |

### Databases & Data Engineering

| Skill | Description |
|---|---|
| `/sql-pro` | Optimizes SQL queries, designs database schemas, and troubleshoots performance issues. Use when a user asks why their query is ... |
| `/postgres-pro` | Use when optimizing PostgreSQL queries, configuring replication, or implementing advanced database features. Invoke for EXPLAIN... |
| `/database-optimizer` | Optimizes database queries and improves performance across PostgreSQL and MySQL systems. Use when investigating slow queries, a... |
| `/spark-engineer` | Use when writing Spark jobs, debugging performance issues, or configuring cluster settings for Apache Spark applications, distr... |
| `/pandas-pro` | Performs pandas DataFrame operations for data analysis, manipulation, and transformation. Use when working with pandas DataFram... |
| `/dbops-changeset` | Generates Liquibase database changesets via a structured workflow: resolve schema+instance (from context or via API discovery),... |

### AI, Machine Learning & Prompting

| Skill | Description |
|---|---|
| `/rag-architect` | Designs and implements production-grade RAG systems by chunking documents, generating embeddings, configuring vector stores, bu... |
| `/prompt-engineer` | Writes, refactors, and evaluates prompts for LLMs — generating optimized prompt templates, structured output schemas, evaluatio... |
| `/ml-pipeline` | Designs and implements production-grade ML pipeline infrastructure: configures experiment tracking with MLflow or Weights & Bia... |
| `/fine-tuning-expert` | Use when fine-tuning LLMs, training custom models, or adapting foundation models for specific tasks. Invoke for configuring LoR... |

### DevOps, Cloud, SRE & Observability

| Skill | Description |
|---|---|
| `/cloud-architect` | Designs cloud architectures, creates migration plans, generates cost optimization recommendations, and produces disaster recove... |
| `/devops-engineer` | Creates Dockerfiles, configures CI/CD pipelines, writes Kubernetes manifests, and generates Terraform/Pulumi infrastructure tem... |
| `/kubernetes-specialist` | Use when deploying or managing Kubernetes workloads. Invoke to create deployment manifests, configure pod security policies, se... |
| `/terraform-engineer` | Use when implementing infrastructure as code with Terraform across AWS, Azure, or GCP. Invoke for module development (create re... |
| `/sre-engineer` | Defines service level objectives, creates error budget policies, designs incident response procedures, develops capacity models... |
| `/monitoring-expert` | Configures monitoring systems, implements structured logging pipelines, creates Prometheus/Grafana dashboards, defines alerting... |

### Quality Assurance, Testing, Security & Chaos

| Skill | Description |
|---|---|
| `/code-reviewer` | Analyzes code diffs and files to identify bugs, security vulnerabilities (SQL injection, XSS, insecure deserialization), code s... |
| `/security-reviewer` | Identifies security vulnerabilities, generates structured audit reports with severity ratings, and provides actionable remediat... |
| `/secure-code-guardian` | Use when implementing authentication/authorization, securing user input, or preventing OWASP Top 10 vulnerabilities — including... |
| `/test-master` | Generates test files, creates mocking strategies, analyzes code coverage, designs test architectures, and produces test plans a... |
| `/debugging-wizard` | Parses error messages, traces execution flow through stack traces, correlates log entries to identify failure points, and appli... |
| `/playwright-expert` | Use when writing E2E tests with Playwright, setting up test infrastructure, or debugging flaky browser tests. Invoke to write t... |
| `/chaos-engineer` | Designs chaos experiments, creates failure injection frameworks, and facilitates game day exercises for distributed systems — p... |

### Harness Pipelines & CI/CD

| Skill | Description |
|---|---|
| `/create-pipeline` | Generate Harness v0 Pipeline YAML for CI/CD workflows and create them via MCP. Supports CI stages (build, test, Docker push), C... |
| `/create-pipeline-v1` | Generate Harness v1 simplified Pipeline YAML using the new concise syntax with flat structure, ${{ }} expressions, script field... |
| `/create-template` | Generate Harness v0 Template YAML for reusable pipeline components (Step, Stage, Pipeline, StepGroup) and create them via MCP. ... |
| `/create-trigger` | Generate Harness Trigger YAML for automated pipeline execution and create via MCP. Use when user says "create trigger", "webhoo... |
| `/debug-pipeline` | Diagnose Harness pipeline executions via MCP. Analyzes any execution (failed or successful) to produce structured reports with ... |
| `/manage-freeze-windows` | Manage Harness deployment freeze windows via MCP. Create, list, get, update, and delete project-level freeze windows; toggle th... |
| `/migrate-pipeline` | Migrate Harness pipelines from v0 to v1 simplified format via MCP tools. Converts expressions from angle-bracket to ${{ }} synt... |
| `/run-pipeline` | Execute and monitor Harness pipeline runs via MCP tools. Find pipelines, provide runtime inputs, trigger executions, monitor pr... |

### Harness Infrastructure & Access Control

| Skill | Description |
|---|---|
| `/create-connector` | Generate Harness Connector YAML for integrations and create/test via MCP. Use when user says "create connector", "git connector... |
| `/create-environment` | Generate Harness Environment YAML for deployment targets and create via MCP. Supports PreProduction and Production types with e... |
| `/create-infrastructure` | Generate Harness Infrastructure Definition YAML for deployment targets and create via MCP. Use when user says "create infrastru... |
| `/create-secret` | Generate Harness Secret definitions and manage secrets via MCP v2 tools. Supports SecretText, SecretFile, SSHKey, and WinRmCred... |
| `/create-service` | Generate Harness Service YAML for deployable workloads and create via MCP. Supports Kubernetes, Helm, ECS, Serverless, SSH, and... |
| `/manage-roles` | Manage Harness RBAC roles, role assignments, permissions, and resource groups via MCP v2 tools. List, create, update, and delet... |
| `/manage-users` | Manage Harness users, user groups, and service accounts via MCP. List and search users, create and manage user groups for team-... |

### Harness Feature Flags & Experimentation (FME)

| Skill | Description |
|---|---|
| `/choose-metric` | Recommend primary/guardrail metrics for an experiment or rollout, judged against a good-metric checklist. Checks existing metri... |
| `/cleanup-feature-flags` | Remove a launched Harness FME feature flag from application code, keeping the treatment FME serves today, and open a pull reque... |
| `/create-experiment` | Thin compatibility entrypoint for creating a new Harness FME experiment. Invokes manage-experiments' Create experiment flow dir... |
| `/create-feature-flag` | Create a new Harness FME feature flag and its per-environment definitions with a safe default, following the project's naming a... |
| `/create-metric` | Create a Harness FME metric: helps decide what to measure, suggests candidate metrics from application code, resolves traffic t... |
| `/discover-feature-flags` | Audit Harness FME feature flags across a project. Read-only; never writes. Four modes: inventory (list flags with metadata), ro... |
| `/explain-flag` | Explain a single FME feature flag in plain language: purpose, default rule, and per-environment targeting state, flagging incon... |
| `/feature-flag-onboarding` | Orchestrate first-time Harness FME onboarding for one application. Confirm an already-enabled FME account/project, then pick on... |
| `/fme-pipeline` | Generate Harness pipelines with FeatureFlag or Custom stages for flag rollout scenarios: progressive ramp, multi-env promotion,... |
| `/instrument-feature-flag` | Gate application code behind an existing Harness FME feature flag. Detect the app's language, framework and any existing flag w... |
| `/instrument-metric` | Wire up a track() call for a metric's event, placed at the real outcome, and verify it arrives. Doesn't create the metric (crea... |
| `/manage-experiments` | Create, update (metrics, dates, description, hypothesis, baseline/comparison treatments, status), or delete Harness FME feature... |
| `/manage-flag-lifecycle` | Manage metadata and lifecycle for a Harness FME feature flag: update description, tags, owners, rollout status; assess archive ... |
| `/manage-segments` | Create, inspect, and maintain Harness FME segments (STANDARD, LARGE, RULE_BASED). Manage metadata, select type-specific definit... |
| `/review-experiment-results` | Explain Harness FME experiment results: winner determination, statistical significance, guardrail metric impact, and data-quali... |
| `/update-flag-targeting` | Change how a Harness FME feature flag serves traffic in environments. Covers percentage rollout, individual targets, targeting ... |

### Harness Observability, Governance & AI Operations

| Skill | Description |
|---|---|
| `/ai-operations` | Configure Harness AI-powered operations (AIDA) via MCP. Set up predictive failure analysis with ML models for memory leaks, dis... |
| `/analyze-costs` | Analyze cloud costs, find optimization opportunities, and track anomalies using Harness CCM via MCP. Use when user says "cloud ... |
| `/audit-report` | Generate audit reports and compliance trails using Harness audit trail data via MCP v2 tools. Track user actions, resource chan... |
| `/chaos-dr-test` | Use when working with Chaos Engineering steps inside a Harness pipeline. Covers ChaosFault, ChaosProbe, ChaosAction steps (DRTe... |
| `/chaos-experiment` | Use when the user asks to create, edit, update, design, or configure a Harness Chaos Experiment — including faults, probes, act... |
| `/create-policy` | Create OPA governance policies for Harness via MCP. Define policies that enforce compliance rules on pipelines, services, envir... |
| `/deployment-readiness` | Assess deployment readiness via Harness MCP. Run pre-deployment readiness checks with go/no-go recommendations, analyze environ... |
| `/dora-metrics` | Generate DORA metrics and engineering performance reports using Harness SEI via MCP. Track deployment frequency, lead time, cha... |
| `/gitops-status` | Monitor GitOps application health, sync status, and manage ArgoCD deployments via Harness MCP. Use when user says "gitops statu... |
| `/incident-response` | Incident response and analysis via Harness MCP. Correlate incidents with recent deployments, assess blast radius and downstream... |
| `/manage-idp` | Manage Harness Internal Developer Portal (IDP) resources via MCP. Create service catalog templates, configure self-service envi... |
| `/manage-slos` | Assist with Harness Service Reliability Management (SRM) tasks that the MCP server currently supports: pulling recent deploymen... |
| `/optimize-pipeline` | Optimize Harness CI/CD pipeline performance via MCP. Configure parallel test execution with Test Intelligence, design multi-lay... |
| `/pr-analysis` | Analyze pull request impact on Harness pipelines via MCP. Assess PR pipeline impact by identifying which pipelines, services, a... |
| `/scorecard-review` | Review service maturity scorecards and compliance using Harness IDP via MCP. Check production readiness, security, and document... |
| `/security-report` | Generate security compliance reports using Harness SCS and STO via MCP. Analyze vulnerabilities, SBOMs, and manage exemptions. ... |
| `/template-usage` | Track Harness template dependencies, usage, and adoption via MCP v2 tools. Find which pipelines or other templates reference a ... |

### Harness Security, Supply Chain & Platform

| Skill | Description |
|---|---|
| `/approve-exempt` | Approve pending Harness STO security exemptions (waivers) at their current scope or elevate them to Project, Org, or Account sc... |
| `/configure-agent-pr-attestation` | Use when asked to set up PR attestation, install harness-scs attestation hook, configure agent attestor, troubleshoot why agent... |
| `/configure-container-scan` | Add container image scanning steps to existing Harness pipelines using Harness STO scanners. Supports Harness SCA (default), Aq... |
| `/configure-dast-scan` | Add Dynamic Application Security Testing (DAST) steps to existing Harness pipelines using Harness STO scanners. Supports API DA... |
| `/configure-repo-scan` | Configure code scanning in Harness pipelines using STO security scanners. Helps identify where to inject SAST/SCA scanning step... |
| `/configure-secret-scan` | Add secret detection scanning steps to existing Harness pipelines using STO security scanners. Detects exposed credentials, API... |
| `/enforce-slsa` | Add an SLSA Verification (SlsaVerification) step to an existing Harness pipeline to verify SLSA provenance attestations and opt... |
| `/exempt-vuln` | Create Harness STO security exemptions (waivers) for vulnerabilities found by SAST, SCA, DAST, secret, container, or IaC scanne... |
| `/generate-slsa` | Add a SLSA Generation step (YAML type: provenance) to an existing Harness pipeline to generate SLSA provenance and optionally a... |
| `/sign-artifact` | Add an Artifact Signing (SscaArtifactSigning) step to an existing Harness pipeline to Cosign-sign container or local-stage arti... |
| `/verify-sign` | Add an Artifact Verification (SscaArtifactVerification) step to an existing Harness pipeline to verify Cosign signatures on con... |
| `/create-sbom` | Add an SBOM (Software Bill of Materials) generation step to an existing Harness pipeline using Harness SCS SscaOrchestration (S... |
| `/enforce-sbom` | Add an SBOM Policy Enforcement (SscaEnforcement / CdSscaEnforcement) step to an existing Harness pipeline to verify SBOM attest... |
| `/manage-artifacts` | Manage Harness Artifact Registry (AR) via MCP. Configure private registries for Docker, Helm, Maven, npm, and PyPI artifacts, s... |
| `/manage-cde` | Manage Harness Cloud Development Environments (CDE) via MCP. Configure on-demand development environments with pre-installed to... |
| `/manage-delegates` | Monitor and manage Harness Delegates via MCP. List delegates and check health status, manage delegate registration tokens (crea... |
| `/manage-iacm` | Manage Harness Infrastructure as Code Management (IaCM) via MCP. Configure Terraform workspaces with remote state and RBAC, set... |
| `/manage-pull-requests` | Manage Harness Code pull requests via MCP. List, create, update, and merge PRs; add reviewers and submit review decisions (appr... |
| `/manage-supply-chain` | Manage Harness Software Supply Chain Assurance (SSCA) via MCP. Configure automated SBOM generation with CycloneDX or SPDX forma... |

### Harness AI Agents

| Skill | Description |
|---|---|
| `/create-agent` | Create and update Harness AI agent instances - standalone templates for agentic workflows in pipelines. Use when asked to creat... |
| `/create-agent-template` | Generate Harness Agent Template files for AI-powered automation agents. Produces metadata.json, pipeline.yaml (v1 syntax), and ... |
