use crate::model::c4::context::{
    Actor, ActorType, ContextDiagram, ExternalSystem, Interface, InterfaceProvider,
    InterfaceUsage, Protocol,
};
use crate::utils::error::{AppError, Result};

/// Operations for modifying ContextDiagram
pub struct Operations;

impl Operations {
    // ==================== Add Operations ====================

    /// Add or update the system
    /// Fails if system already exists with the same ID
    pub fn set_system(
        diagram: &mut ContextDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        // Check if system already exists with this ID
        if !diagram.system.id.is_empty() && diagram.system.id == id {
            return Err(AppError::ElementAlreadyExists(format!("system: {}", id)));
        }

        diagram.system.id = id.to_string();
        if let Some(n) = name {
            diagram.system.name = n.to_string();
        }
        if let Some(d) = description {
            diagram.system.description = Some(d.to_string());
        }
        diagram.touch();
        Ok(())
    }

    /// Add an actor
    pub fn add_actor(
        diagram: &mut ContextDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        actor_type: ActorType,
    ) -> Result<()> {
        // Check if actor already exists
        if diagram.actors.iter().any(|a| a.id == id) {
            return Err(AppError::ElementAlreadyExists(format!("actor: {}", id)));
        }

        // Check ID uniqueness against system and external systems
        Self::check_id_uniqueness(diagram, id)?;

        diagram.actors.push(Actor {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            actor_type,
        });
        diagram.touch();
        Ok(())
    }

    /// Add an external system
    pub fn add_external_system(
        diagram: &mut ContextDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        technology: Option<&str>,
    ) -> Result<()> {
        if diagram.external_systems.iter().any(|e| e.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "external_system: {}",
                id
            )));
        }

        Self::check_id_uniqueness(diagram, id)?;

        diagram.external_systems.push(ExternalSystem {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            technology: technology.map(|s| s.to_string()),
        });
        diagram.touch();
        Ok(())
    }

    /// Add an interface
    pub fn add_interface(
        diagram: &mut ContextDiagram,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
        protocol: Protocol,
    ) -> Result<()> {
        if diagram.interfaces.iter().any(|i| i.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "interface: {}",
                id
            )));
        }

        diagram.interfaces.push(Interface {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            protocol,
            endpoints: vec![],
        });
        diagram.touch();
        Ok(())
    }

    /// Add a provide relation (system provides interface)
    pub fn add_provide_relation(
        diagram: &mut ContextDiagram,
        system_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        // Validate system exists
        if diagram.system.id != system_id {
            // Check if it's an external system
            if !diagram.external_systems.iter().any(|e| e.id == system_id) {
                return Err(AppError::ElementNotFound(format!(
                    "system or external_system: {}",
                    system_id
                )));
            }
        }

        // Validate interface exists
        if !diagram.interfaces.iter().any(|i| i.id == interface_id) {
            return Err(AppError::ElementNotFound(format!(
                "interface: {}",
                interface_id
            )));
        }

        // Find or create provider entry
        if let Some(provider) = diagram
            .interface_providers
            .iter_mut()
            .find(|p| p.system == system_id)
        {
            if provider.interfaces.contains(&interface_id.to_string()) {
                return Err(AppError::ElementAlreadyExists(format!(
                    "provide_relation: {} -> {}",
                    system_id, interface_id
                )));
            }
            provider.interfaces.push(interface_id.to_string());
        } else {
            diagram.interface_providers.push(InterfaceProvider {
                system: system_id.to_string(),
                interfaces: vec![interface_id.to_string()],
            });
        }

        diagram.touch();
        Ok(())
    }

    /// Add an interface usage (actor/system uses interface)
    pub fn add_interface_usage(
        diagram: &mut ContextDiagram,
        actor_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        // Validate actor exists (could be actor, system, or external system)
        let is_valid_actor = diagram.actors.iter().any(|a| a.id == actor_id)
            || diagram.system.id == actor_id
            || diagram.external_systems.iter().any(|e| e.id == actor_id);

        if !is_valid_actor {
            return Err(AppError::ElementNotFound(format!(
                "actor or system: {}",
                actor_id
            )));
        }

        // Validate interface exists
        if !diagram.interfaces.iter().any(|i| i.id == interface_id) {
            return Err(AppError::ElementNotFound(format!(
                "interface: {}",
                interface_id
            )));
        }

        // Find or create usage entry
        if let Some(usage) = diagram
            .interface_usages
            .iter_mut()
            .find(|u| u.actor == actor_id)
        {
            if usage.interfaces.contains(&interface_id.to_string()) {
                return Err(AppError::ElementAlreadyExists(format!(
                    "interface_usage: {} -> {}",
                    actor_id, interface_id
                )));
            }
            usage.interfaces.push(interface_id.to_string());
        } else {
            diagram.interface_usages.push(InterfaceUsage {
                actor: actor_id.to_string(),
                interfaces: vec![interface_id.to_string()],
            });
        }

        diagram.touch();
        Ok(())
    }

    // ==================== Remove Operations ====================

    /// Remove an actor
    pub fn remove_actor(diagram: &mut ContextDiagram, id: &str) -> Result<()> {
        let idx = diagram
            .actors
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("actor: {}", id)))?;

        diagram.actors.remove(idx);

        // Remove related interface usages
        if let Some(idx) = diagram.interface_usages.iter().position(|u| u.actor == id) {
            diagram.interface_usages.remove(idx);
        }

        diagram.touch();
        Ok(())
    }

    /// Remove an external system
    pub fn remove_external_system(diagram: &mut ContextDiagram, id: &str) -> Result<()> {
        let idx = diagram
            .external_systems
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("external_system: {}", id)))?;

        diagram.external_systems.remove(idx);

        // Remove related interface providers
        if let Some(idx) = diagram
            .interface_providers
            .iter()
            .position(|p| p.system == id)
        {
            diagram.interface_providers.remove(idx);
        }

        // Remove related interface usages
        if let Some(idx) = diagram.interface_usages.iter().position(|u| u.actor == id) {
            diagram.interface_usages.remove(idx);
        }

        diagram.touch();
        Ok(())
    }

    /// Remove an interface
    pub fn remove_interface(diagram: &mut ContextDiagram, id: &str) -> Result<()> {
        let idx = diagram
            .interfaces
            .iter()
            .position(|i| i.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("interface: {}", id)))?;

        diagram.interfaces.remove(idx);

        // Remove from providers
        for provider in &mut diagram.interface_providers {
            provider.interfaces.retain(|i| i != id);
        }
        // Remove empty providers
        diagram
            .interface_providers
            .retain(|p| !p.interfaces.is_empty());

        // Remove from usages
        for usage in &mut diagram.interface_usages {
            usage.interfaces.retain(|i| i != id);
        }
        // Remove empty usages
        diagram.interface_usages.retain(|u| !u.interfaces.is_empty());

        diagram.touch();
        Ok(())
    }

    /// Remove a provide relation
    pub fn remove_provide_relation(
        diagram: &mut ContextDiagram,
        system_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        let provider = diagram
            .interface_providers
            .iter_mut()
            .find(|p| p.system == system_id)
            .ok_or_else(|| {
                AppError::ElementNotFound(format!("provider for system: {}", system_id))
            })?;

        let idx = provider
            .interfaces
            .iter()
            .position(|i| i == interface_id)
            .ok_or_else(|| {
                AppError::ElementNotFound(format!(
                    "provide_relation: {} -> {}",
                    system_id, interface_id
                ))
            })?;

        provider.interfaces.remove(idx);

        // Remove provider if empty
        if provider.interfaces.is_empty() {
            diagram
                .interface_providers
                .retain(|p| p.system != system_id || !p.interfaces.is_empty());
        }

        diagram.touch();
        Ok(())
    }

    /// Remove an interface usage
    pub fn remove_interface_usage(
        diagram: &mut ContextDiagram,
        actor_id: &str,
        interface_id: &str,
    ) -> Result<()> {
        let usage = diagram
            .interface_usages
            .iter_mut()
            .find(|u| u.actor == actor_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("usage for actor: {}", actor_id)))?;

        let idx = usage
            .interfaces
            .iter()
            .position(|i| i == interface_id)
            .ok_or_else(|| {
                AppError::ElementNotFound(format!(
                    "interface_usage: {} -> {}",
                    actor_id, interface_id
                ))
            })?;

        usage.interfaces.remove(idx);

        // Remove usage if empty
        if usage.interfaces.is_empty() {
            diagram.interface_usages.retain(|u| u.actor != actor_id);
        }

        diagram.touch();
        Ok(())
    }

    // ==================== Helper Functions ====================

    fn check_id_uniqueness(diagram: &ContextDiagram, id: &str) -> Result<()> {
        if diagram.system.id == id {
            return Err(AppError::InvalidOperation(format!(
                "ID '{}' conflicts with system ID",
                id
            )));
        }
        if diagram.actors.iter().any(|a| a.id == id) {
            return Err(AppError::InvalidOperation(format!(
                "ID '{}' conflicts with an existing actor",
                id
            )));
        }
        if diagram.external_systems.iter().any(|e| e.id == id) {
            return Err(AppError::InvalidOperation(format!(
                "ID '{}' conflicts with an existing external system",
                id
            )));
        }
        Ok(())
    }
}
