use crate::model::c4::context::{ActorType, ContextDiagram};

/// Generate PlantUML C4 Context diagram
pub fn generate_plantuml(diagram: &ContextDiagram) -> String {
    let mut output = String::new();

    // Header
    output.push_str("@startuml\n\n");
    output.push_str("skinparam defaultTextAlignment center\n\n");

    // Group external systems
    if !diagram.external_systems.is_empty() {
        for ext in &diagram.external_systems {
            output.push_str(&format!(
                "rectangle \"<<EXTERNAL_SYSTEM>>\\n{}\" as {}\n",
                ext.name, ext.id
            ));
        }
        output.push_str("\n");
    }

    // Group actors
    if !diagram.actors.is_empty() {
        for actor in &diagram.actors {
            let stereotype = match actor.actor_type {
                ActorType::External => "EXTERNAL_ACTOR",
                ActorType::Internal => "INTERNAL_ACTOR",
            };
            output.push_str(&format!(
                "actor \"<<{}>>\\n{}\" as {}\n",
                stereotype, actor.name, actor.id
            ));
        }
        output.push_str("\n");
    }

    // Group interfaces (declared before system)
    for iface in &diagram.interfaces {
        output.push_str(&format!("interface {}\n", iface.id));
    }
    output.push_str("\n");

    // System (center)
    output.push_str(&format!(
        "rectangle \"<<SYSTEM>>\\n{}\" as {}\n",
        diagram.system.name, diagram.system.id
    ));
    output.push_str("\n");

    // Relationships
    // 1. Actor/System uses interface: USER ..> INTERFACE
    for usage in &diagram.interface_usages {
        for iface_id in &usage.interfaces {
            output.push_str(&format!("{} ..> {}\n", usage.actor, iface_id));
        }
    }
    output.push_str("\n");

    // 2. Interface provided by system: INTERFACE --- SYSTEM
    for provider in &diagram.interface_providers {
        for iface_id in &provider.interfaces {
            output.push_str(&format!("{} --- {}\n", iface_id, provider.system));
        }
    }
    output.push_str("\n");

    // Footer
    output.push_str("@enduml\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::c4::context::{Actor, Interface, InterfaceProvider, InterfaceUsage, Protocol};

    #[test]
    fn test_generate_plantuml() {
        let mut diagram = ContextDiagram::new("TEST_SYSTEM", "Test System");
        diagram.system.name = "Test System".to_string();
        diagram.system.description = Some("A test system".to_string());

        diagram.actors.push(Actor {
            id: "user".to_string(),
            name: "User".to_string(),
            description: Some("A test user".to_string()),
            actor_type: ActorType::External,
        });

        diagram.interfaces.push(Interface {
            id: "API".to_string(),
            name: "API".to_string(),
            description: Some("The API".to_string()),
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        diagram.interface_providers.push(InterfaceProvider {
            system: "TEST_SYSTEM".to_string(),
            interfaces: vec!["API".to_string()],
        });

        diagram.interface_usages.push(InterfaceUsage {
            actor: "user".to_string(),
            interfaces: vec!["API".to_string()],
        });

        let output = generate_plantuml(&diagram);

        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("<<SYSTEM>>"));
        assert!(output.contains("rectangle \"<<SYSTEM>>\\nTest System\" as TEST_SYSTEM"));
        assert!(output.contains("actor \"<<EXTERNAL_ACTOR>>\\nUser\" as user"));
        assert!(output.contains("interface API"));
        assert!(output.contains("user ..> API"));
        assert!(output.contains("API --- TEST_SYSTEM"));
    }
}
