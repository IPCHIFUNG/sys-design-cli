//! Rust 单元测试
//!
//! 运行方式: cargo test
//!
//! 这些测试覆盖:
//! - 数据模型创建和序列化
//! - 关系推导逻辑
//! - 验证规则
//! - PlantUML 生成

#[cfg(test)]
mod tests {
    use sys_design::model::c4::context::{
        ContextDiagram, Actor, ActorType, ExternalSystem, Interface,
        InterfaceProvider, InterfaceUsage, Protocol,
    };
    use sys_design::store::Operations;
    use sys_design::validator::{validate, Severity};
    use sys_design::generator::plantuml::generate_plantuml;

    /// 创建测试用的 ContextDiagram
    fn create_test_diagram() -> ContextDiagram {
        let mut diagram = ContextDiagram::new("test-system", "Test System Diagram");

        // 添加角色
        diagram.actors.push(Actor {
            id: "user".to_string(),
            name: "User".to_string(),
            description: Some("A test user".to_string()),
            actor_type: ActorType::External,
        });

        diagram.actors.push(Actor {
            id: "admin".to_string(),
            name: "Admin".to_string(),
            description: Some("System administrator".to_string()),
            actor_type: ActorType::Internal,
        });

        // 添加外部系统
        diagram.external_systems.push(ExternalSystem {
            id: "payment-gateway".to_string(),
            name: "Payment Gateway".to_string(),
            description: Some("External payment service".to_string()),
            technology: Some("REST API".to_string()),
        });

        // 添加接口
        diagram.interfaces.push(Interface {
            id: "user-api".to_string(),
            name: "User API".to_string(),
            description: Some("API for user operations".to_string()),
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        diagram.interfaces.push(Interface {
            id: "payment-api".to_string(),
            name: "Payment API".to_string(),
            description: Some("API for payment operations".to_string()),
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        // 设置接口提供关系
        diagram.interface_providers.push(InterfaceProvider {
            system: "test-system".to_string(),
            interfaces: vec!["user-api".to_string()],
        });

        diagram.interface_providers.push(InterfaceProvider {
            system: "payment-gateway".to_string(),
            interfaces: vec!["payment-api".to_string()],
        });

        // 设置接口使用关系
        diagram.interface_usages.push(InterfaceUsage {
            actor: "user".to_string(),
            interfaces: vec!["user-api".to_string()],
        });

        diagram.interface_usages.push(InterfaceUsage {
            actor: "admin".to_string(),
            interfaces: vec!["user-api".to_string()],
        });

        diagram.interface_usages.push(InterfaceUsage {
            actor: "test-system".to_string(),
            interfaces: vec!["payment-api".to_string()],
        });

        diagram
    }

    // ==================== 模型测试 ====================

    #[test]
    fn test_create_diagram() {
        let diagram = ContextDiagram::new("my-system", "My System");
        assert_eq!(diagram.system.id, "my-system");
        assert_eq!(diagram.system.name, "my-system");
        assert_eq!(diagram.metadata.title, "My System");
        assert!(diagram.actors.is_empty());
        assert!(diagram.external_systems.is_empty());
        assert!(diagram.interfaces.is_empty());
    }

    #[test]
    fn test_derive_relationships() {
        let diagram = create_test_diagram();
        let relationships = diagram.derive_relationships();

        // 应该有 3 个关系
        assert_eq!(relationships.len(), 3);

        // user -> test-system via user-api
        assert!(relationships.iter().any(|r|
            r.from == "user" && r.to == "test-system" && r.via_interface == "user-api"
        ));

        // admin -> test-system via user-api
        assert!(relationships.iter().any(|r|
            r.from == "admin" && r.to == "test-system" && r.via_interface == "user-api"
        ));

        // test-system -> payment-gateway via payment-api
        assert!(relationships.iter().any(|r|
            r.from == "test-system" && r.to == "payment-gateway" && r.via_interface == "payment-api"
        ));
    }

    #[test]
    fn test_get_element_name() {
        let diagram = create_test_diagram();

        assert_eq!(diagram.get_element_name("test-system"), Some("test-system"));
        assert_eq!(diagram.get_element_name("user"), Some("User"));
        assert_eq!(diagram.get_element_name("payment-gateway"), Some("Payment Gateway"));
        assert_eq!(diagram.get_element_name("nonexistent"), None);
    }

    #[test]
    fn test_all_element_ids() {
        let diagram = create_test_diagram();
        let ids = diagram.all_element_ids();

        assert!(ids.contains(&"test-system"));
        assert!(ids.contains(&"user"));
        assert!(ids.contains(&"admin"));
        assert!(ids.contains(&"payment-gateway"));
    }

    // ==================== 序列化测试 ====================

    #[test]
    fn test_yaml_serialization() {
        let diagram = create_test_diagram();
        let yaml = serde_yaml::to_string(&diagram).unwrap();

        assert!(yaml.contains("test-system"));
        assert!(yaml.contains("user-api"));
        assert!(yaml.contains("payment-gateway"));
    }

    #[test]
    fn test_yaml_deserialization() {
        let original = create_test_diagram();
        let yaml = serde_yaml::to_string(&original).unwrap();
        let parsed: ContextDiagram = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.system.id, original.system.id);
        assert_eq!(parsed.actors.len(), original.actors.len());
        assert_eq!(parsed.external_systems.len(), original.external_systems.len());
        assert_eq!(parsed.interfaces.len(), original.interfaces.len());
    }

    // ==================== 验证器测试 ====================

    #[test]
    fn test_validate_valid_diagram() {
        let diagram = create_test_diagram();
        let result = validate(&diagram);

        assert!(result.is_valid);
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_validate_empty_id() {
        let diagram = ContextDiagram::new("", "Test");
        let result = validate(&diagram);

        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e|
            e.code == "C001" && e.severity == Severity::Error
        ));
    }

    #[test]
    fn test_validate_orphan_interface() {
        let mut diagram = ContextDiagram::new("test", "Test");

        // 添加一个没有使用者的接口
        diagram.interfaces.push(Interface {
            id: "unused-api".to_string(),
            name: "Unused API".to_string(),
            description: None,
            protocol: Protocol::Rest,
            endpoints: vec![],
        });

        let result = validate(&diagram);

        // 应该有警告
        assert!(result.errors.iter().any(|e|
            e.code == "S002" && e.severity == Severity::Warning
        ));
    }

    // ==================== PlantUML 生成测试 ====================

    #[test]
    fn test_generate_plantuml() {
        let diagram = create_test_diagram();
        let plantuml = generate_plantuml(&diagram);

        assert!(plantuml.contains("@startuml"));
        assert!(plantuml.contains("@enduml"));
        assert!(plantuml.contains("System(test-system"));
        assert!(plantuml.contains("Person_Ext(user"));
        assert!(plantuml.contains("Person(admin"));
        assert!(plantuml.contains("System_Ext(payment-gateway"));
    }

    #[test]
    fn test_generate_plantuml_relationships() {
        let diagram = create_test_diagram();
        let plantuml = generate_plantuml(&diagram);

        // 检查关系是否生成
        assert!(plantuml.contains("Rel(user, test-system"));
        assert!(plantuml.contains("Rel(admin, test-system"));
        assert!(plantuml.contains("Rel(test-system, payment-gateway"));
    }

    #[test]
    fn test_generate_plantuml_layout() {
        let diagram = create_test_diagram();
        let plantuml = generate_plantuml(&diagram);

        assert!(plantuml.contains("LAYOUT_WITH_LEGEND"));
    }

    // ==================== 操作测试 ====================

    #[test]
    fn test_add_actor() {
        let mut diagram = ContextDiagram::new("test", "Test");

        Operations::add_actor(
            &mut diagram,
            "user",
            Some("Test User"),
            Some("A test user"),
            ActorType::External,
        ).unwrap();

        assert_eq!(diagram.actors.len(), 1);
        assert_eq!(diagram.actors[0].id, "user");
        assert_eq!(diagram.actors[0].name, "Test User");
    }

    #[test]
    fn test_add_duplicate_actor() {
        let mut diagram = ContextDiagram::new("test", "Test");

        Operations::add_actor(
            &mut diagram,
            "user",
            Some("User 1"),
            None,
            ActorType::External,
        ).unwrap();

        // 添加重复的 actor 应该失败
        let result = Operations::add_actor(
            &mut diagram,
            "user",
            Some("User 2"),
            None,
            ActorType::External,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_add_external_system() {
        let mut diagram = ContextDiagram::new("test", "Test");

        Operations::add_external_system(
            &mut diagram,
            "db",
            Some("Database"),
            Some("PostgreSQL database"),
            Some("PostgreSQL"),
        ).unwrap();

        assert_eq!(diagram.external_systems.len(), 1);
        assert_eq!(diagram.external_systems[0].id, "db");
        assert_eq!(diagram.external_systems[0].technology, Some("PostgreSQL".to_string()));
    }

    #[test]
    fn test_add_interface() {
        let mut diagram = ContextDiagram::new("test", "Test");

        Operations::add_interface(
            &mut diagram,
            "api",
            Some("REST API"),
            Some("Main API"),
            Protocol::Rest,
        ).unwrap();

        assert_eq!(diagram.interfaces.len(), 1);
        assert_eq!(diagram.interfaces[0].id, "api");
    }

    #[test]
    fn test_remove_actor() {
        let mut diagram = create_test_diagram();

        Operations::remove_actor(&mut diagram, "user").unwrap();

        assert_eq!(diagram.actors.len(), 1);
        assert_eq!(diagram.actors[0].id, "admin");

        // 接口使用也应该被移除
        assert!(!diagram.interface_usages.iter().any(|u| u.actor == "user"));
    }
}
