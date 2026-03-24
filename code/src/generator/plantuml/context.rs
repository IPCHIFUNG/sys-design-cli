use crate::model::c4::context::{ActorType, ContextDiagram, Protocol};

/// Generate PlantUML C4 Context diagram
pub fn generate_plantuml(diagram: &ContextDiagram) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n");
    output.push_str("!include C4_Context.puml\n");
    output.push_str("\n");

    // Title
    output.push_str(&format!("title {}\n", diagram.metadata.title));
    output.push_str("\n");

    // System (center)
    output.push_str(&format!(
        "System({}, \"{}\", \"{}\")\n",
        diagram.system.id,
        diagram.system.name,
        diagram.system.description.as_deref().unwrap_or("")
    ));
    output.push_str("\n");

    // Actors
    if !diagram.actors.is_empty() {
        output.push_str("' Actors\n");
        for actor in &diagram.actors {
            let typ = match actor.actor_type {
                ActorType::External => "Person_Ext",
                ActorType::Internal => "Person",
            };
            output.push_str(&format!(
                "{}({}, \"{}\", \"{}\")\n",
                typ,
                actor.id,
                actor.name,
                actor.description.as_deref().unwrap_or("")
            ));
        }
        output.push_str("\n");
    }

    // External Systems
    if !diagram.external_systems.is_empty() {
        output.push_str("' External Systems\n");
        for ext in &diagram.external_systems {
            output.push_str(&format!(
                "System_Ext({}, \"{}\", \"{}\")\n",
                ext.id,
                ext.name,
                ext.description.as_deref().unwrap_or("")
            ));
        }
        output.push_str("\n");
    }

    // Interfaces (as notes or stereotypes)
    if !diagram.interfaces.is_empty() {
        output.push_str("' Interfaces\n");
        for iface in &diagram.interfaces {
            let protocol_str = match &iface.protocol {
                Protocol::Rest => "REST",
                Protocol::Grpc => "gRPC",
                Protocol::Graphql => "GraphQL",
                Protocol::WebSocket => "WebSocket",
                Protocol::Mqtt => "MQTT",
                Protocol::Amqp => "AMQP",
                Protocol::Custom(s) => s,
            };

            // Find provider and users
            let provider = diagram
                .interface_providers
                .iter()
                .find(|p| p.interfaces.contains(&iface.id))
                .map(|p| p.system.as_str());

            let users: Vec<&str> = diagram
                .interface_usages
                .iter()
                .filter(|u| u.interfaces.contains(&iface.id))
                .map(|u| u.actor.as_str())
                .collect();

            if let Some(provider_id) = provider {
                output.push_str(&format!(
                    "note right of \"{}\" \"Interface: {}\\nProtocol: {}\\nUsed by: {}\"\n",
                    provider_id,
                    iface.name,
                    protocol_str,
                    if users.is_empty() {
                        "none".to_string()
                    } else {
                        users.join(", ")
                    }
                ));
                output.push_str("end note\n");
            }
        }
        output.push_str("\n");
    }

    // Relationships (derived)
    output.push_str("' Relationships (derived from interface usages)\n");
    let relationships = diagram.derive_relationships();
    for rel in relationships {
        // Get interface name for label
        let iface_name = diagram
            .interfaces
            .iter()
            .find(|i| i.id == rel.via_interface)
            .map(|i| i.name.as_str())
            .unwrap_or(&rel.via_interface);

        output.push_str(&format!(
            "Rel({}, {}, \"uses\", \"{}\")\n",
            rel.from, rel.to, iface_name
        ));
    }
    output.push_str("\n");

    // Layout
    output.push_str("LAYOUT_WITH_LEGEND()\n");

    // Footer
    output.push_str("@enduml\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::c4::context::{Actor, Interface, InterfaceProvider, InterfaceUsage};

    #[test]
    fn test_generate_plantuml() {
        let mut diagram = ContextDiagram::new("test-system", "Test System");
        diagram.system.name = "Test System".to_string();
        diagram.system.description = Some("A test system".to_string());

        diagram.actors.push(Actor {
            id: "user".to_string(),
            name: "User".to_string(),
            description: Some("A test user".to_string()),
            actor_type: ActorType::External,
        });

        diagram.interfaces.push(Interface {
            id: "api".to_string(),
            name: "API".to_string(),
            description: Some("The API".to_string()),
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        diagram.interface_providers.push(InterfaceProvider {
            system: "test-system".to_string(),
            interfaces: vec!["api".to_string()],
        });

        diagram.interface_usages.push(InterfaceUsage {
            actor: "user".to_string(),
            interfaces: vec!["api".to_string()],
        });

        let output = generate_plantuml(&diagram);

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("System(test-system"));
        assert!(output.contains("Person_Ext(user"));
        assert!(output.contains("Rel(user, test-system"));
    }
}
