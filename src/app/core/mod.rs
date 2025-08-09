pub mod types;
pub mod arena;
pub mod gameplay;

pub use types::*;
pub use arena::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleId {
    NoJump,
    LowJump,
    HighJump,
    SpeedUp,
    NoAttack,
    LavaFloor,
    ProjectileRain,
    OrbCollection,
    MoonGravity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnvVarId {
    Gravity,
    GameSpeed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    Player,
    OrbEnergy,
    FloorStd,
    FloorLarge,
    WallLow,
    WallHigh,
    PanelGlass,
    RampLow,
    RampSteep,
    MoveTeleporterIn,
    MoveTeleporterOut,
    MoveClimbSurface,
    HazardLavaPit,
    HazardLaserEmitterStatic,
    HazardLaserTurretRotate,
    InteractButtonFloor,
    InteractButtonWall,
    InteractLever,
    InteractEnemySpawner,
    InteractBarrierEnergy,
    DecorArchMetallic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Option<serde_json::Value>,
    pub incompatible_with: Vec<RuleId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub id: EnvVarId,
    pub name: String,
    pub description: String,
    pub default_value: f64,
    pub range: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefinition {
    pub id: ModuleId,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub parameters: Option<serde_json::Value>,
    pub wfc_weight: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ArenaCell {
    pub x: i32,
    pub y: i32,
    pub module_id: ModuleId,
    pub module_params: Option<serde_json::Value>,
    pub connections: Vec<Direction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    North, South, East, West, Up, Down
}

#[derive(Debug, Clone)]
pub struct Arena {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub modules: Vec<ArenaCell>,
    pub active_rules: Vec<Rule>,
    pub env_variables: HashMap<EnvVarId, f64>,
    pub generation_metadata: GenerationMetadata,
}

#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    pub seed: u64,
    pub generation_time_ms: u64,
    pub algorithm_version: String,
    pub constraints_applied: Vec<String>,
}

impl Arena {
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            width,
            height,
            modules: Vec::new(),
            active_rules: Vec::new(),
            env_variables: HashMap::new(),
            generation_metadata: GenerationMetadata {
                seed,
                generation_time_ms: 0,
                algorithm_version: "1.0.0".to_string(),
                constraints_applied: Vec::new(),
            },
        }
    }
    
    pub fn get_cell(&self, x: i32, y: i32) -> Option<&ArenaCell> {
        self.modules.iter().find(|cell| cell.x == x && cell.y == y)
    }
    
    pub fn count_modules_by_type(&self, module_type: &ModuleId) -> usize {
        self.modules.iter().filter(|cell| &cell.module_id == module_type).count()
    }
    
    pub fn add_module(&mut self, x: i32, y: i32, module_id: ModuleId, params: Option<serde_json::Value>) {
        let cell = ArenaCell {
            x,
            y,
            module_id,
            module_params: params,
            connections: Vec::new(),
        };
        self.modules.push(cell);
    }
    
    pub fn validate_structural_integrity(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check if player exists
        if !self.modules.iter().any(|cell| matches!(cell.module_id, ModuleId::Player)) {
            issues.push("No player spawn point found".to_string());
        }
        
        // Check for minimum walkable surfaces
        let walkable_count = self.modules.iter()
            .filter(|cell| matches!(cell.module_id, 
                ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep))
            .count();
            
        if walkable_count < 3 {
            issues.push(format!("Insufficient walkable surfaces: {} (minimum 3)", walkable_count));
        }
        
        // Check for energy orbs
        let orb_count = self.count_modules_by_type(&ModuleId::OrbEnergy);
        if orb_count == 0 {
            issues.push("No energy orbs found".to_string());
        }
        
        issues
    }
}
