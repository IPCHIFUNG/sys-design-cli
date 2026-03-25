use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sys-design", about = "System architecture diagram CLI tool")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Context model operations
    ContextModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: ContextModelCommand,
    },

    /// Logic concept model operations
    LogicModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: LogicModelCommand,
    },

    /// Logic Architecture Concept Model operations (defines hierarchy rules)
    ConceptModel {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        #[command(subcommand)]
        command: ConceptModelCommand,
    },

    /// Generate PlantUML diagram
    Generate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Diagram type
        #[arg(short = 't', long, value_enum, default_value = "context")]
        type_: DiagramType,
    },

    /// Validate the model
    Validate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Diagram type
        #[arg(short = 't', long, value_enum, default_value = "context")]
        type_: DiagramType,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum DiagramType {
    /// Context diagram (system as black box)
    Context,
    /// Logic Architecture Concept Model (defines hierarchy rules)
    ConceptModel,
    /// Logic View (concrete implementation, must follow concept model)
    LogicView,
}

#[derive(Subcommand)]
pub enum ContextModelCommand {
    /// Add an element
    #[command(subcommand)]
    Add(AddCommand),

    /// Remove an element
    #[command(subcommand)]
    Remove(RemoveCommand),

    /// List elements
    List {
        /// Element type to list
        #[arg(value_enum)]
        element: ListElement,
    },

    /// Show element details
    Show {
        /// Element ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum AddCommand {
    /// Add or update the system
    System {
        /// System ID
        id: String,
        /// System name
        #[arg(short, long)]
        name: Option<String>,
        /// System description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add an actor
    Actor {
        /// Actor ID
        id: String,
        /// Actor name
        #[arg(short, long)]
        name: Option<String>,
        /// Actor description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Actor type (external or internal)
        #[arg(short = 't', long, value_enum, default_value = "external")]
        actor_type: ActorTypeArg,
    },

    /// Add an external system
    ExternalSystem {
        /// External system ID
        id: String,
        /// External system name
        #[arg(short, long)]
        name: Option<String>,
        /// External system description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Technology used
        #[arg(short, long)]
        tech: Option<String>,
    },

    /// Add an interface
    Interface {
        /// Interface ID
        id: String,
        /// Interface name
        #[arg(short, long)]
        name: Option<String>,
        /// Interface description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Protocol type
        #[arg(short = 'p', long, value_enum, default_value = "rest")]
        protocol: ProtocolArg,
    },

    /// Add a provide relation (system provides interface)
    ProvideRelation {
        /// System ID
        system_id: String,
        /// Interface ID
        interface_id: String,
    },

    /// Add an interface usage (actor uses interface)
    InterfaceUsage {
        /// Actor or system ID
        actor_id: String,
        /// Interface ID
        interface_id: String,
    },
}

#[derive(Subcommand)]
pub enum RemoveCommand {
    /// Remove an actor
    Actor {
        /// Actor ID
        id: String,
    },

    /// Remove an external system
    ExternalSystem {
        /// External system ID
        id: String,
    },

    /// Remove an interface
    Interface {
        /// Interface ID
        id: String,
    },

    /// Remove a provide relation
    ProvideRelation {
        /// System ID
        system_id: String,
        /// Interface ID
        interface_id: String,
    },

    /// Remove an interface usage
    InterfaceUsage {
        /// Actor ID
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
    /// Add an element
    #[command(subcommand)]
    Add(LogicAddCommand),

    /// Remove an element
    #[command(subcommand)]
    Remove(LogicRemoveCommand),

    /// List elements
    List {
        /// Element type to list
        #[arg(value_enum)]
        element: LogicListElement,
    },

    /// Show element details
    Show {
        /// Element ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum LogicAddCommand {
    /// Add or update the system
    System {
        /// System ID
        id: String,
        /// System name
        #[arg(short, long)]
        name: Option<String>,
        /// System description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a subsystem
    Subsystem {
        /// Subsystem ID
        id: String,
        /// Subsystem name
        #[arg(short, long)]
        name: Option<String>,
        /// Subsystem description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a component to system or subsystem
    Component {
        /// Component ID
        id: String,
        /// Component name
        #[arg(short, long)]
        name: Option<String>,
        /// Component description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Parent subsystem ID (optional, if not specified adds to system)
        #[arg(short, long)]
        subsystem: Option<String>,
    },

    /// Add a module (standalone element)
    Module {
        /// Module ID
        id: String,
        /// Module name
        #[arg(short, long)]
        name: Option<String>,
        /// Module description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a submodule (standalone element)
    Submodule {
        /// Submodule ID
        id: String,
        /// Submodule name
        #[arg(short, long)]
        name: Option<String>,
        /// Submodule description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a generic element based on concept model
    Element {
        /// Element type (must be defined in concept model)
        #[arg(value_name = "TYPE")]
        type_name: String,
        /// Element ID
        id: String,
        /// Element name
        #[arg(short, long)]
        name: Option<String>,
        /// Element description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add an interface (standalone element)
    Interface {
        /// Interface ID
        id: String,
        /// Interface name
        #[arg(short, long)]
        name: Option<String>,
        /// Interface description
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },

    /// Add a provide relation (element provides interface)
    ProvideRelation {
        /// Element ID that provides the interface
        element_id: String,
        /// Interface ID to provide
        interface_id: String,
    },

    /// Add a containment relation (parent contains child element)
    Containment {
        /// Parent element ID
        parent_id: String,
        /// Child element ID
        child_id: String,
    },

    /// Add a dependency (module uses interface)
    Dependency {
        /// Module ID that has the dependency
        module_id: String,
        /// Interface ID that is being used
        interface_id: String,
    },

    /// Expose an interface from a component
    Expose {
        /// Component ID
        component_id: String,
        /// Interface ID to expose
        interface_id: String,
    },
}

#[derive(Subcommand)]
pub enum LogicRemoveCommand {
    /// Remove a subsystem
    Subsystem {
        /// Subsystem ID
        id: String,
    },

    /// Remove a component
    Component {
        /// Component ID
        id: String,
    },

    /// Remove a module
    Module {
        /// Component ID
        component_id: String,
        /// Module ID
        id: String,
    },

    /// Remove an interface
    Interface {
        /// Component ID
        component_id: String,
        /// Module ID
        module_id: String,
        /// Interface ID
        id: String,
    },

    /// Remove a dependency
    Dependency {
        /// Component ID
        component_id: String,
        /// Module ID
        module_id: String,
        /// Interface ID
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
    List,

    /// Show level details
    Show {
        /// Level ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ConceptModelAddCommand {
    /// Add an element type (e.g., subsystem, component, module)
    Element {
        /// Element type name
        #[arg(value_name = "TYPE")]
        type_name: String,
    },

    /// Add a containment relationship (parent can contain child)
    Containment {
        /// Parent element type
        parent: String,
        /// Child element type
        child: String,
    },

    /// Add a hierarchy level
    Level {
        /// Level ID (e.g., SYSTEM, COMPONENT, LAYER)
        id: String,
        /// Level name
        #[arg(short, long)]
        name: Option<String>,
        /// Level description
        #[arg(short = 'd', long)]
        desc: Option<String>,
        /// Types this level can contain (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        can_contain: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ConceptModelRemoveCommand {
    /// Remove an element type
    Element {
        /// Element type name
        #[arg(value_name = "TYPE")]
        type_name: String,
    },

    /// Remove a hierarchy level
    Level {
        /// Level ID
        id: String,
    },
}
