//! Built-in code generation templates for game development.
//!
//! Provides pre-defined prompt templates for common game code patterns
//! supporting both Godot (GDScript) and Unity (C#).

use serde::{Deserialize, Serialize};

/// A built-in code template for a specific engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltInTemplate {
    /// Unique template identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Template description.
    pub description: String,
    /// Target game engine.
    pub engine: String,
    /// The prompt template text with {{variable}} placeholders.
    pub prompt_template: String,
    /// List of variables used in the template.
    pub variables: Vec<String>,
}

/// Godot player controller template.
const GODOT_PLAYER_CONTROLLER: &str = r#"Generate a player controller script for Godot 4.

Class: {{class_name}}
Base: CharacterBody2D or CharacterBody3D (use {{base_class}})
Properties:
- Movement speed: {{move_speed}}
- Jump force: {{jump_force}}
- Gravity: {{gravity}}

Requirements:
- Use @export for tunable properties
- Implement _physics_process
- Handle input for movement and jumping
- Include wall jump if {{wall_jump}} is true
- Add coyote time: {{coyote_time}} seconds
- Add jump buffer: {{jump_buffer}} seconds

Style: GDScript 2.0 with type hints
Output format: JSON with files array"#;

/// Godot enemy AI template.
const GODOT_ENEMY_AI: &str = r#"Generate an enemy AI script for Godot 4.

Class: {{class_name}}
Base: {{base_class}}
AI Type: {{ai_type}} (patrol, chase, guard, flyer)

Properties:
- Detection range: {{detection_range}}
- Movement speed: {{move_speed}}
- Health: {{health}}

Requirements:
- Use @export for tunable properties
- Implement state machine with states: idle, chase, attack, wander
- Add line-of-sight checking
- Implement pathfinding to player when detected
- Handle damage and death states

Style: GDScript 2.0 with type hints
Output format: JSON with files array"#;

/// Godot inventory system template.
const GODOT_INVENTORY_SYSTEM: &str = r#"Generate an inventory system for Godot 4.

Classes to generate:
- {{class_name}}Item: Data class for inventory items
- {{class_name}}Slot: UI representation of a slot
- {{class_name}}Inventory: Main inventory controller

Properties:
- Max slots: {{max_slots}}
- Stack size: {{stack_size}}
- Weight system: {{has_weight}}

Requirements:
- Implement add/remove/swap items
- Support drag and drop in UI
- Handle item stacking
- Save/load inventory to disk
- Use signals for inventory changes

Style: GDScript 2.0 with type hints
Output format: JSON with files array"#;

/// Godot dialog tree template.
const GODOT_DIALOG_TREE: &str = r#"Generate a dialog system for Godot 4.

Classes to generate:
- {{class_name}}Node: Base dialog node
- {{class_name}}Branch: Branching dialog (choices)
- {{class_name}}Speaker: NPC speaker with portraits

Properties:
- Auto-advance: {{auto_advance}}
- Text speed: {{text_speed}}
- Audio: {{has_audio}}

Requirements:
- Support branching conversations
- Track player choices
- Include character portraits
- Play audio for voice lines
- Save/load dialog state

Style: GDScript 2.0 with type hints
Output format: JSON with files array"#;

/// Godot state machine template.
const GODOT_STATE_MACHINE: &str = r#"Generate a state machine for Godot 4.

Classes to generate:
- {{class_name}}State: Base state class
- {{class_name}}StateMachine: Main state machine controller
- States to include: {{states}}

Properties:
- Default state: {{default_state}}
- Can interrupt: {{can_interrupt}}

Requirements:
- Use state pattern with enter/exit/process
- Support state transitions
- Emit signals on state change
- Support animation integration
- Handle state stack for nested states

Style: GDScript 2.0 with type hints
Output format: JSON with files array"#;

/// Unity player controller template.
const UNITY_PLAYER_CONTROLLER: &str = r#"Generate a player controller script for Unity.

Class: {{class_name}}
Inherits: {{base_class}} (MonoBehaviour)

Properties:
- Movement speed: {{move_speed}}
- Jump force: {{jump_force}}
- Gravity: {{gravity}}

Requirements:
- Use [SerializeField] for private fields
- Implement FixedUpdate for physics
- Handle input for movement and jumping
- Include wall jump if {{wall_jump}} is true
- Add Input System support

Style: C# with proper namespaces and documentation
Output format: JSON with files array"#;

/// Unity enemy AI template.
const UNITY_ENEMY_AI: &str = r#"Generate an enemy AI script for Unity.

Class: {{class_name}}
Inherits: {{base_class}} (MonoBehaviour)
AI Type: {{ai_type}} (patrol, chase, guard, flyer)

Properties:
- Detection range: {{detection_range}}
- Movement speed: {{move_speed}}
- Health: {{health}}

Requirements:
- Use [SerializeField] for tunable properties
- Implement state machine with states: idle, chase, attack, wander
- Add line-of-sight checking using raycasts
- Implement pathfinding to player when detected
- Handle damage and death states

Style: C# with proper namespaces and documentation
Output format: JSON with files array"#;

/// Unity inventory system template.
const UNITY_INVENTORY_SYSTEM: &str = r#"Generate an inventory system for Unity.

Classes to generate:
- {{class_name}}Item: Data class for inventory items (marked [Serializable])
- {{class_name}}Slot: UI slot component
- {{class_name}}Inventory: Main inventory controller

Properties:
- Max slots: {{max_slots}}
- Stack size: {{stack_size}}
- Weight system: {{has_weight}}

Requirements:
- Implement add/remove/swap items
- Support UI drag and drop
- Handle item stacking
- Save/load inventory to disk using PlayerPrefs or JSON
- Use events for inventory changes

Style: C# with proper namespaces and documentation
Output format: JSON with files array"#;

/// Unity dialog tree template.
const UNITY_DIALOG_TREE: &str = r#"Generate a dialog system for Unity.

Classes to generate:
- {{class_name}}Node: Base dialog node (marked [Serializable])
- {{class_name}}Branch: Branching dialog (choices)
- {{class_name}}Speaker: NPC speaker with portraits

Properties:
- Auto-advance: {{auto_advance}}
- Text speed: {{text_speed}}
- Audio: {{has_audio}}

Requirements:
- Support branching conversations
- Track player choices using a dictionary
- Include character portraits
- Play audio for voice lines
- Save/load dialog state

Style: C# with proper namespaces and documentation
Output format: JSON with files array"#;

/// Unity state machine template.
const UNITY_STATE_MACHINE: &str = r#"Generate a state machine for Unity.

Classes to generate:
- {{class_name}}State: Base state class (marked [Serializable])
- {{class_name}}StateMachine: Main state machine controller
- States to include: {{states}}

Properties:
- Default state: {{default_state}}
- Can interrupt: {{can_interrupt}}

Requirements:
- Use state pattern with OnEnter/OnExit/OnUpdate
- Support state transitions
- Use events for state changes
- Integrate with Animator for animation states
- Support state stack for nested states

Style: C# with proper namespaces and documentation
Output format: JSON with files array"#;

/// Returns all built-in code templates.
pub fn built_in_templates() -> Vec<BuiltInTemplate> {
    vec![
        // Godot templates
        BuiltInTemplate {
            id: "godot_player_controller".to_string(),
            name: "Player Controller".to_string(),
            description:
                "A controllable player character with movement, jumping, and optional wall jump"
                    .to_string(),
            engine: "godot".to_string(),
            prompt_template: GODOT_PLAYER_CONTROLLER.to_string(),
            variables: vec![
                "class_name".to_string(),
                "base_class".to_string(),
                "move_speed".to_string(),
                "jump_force".to_string(),
                "gravity".to_string(),
                "wall_jump".to_string(),
                "coyote_time".to_string(),
                "jump_buffer".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "godot_enemy_ai".to_string(),
            name: "Enemy AI".to_string(),
            description: "An AI-controlled enemy with patrol, chase, and attack behaviors"
                .to_string(),
            engine: "godot".to_string(),
            prompt_template: GODOT_ENEMY_AI.to_string(),
            variables: vec![
                "class_name".to_string(),
                "base_class".to_string(),
                "ai_type".to_string(),
                "detection_range".to_string(),
                "move_speed".to_string(),
                "health".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "godot_inventory_system".to_string(),
            name: "Inventory System".to_string(),
            description: "A complete inventory system with slots, stacking, and save/load"
                .to_string(),
            engine: "godot".to_string(),
            prompt_template: GODOT_INVENTORY_SYSTEM.to_string(),
            variables: vec![
                "class_name".to_string(),
                "max_slots".to_string(),
                "stack_size".to_string(),
                "has_weight".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "godot_dialog_tree".to_string(),
            name: "Dialog Tree".to_string(),
            description: "A branching dialog system with character portraits and audio support"
                .to_string(),
            engine: "godot".to_string(),
            prompt_template: GODOT_DIALOG_TREE.to_string(),
            variables: vec![
                "class_name".to_string(),
                "auto_advance".to_string(),
                "text_speed".to_string(),
                "has_audio".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "godot_state_machine".to_string(),
            name: "State Machine".to_string(),
            description: "A flexible state machine for managing character states and animations"
                .to_string(),
            engine: "godot".to_string(),
            prompt_template: GODOT_STATE_MACHINE.to_string(),
            variables: vec![
                "class_name".to_string(),
                "states".to_string(),
                "default_state".to_string(),
                "can_interrupt".to_string(),
            ],
        },
        // Unity templates
        BuiltInTemplate {
            id: "unity_player_controller".to_string(),
            name: "Player Controller".to_string(),
            description:
                "A controllable player character with movement, jumping, and optional wall jump"
                    .to_string(),
            engine: "unity".to_string(),
            prompt_template: UNITY_PLAYER_CONTROLLER.to_string(),
            variables: vec![
                "class_name".to_string(),
                "base_class".to_string(),
                "move_speed".to_string(),
                "jump_force".to_string(),
                "gravity".to_string(),
                "wall_jump".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "unity_enemy_ai".to_string(),
            name: "Enemy AI".to_string(),
            description: "An AI-controlled enemy with patrol, chase, and attack behaviors"
                .to_string(),
            engine: "unity".to_string(),
            prompt_template: UNITY_ENEMY_AI.to_string(),
            variables: vec![
                "class_name".to_string(),
                "base_class".to_string(),
                "ai_type".to_string(),
                "detection_range".to_string(),
                "move_speed".to_string(),
                "health".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "unity_inventory_system".to_string(),
            name: "Inventory System".to_string(),
            description: "A complete inventory system with slots, stacking, and save/load"
                .to_string(),
            engine: "unity".to_string(),
            prompt_template: UNITY_INVENTORY_SYSTEM.to_string(),
            variables: vec![
                "class_name".to_string(),
                "max_slots".to_string(),
                "stack_size".to_string(),
                "has_weight".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "unity_dialog_tree".to_string(),
            name: "Dialog Tree".to_string(),
            description: "A branching dialog system with character portraits and audio support"
                .to_string(),
            engine: "unity".to_string(),
            prompt_template: UNITY_DIALOG_TREE.to_string(),
            variables: vec![
                "class_name".to_string(),
                "auto_advance".to_string(),
                "text_speed".to_string(),
                "has_audio".to_string(),
            ],
        },
        BuiltInTemplate {
            id: "unity_state_machine".to_string(),
            name: "State Machine".to_string(),
            description: "A flexible state machine for managing character states and animations"
                .to_string(),
            engine: "unity".to_string(),
            prompt_template: UNITY_STATE_MACHINE.to_string(),
            variables: vec![
                "class_name".to_string(),
                "states".to_string(),
                "default_state".to_string(),
                "can_interrupt".to_string(),
            ],
        },
    ]
}

/// Returns templates filtered by engine.
pub fn templates_for_engine(engine: &str) -> Vec<BuiltInTemplate> {
    built_in_templates()
        .into_iter()
        .filter(|t| t.engine == engine)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_built_in_templates_count() {
        let templates = built_in_templates();
        // 5 Godot + 5 Unity = 10 templates
        assert_eq!(templates.len(), 10);
    }

    #[test]
    fn test_templates_for_godot() {
        let templates = templates_for_engine("godot");
        assert_eq!(templates.len(), 5);
        assert!(templates.iter().all(|t| t.engine == "godot"));
    }

    #[test]
    fn test_templates_for_unity() {
        let templates = templates_for_engine("unity");
        assert_eq!(templates.len(), 5);
        assert!(templates.iter().all(|t| t.engine == "unity"));
    }

    #[test]
    fn test_templates_for_unknown_engine() {
        let templates = templates_for_engine("unreal");
        assert!(templates.is_empty());
    }

    #[test]
    fn test_template_has_variables() {
        let templates = built_in_templates();
        for template in templates {
            assert!(!template.id.is_empty());
            assert!(!template.name.is_empty());
            assert!(!template.prompt_template.is_empty());
            assert!(!template.variables.is_empty());
            for var in &template.variables {
                let expected = format!("{{{{{}}}}}", var);
                assert!(
                    template.prompt_template.contains(&expected),
                    "Template {} missing variable {}",
                    template.id,
                    var
                );
            }
        }
    }

    #[test]
    fn test_player_controller_template_contains_key_elements() {
        let templates = built_in_templates();
        let player_controller = templates
            .iter()
            .find(|t| t.id == "godot_player_controller")
            .expect("godot_player_controller template not found");

        assert!(player_controller
            .prompt_template
            .contains("CharacterBody2D"));
        assert!(player_controller.prompt_template.contains("@export"));
        assert!(player_controller
            .prompt_template
            .contains("_physics_process"));
    }

    #[test]
    fn test_unity_template_contains_key_elements() {
        let templates = built_in_templates();
        let player_controller = templates
            .iter()
            .find(|t| t.id == "unity_player_controller")
            .expect("unity_player_controller template not found");

        assert!(player_controller.prompt_template.contains("MonoBehaviour"));
        assert!(player_controller
            .prompt_template
            .contains("[SerializeField]"));
        assert!(player_controller.prompt_template.contains("FixedUpdate"));
    }

    #[test]
    fn test_template_serialization() {
        let templates = built_in_templates();
        for template in &templates {
            let json = serde_json::to_string(template).unwrap();
            let deserialized: BuiltInTemplate = serde_json::from_str(&json).unwrap();
            assert_eq!(template.id, deserialized.id);
            assert_eq!(template.name, deserialized.name);
            assert_eq!(template.engine, deserialized.engine);
        }
    }
}
