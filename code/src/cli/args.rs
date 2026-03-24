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

    /// Generate PlantUML diagram
    Generate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output file path (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate the model
    Validate {
        /// YAML source file path
        #[arg(short, long, value_name = "FILE")]
        src: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: OutputFormat,
    },
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
