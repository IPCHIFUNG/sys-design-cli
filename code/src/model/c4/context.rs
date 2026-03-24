use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for Context Diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDiagram {
    pub version: String,
    pub kind: DiagramKind,
    pub metadata: Metadata,
    pub system: System,
    #[serde(default)]
    pub actors: Vec<Actor>,
    #[serde(default)]
    pub external_systems: Vec<ExternalSystem>,
    #[serde(default)]
    pub interfaces: Vec<Interface>,
    #[serde(default)]
    pub interface_providers: Vec<InterfaceProvider>,
    #[serde(default)]
    pub interface_usages: Vec<InterfaceUsage>,
}

impl ContextDiagram {
    /// Create a new ContextDiagram with the given system
    pub fn new(system_id: &str, title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DiagramKind::ContextDiagram,
            metadata: Metadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            system: System {
                id: system_id.to_string(),
                name: system_id.to_string(),
                description: None,
            },
            actors: Vec::new(),
            external_systems: Vec::new(),
            interfaces: Vec::new(),
            interface_providers: Vec::new(),
            interface_usages: Vec::new(),
        }
    }

    /// Derive relationships from interface usages and providers
    /// Rule: If A uses interface I, and system S provides I, then A -> S
    pub fn derive_relationships(&self) -> Vec<DerivedRelationship> {
        let mut relationships = Vec::new();

        // Build interface -> provider mapping
        let interface_to_provider: HashMap<String, &str> = self
            .interface_providers
            .iter()
            .flat_map(|p| {
                p.interfaces
                    .iter()
                    .map(|i| (i.clone(), p.system.as_str()))
            })
            .collect();

        // Iterate all interface usages
        for usage in &self.interface_usages {
            for interface_id in &usage.interfaces {
                if let Some(&provider_id) = interface_to_provider.get(interface_id) {
                    // Only create relationship if user is not the provider itself
                    if usage.actor != provider_id {
                        relationships.push(DerivedRelationship {
                            from: usage.actor.clone(),
                            to: provider_id.to_string(),
                            via_interface: interface_id.clone(),
                        });
                    }
                }
            }
        }

        relationships
    }

    /// Get all element IDs (for validation)
    pub fn all_element_ids(&self) -> Vec<&str> {
        let mut ids = vec![self.system.id.as_str()];
        ids.extend(self.actors.iter().map(|a| a.id.as_str()));
        ids.extend(self.external_systems.iter().map(|e| e.id.as_str()));
        ids
    }

    /// Find element name by ID
    pub fn get_element_name(&self, id: &str) -> Option<&str> {
        if self.system.id == id {
            return Some(&self.system.name);
        }
        if let Some(actor) = self.actors.iter().find(|a| a.id == id) {
            return Some(&actor.name);
        }
        if let Some(ext) = self.external_systems.iter().find(|e| e.id == id) {
            return Some(&ext.name);
        }
        None
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    ContextDiagram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

/// The core system (singleton)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// External actor (person)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, rename = "type")]
    pub actor_type: ActorType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    #[default]
    External,
    Internal,
}

/// External system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSystem {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

/// Interface definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub protocol: Protocol,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[default]
    Rest,
    Grpc,
    Graphql,
    WebSocket,
    Mqtt,
    Amqp,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

/// Interface provider: system -> interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceProvider {
    pub system: String,
    pub interfaces: Vec<String>,
}

/// Interface usage: actor/system -> interfaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceUsage {
    pub actor: String,
    pub interfaces: Vec<String>,
}

/// Derived relationship from interface usage
#[derive(Debug, Clone)]
pub struct DerivedRelationship {
    pub from: String,
    pub to: String,
    pub via_interface: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_relationships() {
        let mut diagram = ContextDiagram::new("TEST_SYSTEM", "Test System");

        // Add an actor
        diagram.actors.push(Actor {
            id: "user".to_string(),
            name: "User".to_string(),
            description: None,
            actor_type: ActorType::External,
        });

        // Add an interface
        diagram.interfaces.push(Interface {
            id: "API".to_string(),
            name: "API".to_string(),
            description: None,
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        // System provides the interface
        diagram.interface_providers.push(InterfaceProvider {
            system: "TEST_SYSTEM".to_string(),
            interfaces: vec!["API".to_string()],
        });

        // Actor uses the interface
        diagram.interface_usages.push(InterfaceUsage {
            actor: "user".to_string(),
            interfaces: vec!["API".to_string()],
        });

        let relationships = diagram.derive_relationships();
        assert_eq!(relationships.len(), 1);
        assert_eq!(relationships[0].from, "user");
        assert_eq!(relationships[0].to, "TEST_SYSTEM");
        assert_eq!(relationships[0].via_interface, "API");
    }
}
