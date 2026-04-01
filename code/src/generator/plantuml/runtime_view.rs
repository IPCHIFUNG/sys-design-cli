use crate::model::runtime::{
    Block, GroupType, NotePosition, ParticipantType, RuntimeView, StepType,
};
use crate::model::workspace::Workspace;

/// Generate PlantUML sequence diagram for a runtime view scenario
pub fn generate_runtime_plantuml(
    workspace: &Workspace,
    view: &RuntimeView,
    scenario_id: &str,
) -> String {
    let scenario = match view.find_scenario(scenario_id) {
        Some(s) => s,
        None => {
            let mut output = String::new();
            output.push_str("@startuml\n\n");
            output.push_str(&format!(
                "' ERROR: Scenario '{}' not found in runtime_view\n",
                scenario_id
            ));
            output.push_str(&format!(
                "' Available scenarios: {}\n",
                view.scenarios
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            output.push_str("\n@enduml\n");
            return output;
        }
    };

    let mut output = String::new();
    output.push_str("@startuml\n\n");
    output.push_str("autonumber\n\n");

    // Declare participants
    for participant in &scenario.participants {
        let display_name = resolve_element_name(workspace, &participant.element_id)
            .unwrap_or(&participant.element_id);

        let keyword = participant_type_to_plantuml(&participant.participant_type);
        let name = participant
            .alias
            .as_deref()
            .unwrap_or(display_name);

        output.push_str(&format!(
            "{} \"{}\" as {}",
            keyword, name, participant.element_id
        ));

        if let Some(ref color) = participant.color {
            output.push_str(&format!(" #{}", color));
        }

        output.push_str("\n");
    }

    output.push_str("\n");

    // Generate blocks
    generate_blocks(&mut output, &scenario.blocks);

    // Generate notes
    for note in &scenario.notes {
        let position = match note.position {
            NotePosition::Left => "left of",
            NotePosition::Right => "right of",
            NotePosition::Over => "over",
        };
        output.push_str(&format!("note {} {}\n", position, note.target));
        output.push_str(&format!("  {}\n", note.text));
        output.push_str("end note\n");
    }

    // Generate dividers
    for divider in &scenario.dividers {
        output.push_str(&format!("== {} ==\n", divider.label));
    }

    output.push_str("\n@enduml\n");
    output
}

fn generate_blocks(output: &mut String, blocks: &[Block]) {
    for block in blocks {
        match block {
            Block::Step(step) => {
                let arrow = step_type_to_arrow(&step.step_type);
                let from = &step.from;
                let to = &step.to;

                // Self-call detection
                let line = if from == to {
                    format!("{} -> {} : {}\n", from, to, step.message)
                } else {
                    match &step.color {
                        Some(color) => {
                            format!("{} -[{}]> {} : {}\n", from, color, to, step.message)
                        }
                        None => format!("{} {} {} : {}\n", from, arrow, to, step.message),
                    }
                };
                output.push_str(&line);

                if step.activate_target == Some(true) {
                    output.push_str(&format!("activate {}\n", to));
                }
            }
            Block::Group(group) => {
                let keyword = group_type_to_keyword(&group.group_type);

                if group.group_type == GroupType::Alt && !group.branches.is_empty() {
                    // Alt with branches
                    output.push_str(&format!("{} {}\n", keyword, group.label));
                    for (i, branch) in group.branches.iter().enumerate() {
                        if i > 0 {
                            output.push_str(&format!("else {}\n", branch.label));
                        }
                        generate_blocks(output, &branch.blocks);
                    }
                    output.push_str("end\n");
                } else {
                    // Non-alt or alt without branches (use blocks directly)
                    output.push_str(&format!("{} {}\n", keyword, group.label));
                    generate_blocks(output, &group.blocks);
                    output.push_str("end\n");
                }
            }
        }
    }
}

fn participant_type_to_plantuml(pt: &ParticipantType) -> &'static str {
    match pt {
        ParticipantType::Actor => "actor",
        ParticipantType::Participant => "participant",
        ParticipantType::Boundary => "boundary",
        ParticipantType::Control => "control",
        ParticipantType::Entity => "entity",
        ParticipantType::Database => "database",
        ParticipantType::Collections => "collections",
        ParticipantType::Queue => "queue",
    }
}

fn step_type_to_arrow(st: &StepType) -> &'static str {
    match st {
        StepType::Sync => "->",
        StepType::Async => "->>",
        StepType::Return => "-->",
        StepType::Lost => "->o",
    }
}

fn group_type_to_keyword(gt: &GroupType) -> &'static str {
    match gt {
        GroupType::Alt => "alt",
        GroupType::Opt => "opt",
        GroupType::Loop => "loop",
        GroupType::Par => "par",
        GroupType::Break => "break",
        GroupType::Critical => "critical",
        GroupType::Group => "group",
    }
}

/// Resolve element name from workspace (context diagram or logic view)
fn resolve_element_name<'a>(workspace: &'a Workspace, element_id: &str) -> Option<&'a str> {
    // Try context diagram first
    if let Some(ctx) = &workspace.context_diagram {
        if let Some(name) = ctx.get_element_name(element_id) {
            return Some(name);
        }
    }

    // Try logic view
    if let Some(lv) = &workspace.logic_view {
        if let Some(name) = lv.get_element_name(element_id) {
            return Some(name);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::runtime::*;

    fn create_test_view() -> RuntimeView {
        RuntimeView::new("Test")
    }

    fn create_test_workspace() -> Workspace {
        let mut ws = Workspace::new("Test");
        ws.context_diagram = Some(crate::model::c4::context::ContextDiagram::new(
            "MY_SYSTEM", "My System",
        ));
        ws
    }

    #[test]
    fn test_generate_basic_sequence() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test Scenario");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "B".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.blocks.push(Block::Step(Step {
            order: 1,
            from: "A".to_string(),
            to: "B".to_string(),
            message: "hello".to_string(),
            step_type: StepType::Sync,
            protocol: None,
            color: None,
            activate_target: None,
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("@startuml"));
        assert!(result.contains("@enduml"));
        assert!(result.contains("participant \"A\" as A"));
        assert!(result.contains("A -> B : hello"));
        assert!(result.contains("autonumber"));
    }

    #[test]
    fn test_generate_self_call() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.blocks.push(Block::Step(Step {
            order: 1,
            from: "A".to_string(),
            to: "A".to_string(),
            message: "process".to_string(),
            step_type: StepType::Sync,
            protocol: None,
            color: None,
            activate_target: None,
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");
        assert!(result.contains("A -> A : process"));
    }

    #[test]
    fn test_generate_all_step_types() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "B".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });

        scenario.blocks.push(Block::Step(Step {
            order: 1, from: "A".into(), to: "B".into(), message: "sync".into(),
            step_type: StepType::Sync, protocol: None, color: None, activate_target: None,
        }));
        scenario.blocks.push(Block::Step(Step {
            order: 2, from: "A".into(), to: "B".into(), message: "async".into(),
            step_type: StepType::Async, protocol: None, color: None, activate_target: None,
        }));
        scenario.blocks.push(Block::Step(Step {
            order: 3, from: "B".into(), to: "A".into(), message: "ret".into(),
            step_type: StepType::Return, protocol: None, color: None, activate_target: None,
        }));
        scenario.blocks.push(Block::Step(Step {
            order: 4, from: "A".into(), to: "B".into(), message: "lost".into(),
            step_type: StepType::Lost, protocol: None, color: None, activate_target: None,
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("A -> B : sync"));
        assert!(result.contains("A ->> B : async"));
        assert!(result.contains("B --> A : ret"));
        assert!(result.contains("A ->o B : lost"));
    }

    #[test]
    fn test_generate_alt_group() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "B".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.blocks.push(Block::Group(Group {
            group_type: GroupType::Alt,
            label: "result".to_string(),
            blocks: Vec::new(),
            branches: vec![
                GroupBranch {
                    label: "ok".to_string(),
                    blocks: vec![Block::Step(Step {
                        order: 1, from: "B".into(), to: "A".into(), message: "token".into(),
                        step_type: StepType::Return, protocol: None, color: None, activate_target: None,
                    })],
                },
                GroupBranch {
                    label: "err".to_string(),
                    blocks: vec![Block::Step(Step {
                        order: 2, from: "B".into(), to: "A".into(), message: "error".into(),
                        step_type: StepType::Return, protocol: None, color: None, activate_target: None,
                    })],
                },
            ],
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("alt result"));
        assert!(result.contains("B --> A : token"));
        assert!(result.contains("else err"));
        assert!(result.contains("B --> A : error"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_generate_loop_group() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.blocks.push(Block::Group(Group {
            group_type: GroupType::Loop,
            label: "retry".to_string(),
            blocks: vec![Block::Step(Step {
                order: 1, from: "A".into(), to: "A".into(), message: "retry".into(),
                step_type: StepType::Sync, protocol: None, color: None, activate_target: None,
            })],
            branches: Vec::new(),
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("loop retry"));
        assert!(result.contains("A -> A : retry"));
        assert!(result.contains("end"));
    }

    #[test]
    fn test_generate_nested_groups() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "B".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });

        let inner_loop = Block::Group(Group {
            group_type: GroupType::Loop,
            label: "retry".to_string(),
            blocks: vec![Block::Step(Step {
                order: 3, from: "A".into(), to: "B".into(), message: "retry".into(),
                step_type: StepType::Sync, protocol: None, color: None, activate_target: None,
            })],
            branches: Vec::new(),
        });

        scenario.blocks.push(Block::Group(Group {
            group_type: GroupType::Alt,
            label: "result".to_string(),
            blocks: Vec::new(),
            branches: vec![
                GroupBranch {
                    label: "ok".to_string(),
                    blocks: vec![Block::Step(Step {
                        order: 1, from: "B".into(), to: "A".into(), message: "ok".into(),
                        step_type: StepType::Return, protocol: None, color: None, activate_target: None,
                    })],
                },
                GroupBranch {
                    label: "err".to_string(),
                    blocks: vec![inner_loop],
                },
            ],
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("alt result"));
        assert!(result.contains("loop retry"));
        assert!(result.contains("end")); // loop end
        assert!(result.contains("end")); // alt end
    }

    #[test]
    fn test_generate_participant_types() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "U".to_string(),
            participant_type: ParticipantType::Actor,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "D".to_string(),
            participant_type: ParticipantType::Database,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "Q".to_string(),
            participant_type: ParticipantType::Queue,
            alias: None,
            color: None,
        });
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");

        assert!(result.contains("actor \"U\" as U"));
        assert!(result.contains("database \"D\" as D"));
        assert!(result.contains("queue \"Q\" as Q"));
    }

    #[test]
    fn test_generate_nonexistent_scenario() {
        let view = create_test_view();
        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "NOPE");
        assert!(result.contains("ERROR"));
        assert!(result.contains("NOPE"));
    }

    #[test]
    fn test_generate_note() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.notes.push(Note {
            position: NotePosition::Right,
            target: "A".to_string(),
            text: "Important".to_string(),
        });
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");
        assert!(result.contains("note right of A"));
        assert!(result.contains("Important"));
        assert!(result.contains("end note"));
    }

    #[test]
    fn test_generate_activate() {
        let mut view = create_test_view();
        let mut scenario = Scenario::new("S1", "Test");
        scenario.participants.push(Participant {
            element_id: "A".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.participants.push(Participant {
            element_id: "B".to_string(),
            participant_type: ParticipantType::Participant,
            alias: None,
            color: None,
        });
        scenario.blocks.push(Block::Step(Step {
            order: 1, from: "A".into(), to: "B".into(), message: "call".into(),
            step_type: StepType::Sync, protocol: None, color: None, activate_target: Some(true),
        }));
        view.scenarios.push(scenario);

        let ws = create_test_workspace();
        let result = generate_runtime_plantuml(&ws, &view, "S1");
        assert!(result.contains("activate B"));
    }
}
