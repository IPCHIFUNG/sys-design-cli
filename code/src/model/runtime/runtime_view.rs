use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Root structure for Runtime View
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeView {
    pub version: String,
    pub kind: DiagramKind,
    pub metadata: Metadata,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
}

impl RuntimeView {
    /// Create a new RuntimeView
    pub fn new(title: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            kind: DiagramKind::RuntimeView,
            metadata: Metadata {
                title: title.to_string(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            scenarios: Vec::new(),
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.metadata.updated_at = Utc::now();
    }

    /// Find scenario by ID
    pub fn find_scenario(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.iter().find(|s| s.id == id)
    }

    /// Find scenario by ID (mutable)
    pub fn find_scenario_mut(&mut self, id: &str) -> Option<&mut Scenario> {
        self.scenarios.iter_mut().find(|s| s.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    RuntimeView,
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

/// A single runtime scenario (use case, flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub participants: Vec<Participant>,
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub notes: Vec<Note>,
    #[serde(default)]
    pub dividers: Vec<Divider>,
}

impl Scenario {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            participants: Vec::new(),
            blocks: Vec::new(),
            notes: Vec::new(),
            dividers: Vec::new(),
        }
    }

    /// Find participant by element_id
    pub fn find_participant(&self, element_id: &str) -> Option<&Participant> {
        self.participants.iter().find(|p| p.element_id == element_id)
    }

    /// Collect all participant element IDs
    pub fn participant_ids(&self) -> Vec<&str> {
        self.participants.iter().map(|p| p.element_id.as_str()).collect()
    }

    /// Collect all participant element IDs as HashSet
    pub fn participant_id_set(&self) -> HashSet<&str> {
        self.participants.iter().map(|p| p.element_id.as_str()).collect()
    }

    /// Find a top-level group by label (mutable)
    pub fn find_group_mut(&mut self, label: &str) -> Option<&mut Group> {
        for block in &mut self.blocks {
            if let Block::Group(g) = block {
                if g.label == label {
                    return Some(g);
                }
            }
        }
        None
    }

    /// Find a top-level group by label
    pub fn find_group(&self, label: &str) -> Option<&Group> {
        for block in &self.blocks {
            if let Block::Group(g) = block {
                if g.label == label {
                    return Some(g);
                }
            }
        }
        None
    }

    /// Collect all steps recursively from blocks tree
    pub fn collect_all_steps(&self) -> Vec<&Step> {
        let mut steps = Vec::new();
        Self::collect_steps_from_blocks(&self.blocks, &mut steps);
        steps
    }

    /// Get the maximum step order (for auto-assigning new order)
    pub fn max_step_order(&self) -> u32 {
        Self::find_max_order(&self.blocks)
    }

    fn find_max_order(blocks: &[Block]) -> u32 {
        let mut max_order = 0u32;
        for block in blocks {
            match block {
                Block::Step(s) => {
                    if s.order > max_order {
                        max_order = s.order;
                    }
                }
                Block::Group(g) => {
                    let inner_max = if g.branches.is_empty() {
                        Self::find_max_order(&g.blocks)
                    } else {
                        g.branches
                            .iter()
                            .map(|b| Self::find_max_order(&b.blocks))
                            .max()
                            .unwrap_or(0)
                    };
                    if inner_max > max_order {
                        max_order = inner_max;
                    }
                }
            }
        }
        max_order
    }

    fn collect_steps_from_blocks<'a>(blocks: &'a [Block], steps: &mut Vec<&'a Step>) {
        for block in blocks {
            match block {
                Block::Step(s) => steps.push(s),
                Block::Group(g) => {
                    if g.branches.is_empty() {
                        Self::collect_steps_from_blocks(&g.blocks, steps);
                    } else {
                        for branch in &g.branches {
                            Self::collect_steps_from_blocks(&branch.blocks, steps);
                        }
                    }
                }
            }
        }
    }

    /// Get mutable reference to target blocks by group/branch (one level only)
    pub fn get_target_blocks_mut(
        &mut self,
        group_label: Option<&str>,
        branch_label: Option<&str>,
    ) -> Option<&mut Vec<Block>> {
        match group_label {
            None => Some(&mut self.blocks),
            Some(label) => {
                let group = self.find_group_mut(label)?;
                if group.group_type == GroupType::Alt {
                    match branch_label {
                        Some(bl) => group
                            .branches
                            .iter_mut()
                            .find(|b| b.label == bl)
                            .map(|b| &mut b.blocks),
                        None => None, // Alt group requires branch specification
                    }
                } else {
                    if branch_label.is_some() {
                        return None; // Non-alt group does not accept branch
                    }
                    Some(&mut group.blocks)
                }
            }
        }
    }
}

/// A participant in a scenario (references static model element)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub element_id: String,
    #[serde(default = "default_participant_type")]
    pub participant_type: ParticipantType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

fn default_participant_type() -> ParticipantType {
    ParticipantType::Participant
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantType {
    #[default]
    Participant,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

/// A block in the scenario tree (recursive)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Step(Step),
    Group(Group),
}

/// An interaction step between participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub order: u32,
    pub from: String,
    pub to: String,
    pub message: String,
    #[serde(default = "default_step_type")]
    pub step_type: StepType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activate_target: Option<bool>,
}

fn default_step_type() -> StepType {
    StepType::Sync
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    #[default]
    Sync,
    Async,
    Return,
    Lost,
}

/// A group (UML combined fragment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub group_type: GroupType,
    pub label: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub branches: Vec<GroupBranch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GroupType {
    Alt,
    Opt,
    Loop,
    Par,
    Break,
    Critical,
    Group,
}

/// A branch within an alt group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBranch {
    pub label: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
}

/// An annotation note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub position: NotePosition,
    pub target: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NotePosition {
    Left,
    Right,
    Over,
}

/// A logical section divider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divider {
    pub label: String,
    pub after_order: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_runtime_view() {
        let view = RuntimeView::new("Test Runtime View");
        assert_eq!(view.version, "1.0");
        assert_eq!(view.kind, DiagramKind::RuntimeView);
        assert_eq!(view.metadata.title, "Test Runtime View");
        assert!(view.scenarios.is_empty());
    }

    #[test]
    fn test_find_scenario() {
        let mut view = RuntimeView::new("Test");
        view.scenarios.push(Scenario {
            id: "S1".to_string(),
            name: "Scenario 1".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: Vec::new(),
            notes: Vec::new(),
            dividers: Vec::new(),
        });

        assert!(view.find_scenario("S1").is_some());
        assert!(view.find_scenario("S2").is_none());
    }

    #[test]
    fn test_scenario_max_step_order() {
        let scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![
                Block::Step(Step {
                    order: 5,
                    from: "A".to_string(),
                    to: "B".to_string(),
                    message: "msg".to_string(),
                    step_type: StepType::Sync,
                    protocol: None,
                    color: None,
                    activate_target: None,
                }),
                Block::Step(Step {
                    order: 10,
                    from: "B".to_string(),
                    to: "A".to_string(),
                    message: "reply".to_string(),
                    step_type: StepType::Return,
                    protocol: None,
                    color: None,
                    activate_target: None,
                }),
            ],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        assert_eq!(scenario.max_step_order(), 10);
    }

    #[test]
    fn test_nested_group_max_order() {
        let scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![
                Block::Step(Step {
                    order: 1,
                    from: "A".to_string(),
                    to: "B".to_string(),
                    message: "msg".to_string(),
                    step_type: StepType::Sync,
                    protocol: None,
                    color: None,
                    activate_target: None,
                }),
                Block::Group(Group {
                    group_type: GroupType::Alt,
                    label: "result".to_string(),
                    blocks: Vec::new(),
                    branches: vec![
                        GroupBranch {
                            label: "ok".to_string(),
                            blocks: vec![Block::Step(Step {
                                order: 2,
                                from: "B".to_string(),
                                to: "A".to_string(),
                                message: "ok".to_string(),
                                step_type: StepType::Return,
                                protocol: None,
                                color: None,
                                activate_target: None,
                            })],
                        },
                        GroupBranch {
                            label: "err".to_string(),
                            blocks: vec![Block::Step(Step {
                                order: 3,
                                from: "B".to_string(),
                                to: "A".to_string(),
                                message: "err".to_string(),
                                step_type: StepType::Return,
                                protocol: None,
                                color: None,
                                activate_target: None,
                            })],
                        },
                    ],
                }),
            ],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        assert_eq!(scenario.max_step_order(), 3);
        assert_eq!(scenario.collect_all_steps().len(), 3);
    }

    #[test]
    fn test_block_serde_roundtrip() {
        let step = Block::Step(Step {
            order: 1,
            from: "A".to_string(),
            to: "B".to_string(),
            message: "hello".to_string(),
            step_type: StepType::Sync,
            protocol: None,
            color: None,
            activate_target: None,
        });
        let yaml = serde_yaml::to_string(&step).unwrap();
        assert!(yaml.contains("type: step"));
        let parsed: Block = serde_yaml::from_str(&yaml).unwrap();
        assert!(matches!(parsed, Block::Step(_)));
    }

    #[test]
    fn test_group_block_serde_roundtrip() {
        let group = Block::Group(Group {
            group_type: GroupType::Loop,
            label: "retry".to_string(),
            blocks: vec![Block::Step(Step {
                order: 1,
                from: "A".to_string(),
                to: "B".to_string(),
                message: "retry".to_string(),
                step_type: StepType::Sync,
                protocol: None,
                color: None,
                activate_target: None,
            })],
            branches: Vec::new(),
        });
        let yaml = serde_yaml::to_string(&group).unwrap();
        assert!(yaml.contains("type: group"));
        assert!(yaml.contains("group_type: loop"));
        let parsed: Block = serde_yaml::from_str(&yaml).unwrap();
        assert!(matches!(parsed, Block::Group(_)));
    }

    #[test]
    fn test_runtime_view_full_roundtrip() {
        let mut view = RuntimeView::new("Test");
        view.scenarios.push(Scenario {
            id: "LOGIN".to_string(),
            name: "Login".to_string(),
            description: Some("Login flow".to_string()),
            participants: vec![Participant {
                element_id: "USER".to_string(),
                participant_type: ParticipantType::Actor,
                alias: None,
                color: None,
            }],
            blocks: vec![
                Block::Step(Step {
                    order: 1,
                    from: "USER".to_string(),
                    to: "APP".to_string(),
                    message: "Login".to_string(),
                    step_type: StepType::Sync,
                    protocol: None,
                    color: None,
                    activate_target: None,
                }),
                Block::Group(Group {
                    group_type: GroupType::Alt,
                    label: "result".to_string(),
                    blocks: Vec::new(),
                    branches: vec![GroupBranch {
                        label: "ok".to_string(),
                        blocks: vec![Block::Step(Step {
                            order: 2,
                            from: "APP".to_string(),
                            to: "USER".to_string(),
                            message: "token".to_string(),
                            step_type: StepType::Return,
                            protocol: None,
                            color: None,
                            activate_target: None,
                        })],
                    }],
                }),
            ],
            notes: vec![Note {
                position: NotePosition::Right,
                target: "APP".to_string(),
                text: "Auth service".to_string(),
            }],
            dividers: vec![Divider {
                label: "Init".to_string(),
                after_order: 0,
            }],
        });

        let yaml = serde_yaml::to_string(&view).unwrap();
        let parsed: RuntimeView = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.scenarios.len(), 1);
        assert_eq!(parsed.scenarios[0].id, "LOGIN");
        assert_eq!(parsed.scenarios[0].participants.len(), 1);
        assert_eq!(parsed.scenarios[0].blocks.len(), 2);
        assert_eq!(parsed.scenarios[0].notes.len(), 1);
        assert_eq!(parsed.scenarios[0].dividers.len(), 1);
    }

    #[test]
    fn test_optional_fields_omitted() {
        let step = Step {
            order: 1,
            from: "A".to_string(),
            to: "B".to_string(),
            message: "msg".to_string(),
            step_type: StepType::Sync,
            protocol: None,
            color: None,
            activate_target: None,
        };
        let yaml = serde_yaml::to_string(&step).unwrap();
        assert!(!yaml.contains("protocol"));
        assert!(!yaml.contains("color"));
        assert!(!yaml.contains("activate_target"));
    }

    #[test]
    fn test_default_values_applied() {
        let yaml = "order: 1\nfrom: A\nto: B\nmessage: msg\n";
        let step: Step = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(step.step_type, StepType::Sync);
    }

    #[test]
    fn test_get_target_blocks_mut_top_level() {
        let mut scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![Block::Step(Step {
                order: 1,
                from: "A".to_string(),
                to: "B".to_string(),
                message: "msg".to_string(),
                step_type: StepType::Sync,
                protocol: None,
                color: None,
                activate_target: None,
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        let blocks = scenario.get_target_blocks_mut(None, None).unwrap();
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_get_target_blocks_mut_group() {
        let mut scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![Block::Group(Group {
                group_type: GroupType::Loop,
                label: "retry".to_string(),
                blocks: Vec::new(),
                branches: Vec::new(),
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        let blocks = scenario.get_target_blocks_mut(Some("retry"), None).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_get_target_blocks_mut_alt_branch() {
        let mut scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![Block::Group(Group {
                group_type: GroupType::Alt,
                label: "result".to_string(),
                blocks: Vec::new(),
                branches: vec![
                    GroupBranch {
                        label: "ok".to_string(),
                        blocks: Vec::new(),
                    },
                    GroupBranch {
                        label: "err".to_string(),
                        blocks: Vec::new(),
                    },
                ],
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        let blocks = scenario.get_target_blocks_mut(Some("result"), Some("ok")).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_get_target_blocks_mut_alt_no_branch_fails() {
        let mut scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![Block::Group(Group {
                group_type: GroupType::Alt,
                label: "result".to_string(),
                blocks: Vec::new(),
                branches: vec![GroupBranch {
                    label: "ok".to_string(),
                    blocks: Vec::new(),
                }],
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        // Alt group without branch should return None
        assert!(scenario.get_target_blocks_mut(Some("result"), None).is_none());
    }

    #[test]
    fn test_get_target_blocks_mut_non_alt_with_branch_fails() {
        let mut scenario = Scenario {
            id: "S1".to_string(),
            name: "Test".to_string(),
            description: None,
            participants: Vec::new(),
            blocks: vec![Block::Group(Group {
                group_type: GroupType::Loop,
                label: "retry".to_string(),
                blocks: Vec::new(),
                branches: Vec::new(),
            })],
            notes: Vec::new(),
            dividers: Vec::new(),
        };

        // Non-alt group with branch should return None
        assert!(scenario.get_target_blocks_mut(Some("retry"), Some("x")).is_none());
    }
}
