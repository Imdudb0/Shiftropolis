//core/arena.rs
use crate::app::core::*;
use std::collections::{HashMap, HashSet, VecDeque};

impl Arena {
    /// Advanced structural integrity validation
    pub fn validate_advanced_integrity(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check basic requirements
        self.check_basic_requirements(&mut result);

        // Check connectivity
        self.check_connectivity(&mut result);

        // Check balance
        self.check_gameplay_balance(&mut result);

        // Check rule consistency
        self.check_rule_environment_consistency(&mut result);

        // Check spatial distribution
        self.check_spatial_distribution(&mut result);

        result
    }

    fn check_basic_requirements(&self, result: &mut ValidationResult) {
        // Player spawn
        let player_count = self.count_modules_by_type(&ModuleId::Player);
        match player_count {
            0 => result.add_error("No player spawn point found"),
            1 => {}, // Good
            _ => result.add_warning(&format!("Multiple player spawns found: {}", player_count)),
        }

        // Minimum walkable area
        let walkable_count = self.get_walkable_cells().len();
        let min_walkable = (self.width * self.height / 4) as usize; // At least 25%

        if walkable_count < min_walkable {
            result.add_error(&format!(
                "Insufficient walkable area: {} (minimum: {})",
                walkable_count, min_walkable
            ));
        }

        // Energy orbs
        let orb_count = self.count_modules_by_type(&ModuleId::OrbEnergy);
        if orb_count == 0 {
            result.add_error("No energy orbs found");
        } else if orb_count < 3 {
            result.add_warning(&format!("Low energy orb count: {} (recommended: 3+)", orb_count));
        }
    }

    fn check_connectivity(&self, result: &mut ValidationResult) {
        let player_pos = self.get_player_position();
        if let Some(start) = player_pos {
            let reachable = self.get_reachable_positions(start);

            // Check if all orbs are reachable
            let orb_positions = self.get_modules_by_type(&ModuleId::OrbEnergy);
            let unreachable_orbs: Vec<_> = orb_positions.iter()
                .filter(|cell| !reachable.contains(&(cell.x, cell.y)))
                .collect();

            if !unreachable_orbs.is_empty() {
                result.add_critical(&format!(
                    "{} energy orbs are unreachable from player spawn",
                    unreachable_orbs.len()
                ));
            }

            // Check if interactive elements are reachable
            let interactive_positions = self.get_interactive_elements();
            let unreachable_interactive: Vec<_> = interactive_positions.iter()
                .filter(|cell| !reachable.contains(&(cell.x, cell.y)))
                .collect();

            if !unreachable_interactive.is_empty() {
                result.add_warning(&format!(
                    "{} interactive elements are unreachable",
                    unreachable_interactive.len()
                ));
            }

            // Check connectivity ratio
            let total_important_positions = orb_positions.len() + interactive_positions.len();
            let reachable_important = total_important_positions - unreachable_orbs.len() - unreachable_interactive.len();
            let connectivity_ratio = if total_important_positions > 0 {
                reachable_important as f64 / total_important_positions as f64
            } else {
                1.0
            };

            if connectivity_ratio < 0.8 {
                result.add_error(&format!(
                    "Low connectivity ratio: {:.1}% (should be > 80%)",
                    connectivity_ratio * 100.0
                ));
            }
        } else {
            result.add_critical("Cannot perform connectivity check: no player spawn found");
        }
    }

    fn check_gameplay_balance(&self, result: &mut ValidationResult) {
        let stats = self.get_statistics();

        // Hazard density check
        if stats.hazard_density > 0.4 {
            result.add_warning(&format!(
                "High hazard density: {:.1}% (recommended < 40%)",
                stats.hazard_density * 100.0
            ));
        }

        // Walkable ratio check
        if stats.walkable_ratio < 0.25 {
            result.add_error(&format!(
                "Insufficient walkable area: {:.1}% (minimum 25%)",
                stats.walkable_ratio * 100.0
            ));
        }

        // Energy orb distribution
        let orb_density = stats.energy_orbs as f64 / stats.total_cells as f64;
        if orb_density < 0.03 {
            result.add_warning(&format!(
                "Low energy orb density: {:.3} (recommended > 0.03)",
                orb_density
            ));
        } else if orb_density > 0.25 {
            result.add_info(&format!(
                "High energy orb density: {:.3} (might be too easy)",
                orb_density
            ));
        }

        // Interactive element balance
        let interactive_ratio = stats.interactive_elements as f64 / stats.total_cells as f64;
        if interactive_ratio > 0.15 {
            result.add_warning(&format!(
                "Too many interactive elements: {:.1}% (recommended < 15%)",
                interactive_ratio * 100.0
            ));
        }
    }

    fn check_rule_environment_consistency(&self, result: &mut ValidationResult) {
        for rule in &self.active_rules {
            match rule.id {
                RuleId::MoonGravity => {
                    if let Some(&gravity) = self.env_variables.get(&EnvVarId::Gravity) {
                        if gravity > 0.6 {
                            result.add_warning(&format!(
                                "Moon Gravity rule active but gravity is {:.2} (expected < 0.6)",
                                gravity
                            ));
                        }
                    }
                }

                RuleId::LavaFloor => {
                    let lava_count = self.count_modules_by_type(&ModuleId::HazardLavaPit);
                    if lava_count == 0 {
                        result.add_error("Lava Floor rule active but no lava pits found");
                    } else if lava_count < 2 {
                        result.add_warning(&format!(
                            "Lava Floor rule active but only {} lava pit(s) found",
                            lava_count
                        ));
                    }
                }

                RuleId::OrbCollection => {
                    let orb_count = self.count_modules_by_type(&ModuleId::OrbEnergy);
                    let expected_min = 5; // Based on rule parameters
                    if orb_count < expected_min {
                        result.add_warning(&format!(
                            "Orb Collection rule active but only {} orbs found (expected {}+)",
                            orb_count, expected_min
                        ));
                    }
                }

                RuleId::NoJump => {
                    // Check if there are unreachable elevated areas
                    if self.has_unreachable_elevated_areas() {
                        result.add_error(
                            "No Jump rule active but arena has unreachable elevated areas"
                        );
                    }
                }

                _ => {}
            }
        }
    }

    fn check_spatial_distribution(&self, result: &mut ValidationResult) {
        // Check for clustering of hazards
        self.check_hazard_clustering(result);

        // Check for isolated elements
        self.check_isolated_elements(result);

        // Check for empty zones
        self.check_empty_zones(result);
    }

    fn check_hazard_clustering(&self, result: &mut ValidationResult) {
        let hazards = self.get_hazard_positions();

        if hazards.len() > 1 {
            let mut cluster_count = 0;

            for i in 0..hazards.len() {
                for j in i+1..hazards.len() {
                    let distance = self.manhattan_distance(hazards[i], hazards[j]);
                    if distance <= 2 {
                        cluster_count += 1;
                    }
                }
            }

            let clustering_ratio = cluster_count as f64 / hazards.len() as f64;
            if clustering_ratio > 0.5 {
                result.add_warning(&format!(
                    "High hazard clustering detected: {:.1}%",
                    clustering_ratio * 100.0
                ));
            }
        }
    }

    fn check_isolated_elements(&self, result: &mut ValidationResult) {
        let important_elements = self.get_modules_by_type(&ModuleId::OrbEnergy);

        for element in important_elements {
            let adjacent_walkable = self.count_adjacent_walkable(element.x, element.y);
            if adjacent_walkable == 0 {
                result.add_critical(&format!(
                    "Energy orb at ({}, {}) is completely isolated",
                    element.x, element.y
                ));
            } else if adjacent_walkable == 1 {
                result.add_warning(&format!(
                    "Energy orb at ({}, {}) has only one adjacent walkable cell",
                    element.x, element.y
                ));
            }
        }
    }

    fn check_empty_zones(&self, result: &mut ValidationResult) {
        let empty_zones = self.find_large_empty_zones();

        for zone in empty_zones {
            if zone.size > (self.width * self.height / 8) as usize {
                result.add_warning(&format!(
                    "Large empty zone detected at ({}, {}) with {} cells",
                    zone.center.0, zone.center.1, zone.size
                ));
            }
        }
    }

    // Helper methods

    fn get_player_position(&self) -> Option<(i32, i32)> {
        self.modules.iter()
            .find(|cell| matches!(cell.module_id, ModuleId::Player))
            .map(|cell| (cell.x, cell.y))
    }

    fn get_walkable_cells(&self) -> Vec<&ArenaCell> {
        self.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep | ModuleId::RampLow))
            .collect()
    }

    fn get_interactive_elements(&self) -> Vec<&ArenaCell> {
        self.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::InteractButtonFloor |
                ModuleId::InteractButtonWall |
                ModuleId::InteractLever |
                ModuleId::InteractBarrierEnergy |
                ModuleId::MoveTeleporterIn |
                ModuleId::MoveTeleporterOut
            ))
            .collect()
    }

    fn get_hazard_positions(&self) -> Vec<(i32, i32)> {
        self.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::HazardLavaPit |
                ModuleId::HazardLaserEmitterStatic |
                ModuleId::HazardLaserTurretRotate
            ))
            .map(|cell| (cell.x, cell.y))
            .collect()
    }

    fn get_reachable_positions(&self, start: (i32, i32)) -> HashSet<(i32, i32)> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);

        while let Some((x, y)) = queue.pop_front() {
            if visited.contains(&(x, y)) {
                continue;
            }

            visited.insert((x, y));

            // Check adjacent cells
            for (nx, ny) in self.get_adjacent_positions(x, y) {
                if !visited.contains(&(nx, ny)) {
                    if let Some(cell) = self.get_cell(nx, ny) {
                        // Consider walkable or collectible surfaces as reachable
                        if matches!(cell.module_id,
                            ModuleId::FloorStd | ModuleId::FloorLarge |
                            ModuleId::RampSteep | ModuleId::RampLow |
                            ModuleId::OrbEnergy |
                            ModuleId::InteractButtonFloor) {
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }

        visited
    }
    
    fn count_adjacent_walkable(&self, x: i32, y: i32) -> usize {
        self.get_adjacent_positions(x, y).iter()
            .filter(|&&(nx, ny)| {
                if let Some(cell) = self.get_cell(nx, ny) {
                    matches!(cell.module_id,
                        ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep | ModuleId::RampLow)
                } else {
                    false
                }
            })
            .count()
    }

    fn manhattan_distance(&self, pos1: (i32, i32), pos2: (i32, i32)) -> i32 {
        (pos1.0 - pos2.0).abs() + (pos1.1 - pos2.1).abs()
    }

    fn has_unreachable_elevated_areas(&self) -> bool {
        // This is a simplified check - in a real implementation,
        // you'd analyze the 3D structure more thoroughly
        let elevated_modules = self.modules.iter()
            .filter(|cell| matches!(cell.module_id, ModuleId::RampSteep))
            .collect::<Vec<_>>();

        // If we have ramps but no way to reach them, it's a problem
        if let Some(player_pos) = self.get_player_position() {
            let reachable = self.get_reachable_positions(player_pos);

            for elevated in elevated_modules {
                if !reachable.contains(&(elevated.x, elevated.y)) {
                    return true;
                }
            }
        }

        false
    }

    fn find_large_empty_zones(&self) -> Vec<EmptyZone> {
        let mut visited = HashSet::new();
        let mut zones = Vec::new();

        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                if !visited.contains(&(x, y)) && self.get_cell(x, y).is_none() {
                    let zone = self.flood_fill_empty_zone(x, y, &mut visited);
                    if zone.size > 4 { // Only consider zones with 4+ empty cells
                        zones.push(zone);
                    }
                }
            }
        }

        zones
    }

    fn flood_fill_empty_zone(&self, start_x: i32, start_y: i32, visited: &mut HashSet<(i32, i32)>) -> EmptyZone {
        let mut queue = VecDeque::new();
        let mut zone_cells = Vec::new();

        queue.push_back((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            if visited.contains(&(x, y)) || !self.is_valid_position(x, y) {
                continue;
            }

            if self.get_cell(x, y).is_some() {
                continue; // Not empty
            }

            visited.insert((x, y));
            zone_cells.push((x, y));

            // Add adjacent cells to queue
            for (nx, ny) in self.get_adjacent_positions(x, y) {
                queue.push_back((nx, ny));
            }
        }

        let center = if !zone_cells.is_empty() {
            let avg_x = zone_cells.iter().map(|(x, _)| *x).sum::<i32>() / zone_cells.len() as i32;
            let avg_y = zone_cells.iter().map(|(_, y)| *y).sum::<i32>() / zone_cells.len() as i32;
            (avg_x, avg_y)
        } else {
            (start_x, start_y)
        };

        EmptyZone {
            center,
            size: zone_cells.len(),
            cells: zone_cells,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
    pub critical: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            critical: Vec::new(),
        }
    }

    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    pub fn add_info(&mut self, message: &str) {
        self.info.push(message.to_string());
    }

    pub fn add_critical(&mut self, message: &str) {
        self.critical.push(message.to_string());
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.critical.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len() + self.critical.len()
    }
}

#[derive(Debug, Clone)]
pub struct EmptyZone {
    pub center: (i32, i32),
    pub size: usize,
    pub cells: Vec<(i32, i32)>,
}
