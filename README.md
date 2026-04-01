<div align="center">

# sys-design

**Architecture modeling as code. Validate. Generate. Ship.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 2021](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Lines of Code](https://img.shields.io/badge/LOC-16K+-blue.svg)]()

A CLI tool that reads YAML model files and generates [PlantUML](https://plantuml.com/) diagrams — with built-in validation rules for completeness, consistency, and naming conventions.

</div>

---

## Why sys-design?

- **Model as code** — Define your architecture in YAML, version it with Git, review it in PRs
- **8 diagram types** — Context (C4), Logic View, Runtime View, Concept Model, Code, Build, Delivery, Deployment
- **Validate before you draw** — 40+ built-in rules catch errors before they become diagrams
- **Full CRUD** — Add, remove, list, and show elements directly from the terminal
- **PlantUML output** — Generate ready-to-render `.puml` files or pipe to stdout

## Quick Start

```bash
# Clone and build
git clone https://github.com/user/sys-design-cli.git
cd sys-design-cli/code && cargo build --release

# Create your first diagram — from scratch to PlantUML in 60 seconds
sys-design context-model -m model.yaml add system MY_SYSTEM -n "My System"
sys-design context-model -m model.yaml add actor USER -n "User"
sys-design context-model -m model.yaml add interface ITF_API -n "API" -p rest
sys-design context-model -m model.yaml add provide-relation MY_SYSTEM ITF_API
sys-design context-model -m model.yaml add interface-usage USER ITF_API
sys-design validate -m model.yaml -t context
sys-design generate -m model.yaml -o context.puml context-model-diagram
```

## Supported Diagrams

| Diagram | Command | Description |
|---------|---------|-------------|
| Context | `context-model-diagram` | C4 Context — system, actors, external systems, interfaces |
| Concept Model | `concept-model-diagram` | Architecture hierarchy rules and containment |
| Logic View | `logic-model-diagram` | Subsystems, components, modules, dependencies |
| Runtime View | `runtime-model-diagram` | Sequence diagrams with scenarios and control flow |
| Code Model | `code-model-diagram` | Code packages and dependencies |
| Build Model | `build-model-diagram` | Build artifacts, tools, and profiles |
| Delivery Model | `delivery-model-diagram` | Delivery packages and registries |
| Deployment Model | `deployment-model-diagram` | Environments, nodes, and services |

## Usage

### Generate Diagrams

```bash
# Context diagram → file
sys-design generate -m model.yaml -o context.puml context-model-diagram

# Logic view from a specific root element
sys-design generate -m model.yaml logic-model-diagram CTRL_SUBSYSTEM

# Runtime sequence diagram for a scenario
sys-design generate -m model.yaml -o sequence.puml runtime-model-diagram USER_LOGIN

# Any diagram → stdout (pipe to other tools)
sys-design generate -m model.yaml concept-model-diagram
```

### Validate Models

```bash
sys-design validate -m model.yaml -t context
sys-design validate -m model.yaml -t logic-view
sys-design validate -m model.yaml -t runtime-view --format json
```

### CRUD Operations

```bash
# Context model — add actors, interfaces, relations
sys-design context-model -m model.yaml add actor ADMIN -n "Administrator" -t internal
sys-design context-model -m model.yaml add interface ITF_ADMIN -n "Admin API" -p grpc
sys-design context-model -m model.yaml remove actor ADMIN
sys-design context-model -m model.yaml list interfaces

# Logic model — subsystems, components, modules
sys-design logic-model -m model.yaml add subsystem CTRL_SUBSYSTEM -n "Controller"
sys-design logic-model -m model.yaml add component CTRL -n "Controller" --subsystem CTRL_SUBSYSTEM
sys-design logic-model -m model.yaml add module MOTOR_CTRL -n "Motor Controller"
sys-design logic-model -m model.yaml add interface ITF_MOTOR -n "Motor Interface"
sys-design logic-model -m model.yaml add dependency MOTOR_CTRL ITF_SPEED

# Runtime model — scenarios, participants, steps
sys-design runtime-model -m model.yaml add scenario USER_LOGIN -n "User Login Flow"
sys-design runtime-model -m model.yaml add participant USER_LOGIN USER -t actor
sys-design runtime-model -m model.yaml add participant USER_LOGIN WEB_APP -t participant
sys-design runtime-model -m model.yaml add step USER_LOGIN USER WEB_APP "Enter credentials"
sys-design runtime-model -m model.yaml add group USER_LOGIN alt "Auth result" --branches success,failure
```

### Command Reference

| Command | Description |
|---------|-------------|
| `generate` | Generate PlantUML diagrams from YAML |
| `validate` | Validate model files against rules |
| `context-model` | Context model CRUD operations |
| `logic-model` | Logic view CRUD operations |
| `concept-model` | Concept model (hierarchy rules) |
| `runtime-model` | Runtime view CRUD operations |
| `code-model` | Code model CRUD operations |
| `build-model` | Build model CRUD operations |
| `delivery-model` | Delivery model CRUD operations |
| `deployment-model` | Deployment model CRUD operations |

## Validation Rules

| Category | Codes | Example Rules | Severity |
|----------|-------|---------------|----------|
| Completeness | C001–C007 | Required fields, interface providers | Error |
| Consistency | S001–S003 | ID uniqueness, orphan detection | Error/Warning |
| Naming | N001–N003 | UPPER_SNAKE_CASE, name length | Warning/Info |
| Hierarchy | H001–H002 | Concept model conformance | Error |
| Orphan | O001 | Uncontained elements | Error |
| Runtime | R001–R012 | Scenario validation, step references | Error/Warning |

## YAML Model Structure

All diagrams live in a single workspace file:

```yaml
version: "1.0"
metadata:
  title: My Project

context_diagram:
  system:
    id: MY_SYSTEM
    name: My System
  actors:
    - id: USER
      name: User
      type: internal
  interfaces:
    - id: ITF_API
      name: API Interface
      protocol: rest

logic_view:
  system:
    id: MY_SYSTEM
    name: My System
    subsystems:
      - id: CTRL_SUBSYSTEM
        components:
          - id: CTRL
            modules:
              - id: MOTOR_CTRL
                interfaces:
                  - id: ITF_MOTOR
                    name: Motor Interface

runtime_view:
  scenarios:
    - id: USER_LOGIN
      name: User Login Flow
      participants:
        - element_id: USER
          participant_type: actor
      blocks:
        - type: step
          from: USER
          to: WEB_APP
          message: Enter credentials
```

## Building & Testing

```bash
# Build (debug)
cd code && cargo build

# Build (release)
cd code && cargo build --release

# Run tests
cd code && cargo test

# Or use scripts
./scripts/build.sh --release
./scripts/test.sh --all
```

## Architecture

```
code/src/
├── main.rs                  # CLI entry point
├── cli/
│   ├── args.rs              # clap CLI definitions
│   └── commands/            # 10 command implementations
├── model/                   # Data models (8 diagram types + Workspace)
├── store/                   # YAML I/O + CRUD operations
├── validator/               # 40+ validation rules (C/S/N/H/O/R)
├── generator/plantuml/      # PlantUML generators (8 diagram types)
└── utils/error.rs           # Error types
```

## License

[MIT](LICENSE) &copy; 2026 CHIFUNG
