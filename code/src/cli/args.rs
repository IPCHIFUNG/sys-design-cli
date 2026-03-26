use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "sys-design",
    about = "System architecture diagram CLI tool - Generate and validate PlantUML diagrams from YAML models",
    long_about = "sys-design - A CLI tool for generating and validating architecture diagrams.

This tool reads YAML model files and generates PlantUML output, with validation
rules for completeness, consistency, and naming conventions.

SUPPORTED DIAGRAMS:
  • Context Diagram (C4 Context) - System context view with actors, external systems, and interfaces
  • Logic Concept Model - Hierarchy rules defining allowed element relationships
  • Logic View - Concrete implementation following concept model rules

QUICK START:
  # Validate a model
  sys-design validate -s model.yaml -t context

  # Generate a diagram
  sys-design generate -s model.yaml -o output.puml context-model-diagram

  # Add elements via CLI
  sys-design context-model -s model.yaml add system MY_SYSTEM -n \"My System\"

NAMING CONVENTIONS:
  • System, External System, Interface IDs: UPPER_SNAKE_CASE (e.g., PAYMENT_GATEWAY)
  • Actor IDs: UPPER_SNAKE_CASE (e.g., USER, ADMIN)
  • Subsystem, Component, Module IDs: UPPER_SNAKE_CASE (e.g., CTRL_SUBSYSTEM)"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Context model CRUD operations (actors, external systems, interfaces)
    #[command(long_about = "Context model operations for C4 Context diagrams.

Manage system context elements including:
  • System - The central system being described
  • Actors - External/Internal persons interacting with the system
  • External Systems - Other systems the main system connects to
  • Interfaces - API/protocol definitions (REST, gRPC, GraphQL, etc.)

EXAMPLES:
  # Add a system
  sys-design context-model -s model.yaml add system SNP -n \"SNP System\"

  # Add an actor
  sys-design context-model -s model.yaml add actor USER -n \"User\" -t internal

  # Add an interface
  sys-design context-model -s model.yaml add interface ITF_API -n \"API\" -p rest

  # List all actors
  sys-design context-model -s model.yaml list actors")]
    ContextModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: ContextModelCommand,
    },

    /// Logic model CRUD operations (subsystems, components, modules)
    #[command(long_about = "Logic model operations for Logic View diagrams.

Manage logical architecture elements including:
  • System - The top-level system container
  • Subsystems - Major subsystems within the system
  • Components - Functional components within subsystems/system
  • Modules - Modules within components
  • Submodules - Nested submodules (recursive)

EXAMPLES:
  # Add a subsystem
  sys-design logic-model -s model.yaml add subsystem CTRL_SUBSYSTEM -n \"Controller\"

  # Add a component to a subsystem
  sys-design logic-model -s model.yaml add component CTRL --subsystem CTRL_SUBSYSTEM

  # Add a module
  sys-design logic-model -s model.yaml add module MOTOR_CTRL -n \"Motor Controller\"

  # Add a containment relation
  sys-design logic-model -s model.yaml add containment CTRL_SUBSYSTEM CTRL")]
    LogicModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: LogicModelCommand,
    },

    /// Logic architecture concept model (define hierarchy rules)
    #[command(long_about = "Logic Architecture Concept Model operations.

Define the hierarchy rules that the Logic View must follow:
  • Element Types - Types of elements (subsystem, component, module, etc.)
  • Hierarchy Levels - Levels in the architecture with containment rules
  • Containment Rules - Which element types can contain others

EXAMPLES:
  # Add a hierarchy level
  sys-design concept-model -s model.yaml add level SYSTEM -n \"system\" -c SUBSYSTEM,COMPONENT

  # Add element types
  sys-design concept-model -s model.yaml add element subsystem

  # Add containment rule
  sys-design concept-model -s model.yaml add containment SUBSYSTEM COMPONENT

  # List all levels
  sys-design concept-model -s model.yaml list")]
    ConceptModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: ConceptModelCommand,
    },

    /// Generate PlantUML diagrams from YAML models
    #[command(long_about = "Generate PlantUML diagrams from YAML model files.

SUPPORTED DIAGRAM TYPES:
  • context-model-diagram - C4 Context diagram showing system, actors, external systems
  • concept-model-diagram - Hierarchy rules diagram
  • logic-model-diagram - Logic view diagram with nested elements

EXAMPLES:
  # Generate context diagram to file
  sys-design generate -s model.yaml -o context.puml context-model-diagram

  # Generate concept model diagram (stdout)
  sys-design generate -s model.yaml concept-model-diagram

  # Generate full logic view
  sys-design generate -s model.yaml -o logic.puml logic-model-diagram

  # Generate logic view from specific root element
  sys-design generate -s model.yaml logic-model-diagram CTRL_SUBSYSTEM")]
    Generate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        #[command(subcommand)]
        command: GenerateCommand,
    },

    /// Validate model against completeness, consistency, and naming rules
    #[command(long_about = "Validate YAML model files against defined rules.

VALIDATION CATEGORIES:
  • Completeness (C001-C007) - Required fields, interface providers/usages
  • Consistency (S001-S003) - ID uniqueness, orphan elements
  • Naming (N001-N003) - Naming conventions, reserved words
  • Hierarchy (H001-H002) - Conformance to concept model
  • Orphan (O001) - Elements without containment

DIAGRAM TYPES:
  • context - Context diagram validation
  • concept-model - Concept model validation
  • logic-view - Logic view validation

EXAMPLES:
  # Validate context diagram
  sys-design validate -s model.yaml -t context

  # Validate with JSON output
  sys-design validate -s model.yaml -t logic-view --format json")]
    Validate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output format (text or json)
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Diagram type to validate (context, concept-model, logic-view)
        #[arg(short = 't', long, value_enum, default_value = "context")]
        type_: DiagramType,
    },
}

/// Generate command subcommands
#[derive(Subcommand)]
pub enum GenerateCommand {
    /// Generate C4 Context diagram (system context view)
    #[command(long_about = "Generate a C4 Context diagram showing the system as a black box.

Displays:
  • Central system with its boundaries
  • Actors (users) interacting with the system
  • External systems and their connections
  • Interfaces with protocol types

OUTPUT: PlantUML C4 Context diagram code")]
    ContextModelDiagram,

    /// Generate concept model diagram (hierarchy rules visualization)
    #[command(long_about = "Generate a diagram showing the concept model hierarchy rules.

Displays:
  • Hierarchy levels (SYSTEM, SUBSYSTEM, COMPONENT, etc.)
  • Containment relationships between levels
  • Element types defined in the model

OUTPUT: PlantUML diagram showing the hierarchy structure")]
    ConceptModelDiagram,

    /// Generate logic view diagram (concrete implementation)
    #[command(long_about = "Generate a logic view diagram showing the concrete implementation.

Displays:
  • System with all subsystems, components, modules
  • Containment hierarchy
  • Interfaces and their providers
  • Dependencies between modules

ARGUMENTS:
  [ROOT] - Optional root element ID to generate a partial diagram.
           If not specified, generates from system root.

OUTPUT: PlantUML component diagram with nested elements

EXAMPLES:
  # Generate full diagram
  sys-design generate -s model.yaml logic-model-diagram

  # Generate from specific subsystem
  sys-design generate -s model.yaml logic-model-diagram CTRL_SUBSYSTEM")]
    LogicModelDiagram {
        /// Root element ID to start from (optional, defaults to system root)
        root: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DiagramType {
    /// Context diagram - System context view with actors and external systems
    Context,
    /// Concept model - Hierarchy rules defining allowed element relationships
    ConceptModel,
    /// Logic view - Concrete implementation following concept model rules
    LogicView,
}

#[derive(Subcommand)]
pub enum ContextModelCommand {
    /// Add elements to the context model
    #[command(subcommand)]
    Add(AddCommand),

    /// Remove elements from the context model
    #[command(subcommand)]
    Remove(RemoveCommand),

    /// List all elements of a specific type
    #[command(long_about = "List all elements of a specific type.

ELEMENT TYPES:
  • system - The central system
  • actors - All actors (internal and external)
  • external-systems - All external systems
  • interfaces - All defined interfaces
  • relations - Interface providers and usages")]
    List {
        /// Element type: system, actors, external-systems, interfaces, relations
        #[arg(value_enum)]
        element: ListElement,
    },

    /// Show detailed information about an element
    Show {
        /// Element ID to display
        id: String,
    },
}

#[derive(Subcommand)]
pub enum AddCommand {
    /// Add or update the central system (singleton, only one system per context)
    System {
        /// System ID (UPPER_SNAKE_CASE, e.g., MY_SYSTEM, SNP)
        id: String,
        /// System display name
        #[arg(short, long)]
        name: Option<String>,
        /// System description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add an actor (person interacting with the system)
    #[command(long_about = "Add an actor to the context model.

Actor types:
  • external - Outside the organization (e.g., customers, partners)
  • internal - Inside the organization (e.g., admins, operators)")]
    Actor {
        /// Actor ID (UPPER_SNAKE_CASE, e.g., USER, ADMIN)
        id: String,
        /// Actor display name
        #[arg(short, long)]
        name: Option<String>,
        /// Actor description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Actor type: external (outside org) or internal (inside org)
        #[arg(short = 't', long, value_enum, default_value = "external")]
        actor_type: ActorTypeArg,
    },

    /// Add an external system (other systems the main system connects to)
    ExternalSystem {
        /// External system ID (UPPER_SNAKE_CASE, e.g., PAYMENT_GATEWAY)
        id: String,
        /// External system display name
        #[arg(short, long)]
        name: Option<String>,
        /// External system description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Technology used (e.g., \"REST API\", \"gRPC\", \"Kafka\")
        #[arg(short, long)]
        tech: Option<String>,
    },

    /// Add an interface (API/protocol definition)
    #[command(long_about = "Add an interface definition to the context model.

Protocol types:
  • rest - RESTful HTTP API
  • grpc - gRPC protocol
  • graphql - GraphQL API
  • websocket - WebSocket connection
  • mqtt - MQTT messaging
  • amqp - AMQP messaging")]
    Interface {
        /// Interface ID (UPPER_SNAKE_CASE, e.g., ITF_API, ITF_PAYMENT)
        id: String,
        /// Interface display name
        #[arg(short, long)]
        name: Option<String>,
        /// Interface description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Protocol type: rest, grpc, graphql, websocket, mqtt, amqp
        #[arg(short = 'p', long, value_enum, default_value = "rest")]
        protocol: ProtocolArg,
    },

    /// Add a provide relation (system provides an interface)
    #[command(long_about = "Define that a system provides an interface.

This establishes which system exposes which interface.
Both system and interface must exist before creating this relation.")]
    ProvideRelation {
        /// System ID that provides the interface
        system_id: String,
        /// Interface ID being provided
        interface_id: String,
    },

    /// Add an interface usage (actor/system uses an interface)
    #[command(long_about = "Define that an actor or system uses an interface.

This establishes dependencies between actors/systems and interfaces.
Both actor/system and interface must exist before creating this relation.")]
    InterfaceUsage {
        /// Actor or system ID that uses the interface
        actor_id: String,
        /// Interface ID being used
        interface_id: String,
    },
}

#[derive(Subcommand)]
pub enum RemoveCommand {
    /// Remove an actor from the context model
    Actor {
        /// Actor ID to remove
        id: String,
    },

    /// Remove an external system from the context model
    ExternalSystem {
        /// External system ID to remove
        id: String,
    },

    /// Remove an interface from the context model
    #[command(long_about = "Remove an interface.
Note: This also removes related provide relations and interface usages.")]
    Interface {
        /// Interface ID to remove
        id: String,
    },

    /// Remove a provide relation (system no longer provides interface)
    ProvideRelation {
        /// System ID
        system_id: String,
        /// Interface ID
        interface_id: String,
    },

    /// Remove an interface usage (actor/system no longer uses interface)
    InterfaceUsage {
        /// Actor or system ID
        actor_id: String,
        /// Interface ID
        interface_id: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ListElement {
    System,
    Actors,
    ExternalSystems,
    Interfaces,
    Relations,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ActorTypeArg {
    External,
    Internal,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProtocolArg {
    Rest,
    Grpc,
    Graphql,
    WebSocket,
    Mqtt,
    Amqp,
}

impl From<ActorTypeArg> for crate::model::c4::context::ActorType {
    fn from(value: ActorTypeArg) -> Self {
        match value {
            ActorTypeArg::External => crate::model::c4::context::ActorType::External,
            ActorTypeArg::Internal => crate::model::c4::context::ActorType::Internal,
        }
    }
}

impl From<ProtocolArg> for crate::model::c4::context::Protocol {
    fn from(value: ProtocolArg) -> Self {
        match value {
            ProtocolArg::Rest => crate::model::c4::context::Protocol::Rest,
            ProtocolArg::Grpc => crate::model::c4::context::Protocol::Grpc,
            ProtocolArg::Graphql => crate::model::c4::context::Protocol::Graphql,
            ProtocolArg::WebSocket => crate::model::c4::context::Protocol::WebSocket,
            ProtocolArg::Mqtt => crate::model::c4::context::Protocol::Mqtt,
            ProtocolArg::Amqp => crate::model::c4::context::Protocol::Amqp,
        }
    }
}

// ==================== Logic Model Commands ====================

#[derive(Subcommand)]
pub enum LogicModelCommand {
    /// Add elements to the logic model
    #[command(subcommand)]
    Add(LogicAddCommand),

    /// Remove elements from the logic model
    #[command(subcommand)]
    Remove(LogicRemoveCommand),

    /// List all elements of a specific type
    #[command(long_about = "List all elements of a specific type.

ELEMENT TYPES:
  • system - The top-level system
  • subsystems - All subsystems
  • components - All components
  • modules - All modules
  • interfaces - All interfaces
  • dependencies - All module dependencies")]
    List {
        /// Element type: system, subsystems, components, modules, interfaces, dependencies
        #[arg(value_enum)]
        element: LogicListElement,
    },

    /// Show detailed information about an element
    Show {
        /// Element ID to display
        id: String,
    },
}

#[derive(Subcommand)]
pub enum LogicAddCommand {
    /// Add or update the system (top-level container, singleton)
    System {
        /// System ID (UPPER_SNAKE_CASE, e.g., MY_SYSTEM, SNP)
        id: String,
        /// System display name
        #[arg(short, long)]
        name: Option<String>,
        /// System description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a subsystem (major division within the system)
    Subsystem {
        /// Subsystem ID (UPPER_SNAKE_CASE, e.g., CTRL_SUBSYSTEM)
        id: String,
        /// Subsystem display name
        #[arg(short, long)]
        name: Option<String>,
        /// Subsystem description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a component (functional unit within subsystem or system)
    #[command(long_about = "Add a component to the system or a subsystem.

Components are functional units that can contain modules.
If --subsystem is specified, the component belongs to that subsystem.
Otherwise, it belongs directly to the system.")]
    Component {
        /// Component ID (UPPER_SNAKE_CASE, e.g., CTRL, MOTOR)
        id: String,
        /// Component display name
        #[arg(short, long)]
        name: Option<String>,
        /// Component description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Parent subsystem ID (optional, defaults to system level)
        #[arg(short, long)]
        subsystem: Option<String>,
    },

    /// Add a module (unit within a component)
    Module {
        /// Module ID (UPPER_SNAKE_CASE, e.g., MOTOR_CTRL, POSITION_LOOP)
        id: String,
        /// Module display name
        #[arg(short, long)]
        name: Option<String>,
        /// Module description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a submodule (nested unit within a module, can be recursive)
    Submodule {
        /// Submodule ID (UPPER_SNAKE_CASE)
        id: String,
        /// Submodule display name
        #[arg(short, long)]
        name: Option<String>,
        /// Submodule description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a generic element based on concept model type
    #[command(long_about = "Add a generic element with a type defined in the concept model.

This allows adding elements of any type defined in the concept model,
not just the predefined types (system, subsystem, component, etc.).")]
    Element {
        /// Element type (must be defined in concept model)
        #[arg(value_name = "TYPE")]
        type_name: String,
        /// Element ID (UPPER_SNAKE_CASE)
        id: String,
        /// Element display name
        #[arg(short, long)]
        name: Option<String>,
        /// Element description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add an interface (API/contract definition)
    Interface {
        /// Interface ID (UPPER_SNAKE_CASE, e.g., ITF_MOTOR, ITF_CONFIG)
        id: String,
        /// Interface display name
        #[arg(short, long)]
        name: Option<String>,
        /// Interface description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a provide relation (element provides an interface)
    #[command(long_about = "Define that an element provides an interface.

This establishes which element exposes which interface.
Both element and interface must exist before creating this relation.")]
    ProvideRelation {
        /// Element ID that provides the interface
        element_id: String,
        /// Interface ID being provided
        interface_id: String,
    },

    /// Add a containment relation (parent element contains child)
    #[command(long_about = "Define that a parent element contains a child element.

This establishes the hierarchy structure.
Example: system contains subsystem, subsystem contains component.

Both parent and child elements must exist before creating this relation.
Must conform to the concept model's hierarchy rules.")]
    Containment {
        /// Parent element ID
        parent_id: String,
        /// Child element ID
        child_id: String,
    },

    /// Add a dependency (module depends on an interface)
    #[command(long_about = "Define that a module depends on an interface.

This establishes that a module uses/requires an interface.
Both module and interface must exist before creating this relation.")]
    Dependency {
        /// Module ID that has the dependency
        module_id: String,
        /// Interface ID being used
        interface_id: String,
    },

    /// Expose an interface from a component (make visible externally)
    #[command(long_about = "Expose an interface from a component.

This makes an interface visible at the component level,
allowing external elements to use it through the component.")]
    Expose {
        /// Component ID
        component_id: String,
        /// Interface ID to expose
        interface_id: String,
    },
}

#[derive(Subcommand)]
pub enum LogicRemoveCommand {
    /// Remove a subsystem from the logic model
    Subsystem {
        /// Subsystem ID to remove
        id: String,
    },

    /// Remove a component from the logic model
    Component {
        /// Component ID to remove
        id: String,
    },

    /// Remove a module from a component
    #[command(long_about = "Remove a module from a specific component.
Note: This also removes nested submodules and dependencies.")]
    Module {
        /// Parent component ID
        component_id: String,
        /// Module ID to remove
        id: String,
    },

    /// Remove an interface from a module
    Interface {
        /// Parent component ID
        component_id: String,
        /// Parent module ID
        module_id: String,
        /// Interface ID to remove
        id: String,
    },

    /// Remove a dependency from a module
    Dependency {
        /// Parent component ID
        component_id: String,
        /// Module ID
        module_id: String,
        /// Interface ID of the dependency
        interface_id: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LogicListElement {
    System,
    Subsystems,
    Components,
    Modules,
    Interfaces,
    Dependencies,
}

// ==================== Concept Model Commands ====================

#[derive(Subcommand)]
pub enum ConceptModelCommand {
    /// Add elements to the concept model
    #[command(subcommand)]
    Add(ConceptModelAddCommand),

    /// Remove elements from the concept model
    #[command(subcommand)]
    Remove(ConceptModelRemoveCommand),

    /// List all hierarchy levels and element types
    #[command(long_about = "List all hierarchy levels and element types defined in the concept model.

Shows:
  • All hierarchy levels with their names and IDs
  • Element types that each level can contain
  • All registered element types")]
    List,

    /// Show detailed information about a hierarchy level
    Show {
        /// Level ID to display (e.g., SYSTEM, COMPONENT)
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ConceptModelAddCommand {
    /// Add an element type (e.g., subsystem, component, module)
    #[command(long_about = "Register a new element type in the concept model.

Element types define what kinds of elements can exist in the logic view.
Common types: subsystem, component, module, submodule, layer, service")]
    Element {
        /// Element type name (e.g., subsystem, component, module)
        #[arg(value_name = "TYPE")]
        type_name: String,
    },

    /// Add a containment relationship (parent type can contain child type)
    #[command(long_about = "Define that a parent element type can contain a child element type.

This establishes the allowed hierarchy structure.
Example: A 'subsystem' can contain 'component' elements.

Both element types should exist before creating this rule.")]
    Containment {
        /// Parent element type (e.g., subsystem)
        parent: String,
        /// Child element type (e.g., component)
        child: String,
    },

    /// Add a hierarchy level with containment rules
    #[command(long_about = "Add a hierarchy level with its containment rules.

Hierarchy levels define the architectural layers and what they can contain.
Example: SYSTEM level can contain SUBSYSTEM and COMPONENT types.

Use -c/--can-contain with comma-separated list of allowed child types.")]
    Level {
        /// Level ID (UPPER_CASE, e.g., SYSTEM, COMPONENT, LAYER)
        id: String,
        /// Level display name (e.g., \"system\", \"component\")
        #[arg(short, long)]
        name: Option<String>,
        /// Level description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Types this level can contain (comma-separated, e.g., -c SUBSYSTEM,COMPONENT)
        #[arg(short, long, value_delimiter = ',')]
        can_contain: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ConceptModelRemoveCommand {
    /// Remove an element type from the concept model
    #[command(long_about = "Remove an element type.
Note: This may invalidate existing logic view elements of this type.")]
    Element {
        /// Element type name to remove
        #[arg(value_name = "TYPE")]
        type_name: String,
    },

    /// Remove a hierarchy level from the concept model
    #[command(long_about = "Remove a hierarchy level.
Note: This may invalidate hierarchy rules referencing this level.")]
    Level {
        /// Level ID to remove
        id: String,
    },
}
