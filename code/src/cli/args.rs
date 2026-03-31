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
  sys-design validate -m model.yaml -t context

  # Generate a diagram
  sys-design generate -m model.yaml -o output.puml context-model-diagram

  # Add elements via CLI
  sys-design context-model -m model.yaml add system MY_SYSTEM -n \"My System\"

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
  sys-design context-model -m model.yaml add system SNP -n \"SNP System\"

  # Add an actor
  sys-design context-model -m model.yaml add actor USER -n \"User\" -t internal

  # Add an interface
  sys-design context-model -m model.yaml add interface ITF_API -n \"API\" -p rest

  # List all actors
  sys-design context-model -m model.yaml list actors")]
    ContextModel {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

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
  sys-design logic-model -m model.yaml add subsystem CTRL_SUBSYSTEM -n \"Controller\"

  # Add a component to a subsystem
  sys-design logic-model -m model.yaml add component CTRL --subsystem CTRL_SUBSYSTEM

  # Add a module
  sys-design logic-model -m model.yaml add module MOTOR_CTRL -n \"Motor Controller\"

  # Add a containment relation
  sys-design logic-model -m model.yaml add containment CTRL_SUBSYSTEM CTRL")]
    LogicModel {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

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
  sys-design concept-model -m model.yaml add level SYSTEM -n \"system\" -c SUBSYSTEM,COMPONENT

  # Add element types
  sys-design concept-model -m model.yaml add element subsystem

  # Add containment rule
  sys-design concept-model -m model.yaml add containment SUBSYSTEM COMPONENT

  # List all levels
  sys-design concept-model -m model.yaml list")]
    ConceptModel {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

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
  sys-design generate -m model.yaml -o context.puml context-model-diagram

  # Generate concept model diagram (stdout)
  sys-design generate -m model.yaml concept-model-diagram

  # Generate full logic view
  sys-design generate -m model.yaml -o logic.puml logic-model-diagram

  # Generate logic view from specific root element
  sys-design generate -m model.yaml logic-model-diagram CTRL_SUBSYSTEM")]
    Generate {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

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
  sys-design validate -m model.yaml -t context

  # Validate with JSON output
  sys-design validate -m model.yaml -t logic-view --format json")]
    Validate {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

        /// Output format (text or json)
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Diagram type to validate (context, concept-model, logic-view)
        #[arg(short = 't', long, value_enum, default_value = "context")]
        type_: DiagramType,
    },

    /// Runtime model CRUD operations (scenarios, participants, steps, groups)
    #[command(long_about = "Runtime model operations for Runtime View diagrams.

Manage runtime view elements including:
  • Scenarios - Runtime flows (use cases, features)
  • Participants - Elements from context/logic view involved in scenarios
  • Steps - Interactions between participants (sync, async, return, lost)
  • Groups - Control flow (alt, loop, par, opt, break, critical)
  • Notes - Annotations
  • Dividers - Section markers

EXAMPLES:
  # Add a scenario
  sys-design runtime-model -m model.yaml add scenario USER_LOGIN -n \"User Login\"

  # Add participants (must exist in context or logic view)
  sys-design runtime-model -m model.yaml add participant USER_LOGIN USER -t actor

  # Add a step
  sys-design runtime-model -m model.yaml add step USER_LOGIN USER APP \"Login request\"

  # Add an alt group with branches
  sys-design runtime-model -m model.yaml add group USER_LOGIN alt \"Result\" --branches success,failure

  # Add step to a specific branch
  sys-design runtime-model -m model.yaml add step USER_LOGIN APP USER \"Token\" -t return --group \"Result\" --branch success")]
    RuntimeModel {
        /// Model file path
        #[arg(short = 'm', long = "model_file", value_name = "FILE")]
        model_file: PathBuf,

        #[command(subcommand)]
        command: RuntimeModelCommand,
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
  sys-design generate -m model.yaml logic-model-diagram

  # Generate from specific subsystem
  sys-design generate -m model.yaml logic-model-diagram CTRL_SUBSYSTEM")]
    LogicModelDiagram {
        /// Root element ID to start from (optional, defaults to system root)
        root: Option<String>,
    },

    /// Generate runtime view sequence diagram
    #[command(long_about = "Generate a PlantUML sequence diagram for a runtime scenario.

Displays:
  • Participants as lifelines (actors, systems, components)
  • Interaction steps with arrows (sync, async, return, lost)
  • Control flow groups (alt, loop, par, opt, etc.)
  • Notes and dividers

ARGUMENTS:
  [SCENARIO_ID] - Scenario ID to generate. Required when multiple scenarios exist.
                   If only one scenario exists, it is used automatically.

OUTPUT: PlantUML sequence diagram

EXAMPLES:
  # Generate specific scenario
  sys-design generate -m model.yaml runtime-model-diagram USER_LOGIN

  # Generate to file
  sys-design generate -m model.yaml -o login.puml runtime-model-diagram USER_LOGIN")]
    RuntimeModelDiagram {
        /// Scenario ID to generate (required when multiple scenarios exist)
        scenario_id: Option<String>,
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
    /// Runtime view - Dynamic behavior as sequence diagrams
    RuntimeView,
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

// ==================== Runtime Model Commands ====================

#[derive(Subcommand)]
pub enum RuntimeModelCommand {
    /// Add elements to the runtime model
    #[command(subcommand)]
    Add(RuntimeAddCommand),

    /// Remove elements from the runtime model
    #[command(subcommand)]
    Remove(RuntimeRemoveCommand),

    /// List elements
    #[command(long_about = "List runtime model elements.

ELEMENT TYPES:
  • scenarios - All scenarios
  • participants - Participants in a scenario (requires --scenario)
  • steps - Steps in a scenario (requires --scenario)
  • groups - Groups in a scenario (requires --scenario)")]
    List {
        /// Element type: scenarios, participants, steps, groups
        #[arg(value_enum)]
        element: RuntimeListElement,
        /// Scenario ID (required for participants, steps, groups)
        #[arg(short, long)]
        scenario: Option<String>,
    },

    /// Show detailed scenario information
    Show {
        /// Scenario ID to display
        scenario_id: String,
    },
}

#[derive(Subcommand)]
pub enum RuntimeAddCommand {
    /// Add a new scenario (runtime flow)
    Scenario {
        /// Scenario ID (UPPER_SNAKE_CASE, e.g., USER_LOGIN, DATA_SYNC)
        id: String,
        /// Scenario display name
        #[arg(short, long)]
        name: Option<String>,
        /// Scenario description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a participant (references element from context/logic view)
    #[command(long_about = "Add a participant to a scenario.

Participant types:
  • actor - Person/external entity
  • participant - Generic component
  • boundary - System boundary/interface
  • control - Controller/coordinator
  • entity - Data entity
  • database - Data store
  • collections - Collection of items
  • queue - Message queue

The element_id must exist in the context diagram or logic view.")]
    Participant {
        /// Scenario ID to add participant to
        scenario_id: String,
        /// Element ID from context diagram or logic view
        element_id: String,
        /// Participant type (default: participant)
        #[arg(short = 't', long, value_enum, default_value = "participant")]
        participant_type: ParticipantTypeArg,
        /// Display alias
        #[arg(long)]
        alias: Option<String>,
        /// Color (e.g., #FF0000, red)
        #[arg(long)]
        color: Option<String>,
    },

    /// Add an interaction step
    #[command(long_about = "Add an interaction step between participants.

Step types:
  • sync - Synchronous call (solid arrow ->)
  • async - Asynchronous message (open arrow ->>)
  • return - Return/response (dashed arrow -->)
  • lost - Lost message (circle-x arrow -[o]->)

The order is auto-assigned (max + 1).")]
    Step {
        /// Scenario ID
        scenario_id: String,
        /// Source participant element ID
        from: String,
        /// Target participant element ID
        to: String,
        /// Message content
        message: String,
        /// Step type: sync, async, return, lost
        #[arg(short = 't', long, value_enum, default_value = "sync")]
        step_type: StepTypeArg,
        /// Protocol (e.g., REST, gRPC)
        #[arg(short = 'p', long)]
        protocol: Option<String>,
        /// Color
        #[arg(long)]
        color: Option<String>,
        /// Activate target lifeline
        #[arg(long)]
        activate: Option<bool>,
        /// Target group label (one level only)
        #[arg(long)]
        group: Option<String>,
        /// Target branch label (for alt groups only)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Add a control flow group
    #[command(long_about = "Add a control flow group (UML combined fragment).

Group types:
  • alt - Conditional (if/else), requires --branches
  • opt - Optional execution
  • loop - Iteration
  • par - Parallel execution
  • break - Exception/break handling
  • critical - Atomic/critical section
  • group - Generic named grouping

For alt groups, specify branches with --branches (comma-separated).
Non-alt groups use inline blocks.")]
    Group {
        /// Scenario ID
        scenario_id: String,
        /// Group type: alt, opt, loop, par, break, critical, group
        group_type: GroupTypeArg,
        /// Group label
        label: String,
        /// Branch labels for alt groups (comma-separated, e.g., success,failure)
        #[arg(short, long, value_delimiter = ',')]
        branches: Vec<String>,
        /// Parent group label (one level only)
        #[arg(long)]
        group: Option<String>,
        /// Parent branch label (for alt parent group)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Add an annotation note
    Note {
        /// Scenario ID
        scenario_id: String,
        /// Note position: left, right, over
        position: NotePositionArg,
        /// Target participant element ID
        target: String,
        /// Note text
        text: String,
    },

    /// Add a section divider
    Divider {
        /// Scenario ID
        scenario_id: String,
        /// Divider label
        label: String,
        /// Insert after step with this order number
        #[arg(long)]
        after_order: u32,
    },
}

#[derive(Subcommand)]
pub enum RuntimeRemoveCommand {
    /// Remove a scenario
    Scenario {
        /// Scenario ID to remove
        id: String,
    },

    /// Remove a participant (cascades to referencing steps)
    #[command(long_about = "Remove a participant.
Note: This also removes all steps that reference this participant.")]
    Participant {
        /// Scenario ID
        scenario_id: String,
        /// Participant element ID to remove
        element_id: String,
    },

    /// Remove a step by order number
    Step {
        /// Scenario ID
        scenario_id: String,
        /// Step order number
        order: u32,
        /// Target group label (one level only)
        #[arg(long)]
        group: Option<String>,
        /// Target branch label (for alt groups)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Remove a group (cascades to entire subtree)
    #[command(long_about = "Remove a group.
Note: This removes the entire group subtree including all nested steps and groups.")]
    Group {
        /// Scenario ID
        scenario_id: String,
        /// Group label to remove
        label: String,
        /// Parent group label (one level only)
        #[arg(long)]
        group: Option<String>,
        /// Parent branch label (for alt parent group)
        #[arg(long)]
        branch: Option<String>,
    },

    /// Remove a note by index
    Note {
        /// Scenario ID
        scenario_id: String,
        /// Note index (0-based)
        index: usize,
    },

    /// Remove a divider by index
    Divider {
        /// Scenario ID
        scenario_id: String,
        /// Divider index (0-based)
        index: usize,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum RuntimeListElement {
    Scenarios,
    Participants,
    Steps,
    Groups,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ParticipantTypeArg {
    Participant,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

impl From<ParticipantTypeArg> for crate::model::runtime::ParticipantType {
    fn from(value: ParticipantTypeArg) -> Self {
        match value {
            ParticipantTypeArg::Participant => crate::model::runtime::ParticipantType::Participant,
            ParticipantTypeArg::Actor => crate::model::runtime::ParticipantType::Actor,
            ParticipantTypeArg::Boundary => crate::model::runtime::ParticipantType::Boundary,
            ParticipantTypeArg::Control => crate::model::runtime::ParticipantType::Control,
            ParticipantTypeArg::Entity => crate::model::runtime::ParticipantType::Entity,
            ParticipantTypeArg::Database => crate::model::runtime::ParticipantType::Database,
            ParticipantTypeArg::Collections => crate::model::runtime::ParticipantType::Collections,
            ParticipantTypeArg::Queue => crate::model::runtime::ParticipantType::Queue,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum StepTypeArg {
    Sync,
    Async,
    Return,
    Lost,
}

impl From<StepTypeArg> for crate::model::runtime::StepType {
    fn from(value: StepTypeArg) -> Self {
        match value {
            StepTypeArg::Sync => crate::model::runtime::StepType::Sync,
            StepTypeArg::Async => crate::model::runtime::StepType::Async,
            StepTypeArg::Return => crate::model::runtime::StepType::Return,
            StepTypeArg::Lost => crate::model::runtime::StepType::Lost,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum GroupTypeArg {
    Alt,
    Opt,
    Loop,
    Par,
    Break,
    Critical,
    Group,
}

impl From<GroupTypeArg> for crate::model::runtime::GroupType {
    fn from(value: GroupTypeArg) -> Self {
        match value {
            GroupTypeArg::Alt => crate::model::runtime::GroupType::Alt,
            GroupTypeArg::Opt => crate::model::runtime::GroupType::Opt,
            GroupTypeArg::Loop => crate::model::runtime::GroupType::Loop,
            GroupTypeArg::Par => crate::model::runtime::GroupType::Par,
            GroupTypeArg::Break => crate::model::runtime::GroupType::Break,
            GroupTypeArg::Critical => crate::model::runtime::GroupType::Critical,
            GroupTypeArg::Group => crate::model::runtime::GroupType::Group,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum NotePositionArg {
    Left,
    Right,
    Over,
}

impl From<NotePositionArg> for crate::model::runtime::NotePosition {
    fn from(value: NotePositionArg) -> Self {
        match value {
            NotePositionArg::Left => crate::model::runtime::NotePosition::Left,
            NotePositionArg::Right => crate::model::runtime::NotePosition::Right,
            NotePositionArg::Over => crate::model::runtime::NotePosition::Over,
        }
    }
}
