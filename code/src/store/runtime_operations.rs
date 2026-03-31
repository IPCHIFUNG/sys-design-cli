use crate::model::runtime::{
    Block, Divider, Group, GroupBranch, GroupType, Note, NotePosition, Participant, ParticipantType,
    RuntimeView, Scenario, Step, StepType,
};
use crate::utils::error::{AppError, Result};

/// Operations for modifying RuntimeView
pub struct RuntimeOperations;

impl RuntimeOperations {
    // ==================== Add Operations ====================

    /// Add a new scenario
    pub fn add_scenario(
        view: &mut RuntimeView,
        id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        if view.scenarios.iter().any(|s| s.id == id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "scenario: {}",
                id
            )));
        }

        view.scenarios.push(Scenario {
            id: id.to_string(),
            name: name.unwrap_or(id).to_string(),
            description: description.map(|s| s.to_string()),
            participants: Vec::new(),
            blocks: Vec::new(),
            notes: Vec::new(),
            dividers: Vec::new(),
        });
        view.touch();
        Ok(())
    }

    /// Add a participant to a scenario
    pub fn add_participant(
        view: &mut RuntimeView,
        scenario_id: &str,
        element_id: &str,
        participant_type: ParticipantType,
        alias: Option<&str>,
        color: Option<&str>,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        if scenario.participants.iter().any(|p| p.element_id == element_id) {
            return Err(AppError::ElementAlreadyExists(format!(
                "participant: {} in scenario: {}",
                element_id, scenario_id
            )));
        }

        scenario.participants.push(Participant {
            element_id: element_id.to_string(),
            participant_type,
            alias: alias.map(|s| s.to_string()),
            color: color.map(|s| s.to_string()),
        });
        view.touch();
        Ok(())
    }

    /// Add a step to a scenario (optionally into a group/branch)
    pub fn add_step(
        view: &mut RuntimeView,
        scenario_id: &str,
        from: &str,
        to: &str,
        message: &str,
        step_type: StepType,
        protocol: Option<&str>,
        color: Option<&str>,
        activate_target: Option<bool>,
        group_label: Option<&str>,
        branch_label: Option<&str>,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        let order = scenario.max_step_order() + 1;

        let target_blocks = scenario
            .get_target_blocks_mut(group_label, branch_label)
            .ok_or_else(|| {
                if let Some(gl) = group_label {
                    if let Some(bl) = branch_label {
                        AppError::ElementNotFound(format!(
                            "group '{}' branch '{}' in scenario: {}",
                            gl, bl, scenario_id
                        ))
                    } else {
                        AppError::InvalidOperation(format!(
                            "cannot add step to alt group '{}' without specifying a branch",
                            gl
                        ))
                    }
                } else {
                    AppError::ElementNotFound(format!("scenario: {}", scenario_id))
                }
            })?;

        target_blocks.push(Block::Step(Step {
            order,
            from: from.to_string(),
            to: to.to_string(),
            message: message.to_string(),
            step_type,
            protocol: protocol.map(|s| s.to_string()),
            color: color.map(|s| s.to_string()),
            activate_target,
        }));
        view.touch();
        Ok(())
    }

    /// Add a group to a scenario (optionally into a parent group/branch)
    pub fn add_group(
        view: &mut RuntimeView,
        scenario_id: &str,
        group_type: GroupType,
        label: &str,
        branches: Vec<String>,
        parent_group_label: Option<&str>,
        parent_branch_label: Option<&str>,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        let target_blocks = scenario
            .get_target_blocks_mut(parent_group_label, parent_branch_label)
            .ok_or_else(|| {
                if let Some(gl) = parent_group_label {
                    if let Some(bl) = parent_branch_label {
                        AppError::ElementNotFound(format!(
                            "group '{}' branch '{}' in scenario: {}",
                            gl, bl, scenario_id
                        ))
                    } else {
                        AppError::InvalidOperation(format!(
                            "cannot add group to alt group '{}' without specifying a branch",
                            gl
                        ))
                    }
                } else {
                    AppError::ElementNotFound(format!("scenario: {}", scenario_id))
                }
            })?;

        let group_branches: Vec<GroupBranch> = branches
            .into_iter()
            .map(|b| GroupBranch {
                label: b,
                blocks: Vec::new(),
            })
            .collect();

        target_blocks.push(Block::Group(Group {
            group_type,
            label: label.to_string(),
            blocks: if group_branches.is_empty() {
                Vec::new()
            } else {
                Vec::new()
            },
            branches: group_branches,
        }));
        view.touch();
        Ok(())
    }

    /// Add a note to a scenario
    pub fn add_note(
        view: &mut RuntimeView,
        scenario_id: &str,
        position: NotePosition,
        target: &str,
        text: &str,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        scenario.notes.push(Note {
            position,
            target: target.to_string(),
            text: text.to_string(),
        });
        view.touch();
        Ok(())
    }

    /// Add a divider to a scenario
    pub fn add_divider(
        view: &mut RuntimeView,
        scenario_id: &str,
        label: &str,
        after_order: u32,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        scenario.dividers.push(Divider {
            label: label.to_string(),
            after_order,
        });
        view.touch();
        Ok(())
    }

    // ==================== Remove Operations ====================

    /// Remove a scenario
    pub fn remove_scenario(view: &mut RuntimeView, id: &str) -> Result<()> {
        let idx = view
            .scenarios
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", id)))?;

        view.scenarios.remove(idx);
        view.touch();
        Ok(())
    }

    /// Remove a participant and cascade-remove referencing steps
    pub fn remove_participant(
        view: &mut RuntimeView,
        scenario_id: &str,
        element_id: &str,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        let idx = scenario
            .participants
            .iter()
            .position(|p| p.element_id == element_id)
            .ok_or_else(|| AppError::ElementNotFound(format!(
                "participant: {} in scenario: {}",
                element_id, scenario_id
            )))?;

        scenario.participants.remove(idx);

        // Cascade-remove steps referencing this participant
        Self::remove_steps_referencing_participant(&mut scenario.blocks, element_id);

        view.touch();
        Ok(())
    }

    /// Remove a step by order from specified location
    pub fn remove_step(
        view: &mut RuntimeView,
        scenario_id: &str,
        order: u32,
        group_label: Option<&str>,
        branch_label: Option<&str>,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        let target_blocks = scenario
            .get_target_blocks_mut(group_label, branch_label)
            .ok_or_else(|| AppError::ElementNotFound(format!(
                "target location in scenario: {}",
                scenario_id
            )))?;

        let idx = target_blocks
            .iter()
            .position(|b| matches!(b, Block::Step(s) if s.order == order))
            .ok_or_else(|| AppError::ElementNotFound(format!(
                "step with order {} in scenario: {}",
                order, scenario_id
            )))?;

        target_blocks.remove(idx);
        view.touch();
        Ok(())
    }

    /// Remove a group by label (cascades to entire subtree)
    pub fn remove_group(
        view: &mut RuntimeView,
        scenario_id: &str,
        label: &str,
        parent_group_label: Option<&str>,
        parent_branch_label: Option<&str>,
    ) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        let target_blocks = scenario
            .get_target_blocks_mut(parent_group_label, parent_branch_label)
            .ok_or_else(|| AppError::ElementNotFound(format!(
                "target location in scenario: {}",
                scenario_id
            )))?;

        let idx = target_blocks
            .iter()
            .position(|b| matches!(b, Block::Group(g) if g.label == label))
            .ok_or_else(|| AppError::ElementNotFound(format!(
                "group '{}' in scenario: {}",
                label, scenario_id
            )))?;

        target_blocks.remove(idx);
        view.touch();
        Ok(())
    }

    /// Remove a note by index
    pub fn remove_note(view: &mut RuntimeView, scenario_id: &str, index: usize) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        if index >= scenario.notes.len() {
            return Err(AppError::ElementNotFound(format!(
                "note index {} in scenario: {}",
                index, scenario_id
            )));
        }

        scenario.notes.remove(index);
        view.touch();
        Ok(())
    }

    /// Remove a divider by index
    pub fn remove_divider(view: &mut RuntimeView, scenario_id: &str, index: usize) -> Result<()> {
        let scenario = view
            .find_scenario_mut(scenario_id)
            .ok_or_else(|| AppError::ElementNotFound(format!("scenario: {}", scenario_id)))?;

        if index >= scenario.dividers.len() {
            return Err(AppError::ElementNotFound(format!(
                "divider index {} in scenario: {}",
                index, scenario_id
            )));
        }

        scenario.dividers.remove(index);
        view.touch();
        Ok(())
    }

    // ==================== Helper Functions ====================

    /// Recursively remove steps referencing a participant from blocks tree
    fn remove_steps_referencing_participant(blocks: &mut Vec<Block>, element_id: &str) {
        blocks.retain(|b| !matches!(b, Block::Step(s) if s.from == element_id || s.to == element_id));

        for block in blocks.iter_mut() {
            if let Block::Group(g) = block {
                if g.branches.is_empty() {
                    Self::remove_steps_referencing_participant(&mut g.blocks, element_id);
                } else {
                    for branch in &mut g.branches {
                        Self::remove_steps_referencing_participant(&mut branch.blocks, element_id);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_view() -> RuntimeView {
        RuntimeView::new("Test Runtime")
    }

    #[test]
    fn test_add_scenario() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("Scenario 1"), None).unwrap();
        assert_eq!(view.scenarios.len(), 1);
        assert_eq!(view.scenarios[0].id, "S1");
    }

    #[test]
    fn test_add_duplicate_scenario() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        let result = RuntimeOperations::add_scenario(&mut view, "S1", Some("S1 dup"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_participant() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_participant(
            &mut view,
            "S1",
            "USER",
            ParticipantType::Actor,
            None,
            None,
        )
        .unwrap();
        assert_eq!(view.scenarios[0].participants.len(), 1);
    }

    #[test]
    fn test_add_duplicate_participant() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_participant(&mut view, "S1", "USER", ParticipantType::Actor, None, None).unwrap();
        let result = RuntimeOperations::add_participant(&mut view, "S1", "USER", ParticipantType::Actor, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_step_top_level() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_participant(&mut view, "S1", "A", ParticipantType::Participant, None, None).unwrap();
        RuntimeOperations::add_participant(&mut view, "S1", "B", ParticipantType::Participant, None, None).unwrap();
        RuntimeOperations::add_step(
            &mut view, "S1", "A", "B", "hello",
            StepType::Sync, None, None, None, None, None,
        ).unwrap();

        assert_eq!(view.scenarios[0].blocks.len(), 1);
        if let Block::Step(s) = &view.scenarios[0].blocks[0] {
            assert_eq!(s.order, 1);
            assert_eq!(s.from, "A");
        }
    }

    #[test]
    fn test_auto_step_order() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_step(&mut view, "S1", "A", "B", "first", StepType::Sync, None, None, None, None, None).unwrap();
        RuntimeOperations::add_step(&mut view, "S1", "B", "A", "second", StepType::Return, None, None, None, None, None).unwrap();

        if let Block::Step(s) = &view.scenarios[0].blocks[1] {
            assert_eq!(s.order, 2);
        }
    }

    #[test]
    fn test_add_step_to_alt_branch() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Alt, "result",
            vec!["ok".to_string(), "err".to_string()], None, None,
        ).unwrap();

        RuntimeOperations::add_step(
            &mut view, "S1", "A", "B", "success",
            StepType::Sync, None, None, None, Some("result"), Some("ok"),
        ).unwrap();

        let scenario = &view.scenarios[0];
        if let Block::Group(g) = &scenario.blocks[0] {
            assert_eq!(g.branches[0].blocks.len(), 1);
        }
    }

    #[test]
    fn test_add_step_to_nonexistent_group() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        let result = RuntimeOperations::add_step(
            &mut view, "S1", "A", "B", "msg",
            StepType::Sync, None, None, None, Some("nonexistent"), None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_step_to_alt_without_branch() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Alt, "result",
            vec!["ok".to_string()], None, None,
        ).unwrap();
        let result = RuntimeOperations::add_step(
            &mut view, "S1", "A", "B", "msg",
            StepType::Sync, None, None, None, Some("result"), None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_step_with_branch_to_non_alt() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Loop, "retry",
            vec![], None, None,
        ).unwrap();
        let result = RuntimeOperations::add_step(
            &mut view, "S1", "A", "B", "msg",
            StepType::Sync, None, None, None, Some("retry"), Some("x"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_participant_cascade() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_step(&mut view, "S1", "A", "B", "msg1", StepType::Sync, None, None, None, None, None).unwrap();
        RuntimeOperations::add_step(&mut view, "S1", "B", "A", "msg2", StepType::Return, None, None, None, None, None).unwrap();

        // Remove A should cascade-remove both steps (both reference A)
        RuntimeOperations::remove_step(&mut view, "S1", 2, None, None).unwrap(); // remove msg2 first
        // Now only step1 (A->B) remains. Remove participant A.
        // We need to add A and B as participants first
        let scenario = view.find_scenario_mut("S1").unwrap();
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

        RuntimeOperations::remove_participant(&mut view, "S1", "A").unwrap();
        assert_eq!(view.scenarios[0].participants.len(), 1);
        assert_eq!(view.scenarios[0].blocks.len(), 0); // step1 (A->B) was cascade-removed
    }

    #[test]
    fn test_remove_group_cascade() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_step(&mut view, "S1", "A", "B", "before", StepType::Sync, None, None, None, None, None).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Loop, "retry",
            vec![], None, None,
        ).unwrap();

        assert_eq!(view.scenarios[0].blocks.len(), 2);
        RuntimeOperations::remove_group(&mut view, "S1", "retry", None, None).unwrap();
        assert_eq!(view.scenarios[0].blocks.len(), 1);
    }

    #[test]
    fn test_remove_nonexistent_scenario() {
        let mut view = create_test_view();
        let result = RuntimeOperations::remove_scenario(&mut view, "NOPE");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_nonexistent_step() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        let result = RuntimeOperations::remove_step(&mut view, "S1", 999, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_nested_group() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Alt, "result",
            vec!["ok".to_string(), "err".to_string()], None, None,
        ).unwrap();
        RuntimeOperations::add_group(
            &mut view, "S1", GroupType::Loop, "retry",
            vec![], Some("result"), Some("err"),
        ).unwrap();

        let scenario = &view.scenarios[0];
        if let Block::Group(g) = &scenario.blocks[0] {
            assert_eq!(g.branches[1].blocks.len(), 1); // "err" branch contains "retry" group
            if let Block::Group(nested) = &g.branches[1].blocks[0] {
                assert_eq!(nested.label, "retry");
            }
        }
    }

    #[test]
    fn test_add_note_and_divider() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_note(&mut view, "S1", NotePosition::Right, "A", "test note").unwrap();
        RuntimeOperations::add_divider(&mut view, "S1", "Phase 1", 0).unwrap();

        assert_eq!(view.scenarios[0].notes.len(), 1);
        assert_eq!(view.scenarios[0].dividers.len(), 1);
    }

    #[test]
    fn test_remove_note_and_divider() {
        let mut view = create_test_view();
        RuntimeOperations::add_scenario(&mut view, "S1", Some("S1"), None).unwrap();
        RuntimeOperations::add_note(&mut view, "S1", NotePosition::Right, "A", "note1").unwrap();
        RuntimeOperations::add_note(&mut view, "S1", NotePosition::Left, "B", "note2").unwrap();
        RuntimeOperations::add_divider(&mut view, "S1", "Phase 1", 0).unwrap();

        RuntimeOperations::remove_note(&mut view, "S1", 0).unwrap();
        assert_eq!(view.scenarios[0].notes.len(), 1);

        RuntimeOperations::remove_divider(&mut view, "S1", 0).unwrap();
        assert_eq!(view.scenarios[0].dividers.len(), 0);
    }
}
