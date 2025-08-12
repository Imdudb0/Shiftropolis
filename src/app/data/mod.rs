//data/mod.rs..
use crate::app::core::*;
use std::collections::HashMap;

pub struct RulesDatabase {
    rules: HashMap<RuleId, Rule>,
}

impl RulesDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            rules: HashMap::new(),
        };
        db.initialize();
        db
    }

    fn initialize(&mut self) {
        let rules = vec![
            Rule {
                id: RuleId::NoJump,
                name: "No Jump".to_string(),
                description: "The jump is deactivated.".to_string(),
                tags: vec!["movement".to_string(), "restriction".to_string(), "difficulty_medium".to_string()],
                parameters: None,
                incompatible_with: vec![RuleId::LowJump, RuleId::HighJump, RuleId::MoonGravity],
            },
            Rule {
                id: RuleId::LowJump,
                name: "Low Jump".to_string(),
                description: "The jump height is reduced.".to_string(),
                tags: vec!["movement".to_string(), "modifier".to_string(), "difficulty_easy".to_string()],
                parameters: Some(serde_json::json!({"jumpHeightMultiplier": 0.5})),
                incompatible_with: vec![RuleId::NoJump, RuleId::HighJump],
            },
            Rule {
                id: RuleId::HighJump,
                name: "High Jump".to_string(),
                description: "The jump height is increased.".to_string(),
                tags: vec!["movement".to_string(), "modifier".to_string(), "difficulty_easy".to_string()],
                parameters: Some(serde_json::json!({"jumpHeightMultiplier": 1.5})),
                incompatible_with: vec![RuleId::NoJump, RuleId::LowJump],
            },
            Rule {
                id: RuleId::SpeedUp,
                name: "Speed Up".to_string(),
                description: "Your movement speed is increased".to_string(),
                tags: vec!["movement".to_string(), "modifier".to_string(), "difficulty_easy".to_string()],
                parameters: Some(serde_json::json!({"speedMultiplier": 1.5})),
                incompatible_with: vec![],
            },
            Rule {
                id: RuleId::NoAttack,
                name: "No Attack".to_string(),
                description: "Attack is disabled.".to_string(),
                tags: vec!["combat".to_string(), "restriction".to_string(), "difficulty_hard".to_string()],
                parameters: None,
                incompatible_with: vec![],
            },
            Rule {
                id: RuleId::LavaFloor,
                name: "Lava Floor".to_string(),
                description: "Dangerous lava pits appear throughout the arena.".to_string(),
                tags: vec!["hazard".to_string(), "environment".to_string(), "difficulty_medium".to_string()],
                parameters: None,
                incompatible_with: vec![],
            },
            Rule {
                id: RuleId::ProjectileRain,
                name: "Projectile Rain".to_string(),
                description: "Projectiles rain from above.".to_string(),
                tags: vec!["hazard".to_string(), "dynamic".to_string(), "difficulty_hard".to_string()],
                parameters: Some(serde_json::json!({"intensity": 1.0, "frequency": 2.0})),
                incompatible_with: vec![],
            },
            Rule {
                id: RuleId::OrbCollection,
                name: "Orb Collection".to_string(),
                description: "More energy orbs spawn in the arena.".to_string(),
                tags: vec!["resource".to_string(), "collection".to_string(), "difficulty_easy".to_string()],
                parameters: Some(serde_json::json!({"orbMultiplier": 2.0})),
                incompatible_with: vec![],
            },
            Rule {
                id: RuleId::MoonGravity,
                name: "Moon Gravity".to_string(),
                description: "Gravity is significantly reduced.".to_string(),
                tags: vec!["physics".to_string(), "environment".to_string(), "difficulty_medium".to_string()],
                parameters: Some(serde_json::json!({"gravityMultiplier": 0.3})),
                incompatible_with: vec![RuleId::NoJump],
            },
        ];

        for rule in rules {
            self.rules.insert(rule.id.clone(), rule);
        }
    }

    pub fn get_rule(&self, id: &RuleId) -> Option<&Rule> {
        self.rules.get(id)
    }

    pub fn get_all_rules(&self) -> Vec<&Rule> {
        self.rules.values().collect()
    }
}

pub struct EnvVarsDatabase {
    variables: HashMap<EnvVarId, EnvVariable>,
}

impl EnvVarsDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            variables: HashMap::new(),
        };
        db.initialize();
        db
    }

    fn initialize(&mut self) {
        let variables = vec![
            EnvVariable {
                id: EnvVarId::Gravity,
                name: "Global Gravity".to_string(),
                description: "Modifies global attraction.".to_string(),
                default_value: 1.0,
                range: (0.2, 3.0),
            },
            EnvVariable {
                id: EnvVarId::GameSpeed,
                name: "Game Speed".to_string(),
                description: "Modifies global game speed.".to_string(),
                default_value: 1.0,
                range: (0.5, 2.0),
            },
        ];

        for var in variables {
            self.variables.insert(var.id.clone(), var);
        }
    }

    pub fn get_variable(&self, id: &EnvVarId) -> Option<&EnvVariable> {
        self.variables.get(id)
    }

    pub fn get_all_variables(&self) -> Vec<&EnvVariable> {
        self.variables.values().collect()
    }
}

pub struct ModulesDatabase {
    modules: HashMap<ModuleId, ModuleDefinition>,
}

impl ModulesDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            modules: HashMap::new(),
        };
        db.initialize();
        db
    }

    fn initialize(&mut self) {
        let modules = vec![
            ModuleDefinition {
                id: ModuleId::Player,
                name: "Player".to_string(),
                description: "Player spawn point.".to_string(),
                tags: vec!["player".to_string(), "spawn".to_string(), "unique".to_string()],
                parameters: Some(serde_json::json!({
                    "health": 100,
                    "speed": 5.0,
                    "jumpForce": 10.0
                })),
                wfc_weight: Some(1), // Very low weight, usually placed manually
            },
            ModuleDefinition {
                id: ModuleId::OrbEnergy,
                name: "Energy Orb".to_string(),
                description: "Collecting this orb adds time to survival countdown.".to_string(),
                tags: vec!["collectible".to_string(), "resource".to_string(), "energy".to_string()],
                parameters: Some(serde_json::json!({"timeValue": 5.0})),
                wfc_weight: Some(15),
            },
            ModuleDefinition {
                id: ModuleId::FloorStd,
                name: "Standard Floor".to_string(),
                description: "A solid basic platform.".to_string(),
                tags: vec!["structure".to_string(), "walkable".to_string(), "basic".to_string()],
                parameters: None,
                wfc_weight: Some(30),
            },
            ModuleDefinition {
                id: ModuleId::FloorLarge,
                name: "Large Floor Tile".to_string(),
                description: "A larger solid platform.".to_string(),
                tags: vec!["structure".to_string(), "walkable".to_string(), "basic".to_string()],
                parameters: Some(serde_json::json!({"sizeX": 2, "sizeZ": 2})),
                wfc_weight: Some(20)
            },
            ModuleDefinition {
                id: ModuleId::WallLow,
                name: "Wall low".to_string(),
                description: "A small obstacle.".to_string(),
                tags: vec!["structure".to_string(), "obstacle".to_string(), "cover".to_string(), "blocks_movement_low".to_string(), "connect_sides_only".to_string(), "basic".to_string(), "static".to_string()],
                parameters: None,
                wfc_weight: Some(15)
            },
            ModuleDefinition {
                id: ModuleId::WallHigh,
                name: "Wall high".to_string(),
                description: "A small obstacle.".to_string(),
                tags: vec!["structure".to_string(), "obstacle".to_string(), "blocks_vision".to_string(), "blocks_movement".to_string(), "connect_sides_only".to_string(), "basic".to_string(), "static".to_string()],
                parameters: None,
                wfc_weight: Some(10)
            },
            ModuleDefinition {
                id: ModuleId::PanelGlass,
                name: "Panel glass".to_string(),
                description: "Solid but transparent.".to_string(),
                tags: vec!["structure".to_string(), "obstacle".to_string(), "transparent".to_string(), "blocks_movement".to_string(), "connect_sides_only".to_string(), "static".to_string()],
                parameters: Some(serde_json::json!({
                    "breakable": false
                })),
                wfc_weight: Some(7)
            },
            ModuleDefinition {
                id: ModuleId::RampLow,
                name: "Ramp low".to_string(),
                description: "Allows you to change elevation smoothly.".to_string(),
                tags: vec!["structure".to_string(), "walkable".to_string(), "ramp".to_string(), "connect_ends_level_diff".to_string(), "basic".to_string(), "static".to_string()],
                parameters: Some(serde_json::json!({
                    "angle": 30
                })),
                wfc_weight: Some(12)
            },
            ModuleDefinition {
                id: ModuleId::RampSteep,
                name: "Steep Ramp".to_string(),
                description: "Allows quick elevation change.".to_string(),
                tags: vec!["structure".to_string(), "walkable".to_string(), "ramp".to_string()],
                parameters: Some(serde_json::json!({"angle": 45})),
                wfc_weight: Some(10),
            },
            ModuleDefinition {
                id: ModuleId::MoveTeleporterIn,
                name: "Teleporter Entry".to_string(),
                description: "Entry point for teleportation.".to_string(),
                tags: vec!["movement_aid".to_string(), "interactive".to_string(), "teleporter".to_string()],
                parameters: Some(serde_json::json!({"linkId": null})),
                wfc_weight: Some(5),
            },
            ModuleDefinition {
                id: ModuleId::MoveTeleporterOut,
                name: "Teleporter Exit".to_string(),
                description: "Exit point for teleportation.".to_string(),
                tags: vec!["movement_aid".to_string(), "teleporter".to_string(), "destination".to_string()],
                parameters: Some(serde_json::json!({"linkId": null})),
                wfc_weight: Some(5),
            },
            ModuleDefinition {
                id: ModuleId::MoveClimbSurface,
                name: "Climbing Surface".to_string(),
                description: "Allows climbing.".to_string(),
                tags: vec!["movement_aid".to_string(), "climbable".to_string(), "vertical".to_string()],
                parameters: Some(serde_json::json!({"climbSpeed": 3})),
                wfc_weight: Some(8),
            },
            ModuleDefinition {
                id: ModuleId::HazardLavaPit,
                name: "Lava Pit".to_string(),
                description: "Continuous damage hazard.".to_string(),
                tags: vec!["hazard".to_string(), "damage".to_string(), "environmental".to_string()],
                parameters: Some(serde_json::json!({"damagePerSecond": 25})),
                wfc_weight: Some(5),
            },
            ModuleDefinition {
                id: ModuleId::HazardLaserEmitterStatic,
                name: "Static Laser Emitter".to_string(),
                description: "Continuous laser beam.".to_string(),
                tags: vec!["hazard".to_string(), "damage".to_string(), "beam".to_string(), "static".to_string()],
                parameters: Some(serde_json::json!({
                    "damagePerSecond": 30,
                    "beamLength": 20
                })),
                wfc_weight: Some(3),
            },
            ModuleDefinition {
                id: ModuleId::HazardLaserTurretRotate,
                name: "Rotating Laser Turret".to_string(),
                description: "Sweeping laser beam.".to_string(),
                tags: vec!["hazard".to_string(), "damage".to_string(), "beam".to_string(), "dynamic".to_string()],
                parameters: Some(serde_json::json!({
                    "damagePerSecond": 40,
                    "rotationSpeed": 45,
                    "arc": 180,
                    "beamLength": 20
                })),
                wfc_weight: Some(2),
            },
            ModuleDefinition {
                id: ModuleId::InteractButtonFloor,
                name: "Floor Button".to_string(),
                description: "Activated by walking on it.".to_string(),
                tags: vec!["interactive".to_string(), "trigger".to_string(), "walkable".to_string()],
                parameters: Some(serde_json::json!({
                    "triggerId": null,
                    "oneTime": false,
                    "resetDelay": 0.5
                })),
                wfc_weight: Some(7),
            },
            ModuleDefinition {
                id: ModuleId::InteractButtonWall,
                name: "Wall Button".to_string(),
                description: "Activated by interaction or shooting.".to_string(),
                tags: vec!["interactive".to_string(), "trigger".to_string(), "wall_mount".to_string()],
                parameters: Some(serde_json::json!({
                    "triggerId": null,
                    "shootable": true
                })),
                wfc_weight: Some(6),
            },
            ModuleDefinition {
                id: ModuleId::InteractLever,
                name: "Lever".to_string(),
                description: "Manual interaction toggle.".to_string(),
                tags: vec!["interactive".to_string(), "trigger".to_string(), "toggle".to_string()],
                parameters: Some(serde_json::json!({
                    "triggerId": null,
                    "startsOn": false
                })),
                wfc_weight: Some(6),
            },
            ModuleDefinition {
                id: ModuleId::InteractEnemySpawner,
                name: "Enemy Spawner".to_string(),
                description: "Spawns enemies.".to_string(),
                tags: vec!["interactive".to_string(), "spawner".to_string(), "enemy".to_string()],
                parameters: Some(serde_json::json!({
                    "enemyType": "ENEMY_TYPE_BASIC_ROBOT",
                    "spawnLimit": 3,
                    "triggerId": null,
                    "spawnRadius": 2,
                    "activationDelay": 0.5
                })),
                wfc_weight: Some(4),
            },
            ModuleDefinition {
                id: ModuleId::InteractBarrierEnergy,
                name: "Energy Barrier".to_string(),
                description: "Blocks passage and projectiles.".to_string(),
                tags: vec!["interactive".to_string(), "obstacle".to_string(), "toggleable".to_string()],
                parameters: Some(serde_json::json!({
                    "health": 100,
                    "disableOnTriggerId": null,
                    "disableDuration": 5.0,
                    "startActive": true
                })),
                wfc_weight: Some(5),
            },
            ModuleDefinition {
                id: ModuleId::DecorArchMetallic,
                name: "Metal Arch".to_string(),
                description: "Large arch in stylish metal to give visual structure to the arena.".to_string(),
                tags: vec!["decor".to_string(), "structure".to_string(), "static".to_string(), "metallic".to_string(), "connect_all_sides".to_string(), "variantGroup".to_string()],
                parameters: Some(serde_json::json!({
                    "colorVariant": 3
                })),
                wfc_weight: Some(4)
            },
        ];

        for module in modules {
            self.modules.insert(module.id.clone(), module);
        }
    }

    pub fn get_module(&self, id: &ModuleId) -> Option<&ModuleDefinition> {
        self.modules.get(id)
    }

    pub fn get_all_modules(&self) -> Vec<&ModuleDefinition> {
        self.modules.values().collect()
    }

    pub fn get_modules_by_tag(&self, tag: &str) -> Vec<&ModuleDefinition> {
        self.modules.values()
            .filter(|module| module.tags.contains(&tag.to_string()))
            .collect()
    }

    pub fn get_weighted_modules(&self) -> Vec<(&ModuleDefinition, u32)> {
        self.modules.values()
            .filter_map(|module| {
                module.wfc_weight.map(|weight| (module, weight))
            })
            .collect()
    }
}
