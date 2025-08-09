//core/types.rs
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::core::*;

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RuleId::NoJump => "NO_JUMP",
            RuleId::LowJump => "LOW_JUMP",
            RuleId::HighJump => "HIGH_JUMP",
            RuleId::SpeedUp => "SPEED_UP",
            RuleId::NoAttack => "NO_ATTACK",
            RuleId::LavaFloor => "LAVA_FLOOR",
            RuleId::ProjectileRain => "PROJECTILE_RAIN",
            RuleId::OrbCollection => "ORB_COLLECTION",
            RuleId::MoonGravity => "MOON_GRAVITY",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ModuleId::Player => "PLAYER",
            ModuleId::OrbEnergy => "ORB_ENERGY",
            ModuleId::FloorStd => "FLOOR_STD",
            ModuleId::FloorLarge => "FLOOR_LARGE",
            ModuleId::WallLow => "WALL_LOW",
            ModuleId::WallHigh => "WALL_HIGH",
            ModuleId::PanelGlass => "PANEL_GLASS",
            ModuleId::RampLow => "RAMP_LOW",
            ModuleId::RampSteep => "RAMP_STEEP",
            ModuleId::MoveTeleporterIn => "TELEPORTER_IN",
            ModuleId::MoveTeleporterOut => "TELEPORTER_OUT",
            ModuleId::MoveClimbSurface => "CLIMB_SURFACE",
            ModuleId::HazardLavaPit => "LAVA_PIT",
            ModuleId::HazardLaserEmitterStatic => "LASER_STATIC",
            ModuleId::HazardLaserTurretRotate => "LASER_TURRET",
            ModuleId::InteractButtonFloor => "BUTTON_FLOOR",
            ModuleId::InteractButtonWall => "BUTTON_WALL",
            ModuleId::InteractLever => "LEVER",
            ModuleId::InteractEnemySpawner => "ENEMY_SPAWNER",
            ModuleId::InteractBarrierEnergy => "ENERGY_BARRIER",
            ModuleId::DecorArchMetallic => "DECOR_ARCH_METALLIC"
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for EnvVarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EnvVarId::Gravity => "GRAVITY",
            EnvVarId::GameSpeed => "GAME_SPEED",
        };
        write!(f, "{}", name)
    }
}

// Utility functions for Arena analysis
impl crate::Arena {
    /// Returns all modules of a specific type
    pub fn get_modules_by_type(&self, module_type: &ModuleId) -> Vec<&crate::ArenaCell> {
        self.modules.iter()
            .filter(|cell| &cell.module_id == module_type)
            .collect()
    }

    /// Check if a position is within arena bounds
    pub fn is_valid_position(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    /// Get all adjacent positions (4-directional)
    pub fn get_adjacent_positions(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        vec![(x+1, y), (x-1, y), (x, y+1), (x, y-1)]
            .into_iter()
            .filter(|&(nx, ny)| self.is_valid_position(nx, ny))
            .collect()
    }

    /// Calculate density of a specific module type
    pub fn calculate_module_density(&self, module_type: &ModuleId) -> f64 {
        let count = self.count_modules_by_type(module_type) as f64;
        let total_area = (self.width * self.height) as f64;
        count / total_area
    }

    /// Get arena statistics
    pub fn get_statistics(&self) -> ArenaStatistics {
        let total_cells = (self.width * self.height) as usize;
        let filled_cells = self.modules.len();
        let empty_cells = total_cells - filled_cells;

        let mut module_counts = std::collections::HashMap::new();
        for cell in &self.modules {
            *module_counts.entry(&cell.module_id).or_insert(0) += 1;
        }

        let walkable_count = self.get_modules_by_type(&ModuleId::FloorStd).len() +
                           self.get_modules_by_type(&ModuleId::FloorLarge).len() +
                           self.get_modules_by_type(&ModuleId::RampSteep).len();

        let hazard_count = self.get_modules_by_type(&ModuleId::HazardLavaPit).len() +
                         self.get_modules_by_type(&ModuleId::HazardLaserEmitterStatic).len() +
                         self.get_modules_by_type(&ModuleId::HazardLaserTurretRotate).len();

        ArenaStatistics {
            total_cells,
            filled_cells,
            empty_cells,
            walkable_cells: walkable_count,
            hazard_cells: hazard_count,
            energy_orbs: self.count_modules_by_type(&ModuleId::OrbEnergy),
            interactive_elements: self.count_interactive_elements(),
            fill_ratio: filled_cells as f64 / total_cells as f64,
            hazard_density: hazard_count as f64 / total_cells as f64,
            walkable_ratio: walkable_count as f64 / total_cells as f64,
        }
    }

    fn count_interactive_elements(&self) -> usize {
        self.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::InteractButtonFloor |
                ModuleId::InteractButtonWall |
                ModuleId::InteractLever |
                ModuleId::InteractBarrierEnergy |
                ModuleId::MoveTeleporterIn |
                ModuleId::MoveTeleporterOut
            ))
            .count()
    }
}

#[derive(Debug, Clone)]
pub struct ArenaStatistics {
    pub total_cells: usize,
    pub filled_cells: usize,
    pub empty_cells: usize,
    pub walkable_cells: usize,
    pub hazard_cells: usize,
    pub energy_orbs: usize,
    pub interactive_elements: usize,
    pub fill_ratio: f64,
    pub hazard_density: f64,
    pub walkable_ratio: f64,
}

impl fmt::Display for ArenaStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Arena Statistics:")?;
        writeln!(f, "  Total Cells: {}", self.total_cells)?;
        writeln!(f, "  Filled: {} ({:.1}%)", self.filled_cells, self.fill_ratio * 100.0)?;
        writeln!(f, "  Empty: {}", self.empty_cells)?;
        writeln!(f, "  Walkable: {} ({:.1}%)", self.walkable_cells, self.walkable_ratio * 100.0)?;
        writeln!(f, "  Hazards: {} ({:.1}%)", self.hazard_cells, self.hazard_density * 100.0)?;
        writeln!(f, "  Energy Orbs: {}", self.energy_orbs)?;
        writeln!(f, "  Interactive: {}", self.interactive_elements)?;
        Ok(())
    }
}

// Error types for better error handling
#[derive(Debug, thiserror::Error)]
pub enum ArenaGenerationError {
    #[error("Arena generation failed: {message}")]
    GenerationFailed { message: String },

    #[error("Invalid configuration: {details}")]
    InvalidConfiguration { details: String },

    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },

    #[error("Critical anomaly detected: {anomaly}")]
    CriticalAnomaly { anomaly: String },

    #[error("Timeout during generation (exceeded {max_time_ms}ms)")]
    Timeout { max_time_ms: u64 },
}

// Performance monitoring structures
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub generation_time_ms: u64,
    pub wfc_iterations: u32,
    pub constraint_checks: u32,
    pub anomaly_checks: u32,
    pub memory_peak_mb: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn record_generation_time(&mut self, time_ms: u64) {
        self.generation_time_ms = time_ms;
    }

    pub fn increment_wfc_iterations(&mut self) {
        self.wfc_iterations += 1;
    }

    pub fn increment_constraint_checks(&mut self) {
        self.constraint_checks += 1;
    }

    pub fn increment_anomaly_checks(&mut self) {
        self.anomaly_checks += 1;
    }

    pub fn record_memory_usage(&mut self, memory_mb: f64) {
        if memory_mb > self.memory_peak_mb {
            self.memory_peak_mb = memory_mb;
        }
    }
}

impl fmt::Display for PerformanceMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Performance Metrics:")?;
        writeln!(f, "  Generation Time: {}ms", self.generation_time_ms)?;
        writeln!(f, "  WFC Iterations: {}", self.wfc_iterations)?;
        writeln!(f, "  Constraint Checks: {}", self.constraint_checks)?;
        writeln!(f, "  Anomaly Checks: {}", self.anomaly_checks)?;
        writeln!(f, "  Peak Memory: {:.1}MB", self.memory_peak_mb)?;
        Ok(())
    }
}
