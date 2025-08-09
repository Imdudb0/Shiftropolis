use crate::app::core::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnomalySeverity {
    Critical,  // Arena unplayable/broken
    Warning,   // Suboptimal but playable
    Info,      // Unusual but not problematic
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub category: String,
    pub message: String,
    pub severity: AnomalySeverity,
    pub detected_at: Instant,
    pub context: Option<serde_json::Value>,
}

pub struct AnomalyMonitor {
    anomalies: Vec<Anomaly>,
    metrics: HashMap<String, f64>,
    rules_applied: Vec<RuleId>,
    generation_start: Option<Instant>,
}

impl AnomalyMonitor {
    pub fn new() -> Self {
        Self {
            anomalies: Vec::new(),
            metrics: HashMap::new(),
            rules_applied: Vec::new(),
            generation_start: None,
        }
    }

    pub fn start_generation(&mut self) {
        self.generation_start = Some(Instant::now());
    }

    pub fn record_rule_application(&mut self, rule_id: RuleId) {
        self.rules_applied.push(rule_id);
    }

    pub fn record_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    pub fn check_generation_time(&mut self) {
        if let Some(start) = self.generation_start {
            let elapsed = start.elapsed();
            self.record_metric("generation_time_ms", elapsed.as_millis() as f64);

            if elapsed > Duration::from_secs(5) {
                self.report_anomaly(
                    "PERFORMANCE",
                    format!("Generation took {:.2}s (expected < 5s)", elapsed.as_secs_f64()),
                    AnomalySeverity::Warning,
                    None,
                );
            } else if elapsed > Duration::from_secs(10) {
                self.report_anomaly(
                    "PERFORMANCE",
                    format!("Generation took {:.2}s (critically slow)", elapsed.as_secs_f64()),
                    AnomalySeverity::Critical,
                    None,
                );
            }
        }
    }

    pub fn validate_arena(&mut self, arena: &Arena) {
        self.check_structural_integrity(arena);
        self.check_rule_compatibility(arena);
        self.check_gameplay_balance(arena);
        self.check_module_distribution(arena);
        self.check_environmental_variables(arena);
        self.check_spatial_coherence(arena);
    }

    fn check_structural_integrity(&mut self, arena: &Arena) {
        let issues = arena.validate_structural_integrity();
        for issue in issues {
            self.report_anomaly(
                "STRUCTURAL",
                issue,
                AnomalySeverity::Critical,
                None,
            );
        }

        // Check arena bounds
        for cell in &arena.modules {
            if cell.x < 0 || cell.y < 0 ||
               cell.x >= arena.width as i32 || cell.y >= arena.height as i32 {
                self.report_anomaly(
                    "BOUNDS",
                    format!("Module at ({}, {}) outside arena bounds {}x{}",
                           cell.x, cell.y, arena.width, arena.height),
                    AnomalySeverity::Critical,
                    None,
                );
            }
        }
    }

    fn check_rule_compatibility(&mut self, arena: &Arena) {
        // Check for conflicting rules
        for (i, rule1) in arena.active_rules.iter().enumerate() {
            for rule2 in arena.active_rules.iter().skip(i + 1) {
                if rule1.incompatible_with.contains(&rule2.id) {
                    self.report_anomaly(
                        "RULES",
                        format!("Incompatible rules active: {:?} and {:?}", rule1.id, rule2.id),
                        AnomalySeverity::Critical,
                        None,
                    );
                }
            }
        }

        // Check rule-environment consistency
        for rule in &arena.active_rules {
            match rule.id {
                RuleId::MoonGravity => {
                    if let Some(&gravity) = arena.env_variables.get(&EnvVarId::Gravity) {
                        if gravity > 0.5 {
                            self.report_anomaly(
                                "RULES",
                                format!("Moon Gravity rule active but gravity is {:.2} (expected < 0.5)", gravity),
                                AnomalySeverity::Warning,
                                None,
                            );
                        }
                    }
                }
                RuleId::LavaFloor => {
                    let lava_count = arena.count_modules_by_type(&ModuleId::HazardLavaPit);
                    if lava_count == 0 {
                        self.report_anomaly(
                            "RULES",
                            "Lava Floor rule active but no lava pits found".to_string(),
                            AnomalySeverity::Warning,
                            None,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn check_gameplay_balance(&mut self, arena: &Arena) {
        let total_cells = (arena.width * arena.height) as f64;

        // Energy orb density
        let orb_count = arena.count_modules_by_type(&ModuleId::OrbEnergy) as f64;
        let orb_density = orb_count / total_cells;

        self.record_metric("orb_density", orb_density);

        if orb_density < 0.05 {
            self.report_anomaly(
                "BALANCE",
                format!("Low energy orb density: {:.3} (recommended > 0.05)", orb_density),
                AnomalySeverity::Warning,
                None,
            );
        } else if orb_density > 0.3 {
            self.report_anomaly(
                "BALANCE",
                format!("High energy orb density: {:.3} (recommended < 0.3)", orb_density),
                AnomalySeverity::Info,
                None,
            );
        }

        // Hazard density
        let hazard_count = arena.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::HazardLavaPit |
                ModuleId::HazardLaserEmitterStatic |
                ModuleId::HazardLaserTurretRotate))
            .count() as f64;

        let hazard_density = hazard_count / total_cells;
        self.record_metric("hazard_density", hazard_density);

        if hazard_density > 0.4 {
            self.report_anomaly(
                "BALANCE",
                format!("Excessive hazard density: {:.3} (recommended < 0.4)", hazard_density),
                AnomalySeverity::Warning,
                None,
            );
        }

        // Walkable surface ratio
        let walkable_count = arena.modules.iter()
            .filter(|cell| matches!(cell.module_id,
                ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep))
            .count() as f64;

        let walkable_ratio = walkable_count / total_cells;
        self.record_metric("walkable_ratio", walkable_ratio);

        if walkable_ratio < 0.3 {
            self.report_anomaly(
                "BALANCE",
                format!("Insufficient walkable area: {:.3} (recommended > 0.3)", walkable_ratio),
                AnomalySeverity::Critical,
                None,
            );
        }
    }

    fn check_module_distribution(&mut self, arena: &Arena) {
        let mut module_counts = HashMap::new();

        for cell in &arena.modules {
            *module_counts.entry(&cell.module_id).or_insert(0) += 1;
        }

        // Check for module clustering
        self.check_spatial_clustering(arena, &module_counts);

        // Check for mandatory modules
        if !module_counts.contains_key(&ModuleId::Player) {
            self.report_anomaly(
                "MODULES",
                "Player module missing".to_string(),
                AnomalySeverity::Critical,
                None,
            );
        }

        // Check teleporter pairs
        let teleporter_in = module_counts.get(&ModuleId::MoveTeleporterIn).unwrap_or(&0);
        let teleporter_out = module_counts.get(&ModuleId::MoveTeleporterOut).unwrap_or(&0);

        if teleporter_in != teleporter_out && (*teleporter_in > 0 || *teleporter_out > 0) {
            self.report_anomaly(
                "MODULES",
                format!("Unmatched teleporters: {} in, {} out", teleporter_in, teleporter_out),
                AnomalySeverity::Warning,
                None,
            );
        }
    }

    fn check_spatial_clustering(&mut self, arena: &Arena, module_counts: &HashMap<&ModuleId, i32>) {
        // Check for excessive clustering of hazards
        for &module_id in [&ModuleId::HazardLavaPit, &ModuleId::HazardLaserEmitterStatic].iter() {
            if let Some(&count) = module_counts.get(module_id) {
                if count > 1 {
                    let positions: Vec<_> = arena.modules.iter()
                        .filter(|cell| &cell.module_id == module_id)
                        .map(|cell| (cell.x, cell.y))
                        .collect();

                    let avg_distance = self.calculate_average_distance(&positions);

                    if avg_distance < 2.0 {
                        self.report_anomaly(
                            "SPATIAL",
                            format!("{:?} modules too clustered (avg distance: {:.1})", module_id, avg_distance),
                            AnomalySeverity::Warning,
                            None,
                        );
                    }
                }
            }
        }
    }

    fn calculate_average_distance(&self, positions: &[(i32, i32)]) -> f64 {
        if positions.len() < 2 {
            return f64::INFINITY;
        }

        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..positions.len() {
            for j in i+1..positions.len() {
                let dx = (positions[i].0 - positions[j].0) as f64;
                let dy = (positions[i].1 - positions[j].1) as f64;
                total_distance += (dx * dx + dy * dy).sqrt();
                count += 1;
            }
        }

        total_distance / count as f64
    }

    fn check_environmental_variables(&mut self, arena: &Arena) {
        for (env_id, &value) in &arena.env_variables {
            let (expected_min, expected_max) = match env_id {
                EnvVarId::Gravity => (0.1, 3.0),
                EnvVarId::GameSpeed => (0.5, 2.0),
            };

            if value < expected_min || value > expected_max {
                self.report_anomaly(
                    "ENVIRONMENT",
                    format!("{:?} value {:.2} outside expected range [{:.1}, {:.1}]",
                           env_id, value, expected_min, expected_max),
                    AnomalySeverity::Warning,
                    None,
                );
            }
        }

        // Check for missing critical environmental variables
        if !arena.env_variables.contains_key(&EnvVarId::Gravity) {
            self.report_anomaly(
                "ENVIRONMENT",
                "Gravity environmental variable not set".to_string(),
                AnomalySeverity::Critical,
                None,
            );
        }
    }

    fn check_spatial_coherence(&mut self, arena: &Arena) {
        // Check for isolated modules (no adjacent walkable surfaces)
        for cell in &arena.modules {
            if matches!(cell.module_id, ModuleId::OrbEnergy | ModuleId::InteractButtonFloor) {
                let adjacent_walkable = self.count_adjacent_walkable(arena, cell.x, cell.y);

                if adjacent_walkable == 0 {
                    self.report_anomaly(
                        "SPATIAL",
                        format!("{:?} at ({}, {}) is isolated (no adjacent walkable surfaces)",
                               cell.module_id, cell.x, cell.y),
                        AnomalySeverity::Critical,
                        None,
                    );
                }
            }
        }

        // Check for unreachable areas
        self.check_reachability(arena);
    }

    fn count_adjacent_walkable(&self, arena: &Arena, x: i32, y: i32) -> usize {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut count = 0;

        for (dx, dy) in directions.iter() {
            let nx = x + dx;
            let ny = y + dy;

            if let Some(cell) = arena.get_cell(nx, ny) {
                if matches!(cell.module_id,
                    ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep) {
                    count += 1;
                }
            }
        }

        count
    }

    fn check_reachability(&mut self, arena: &Arena) {
        // Simple flood-fill from player position to check reachability
        let player_pos = arena.modules.iter()
            .find(|cell| matches!(cell.module_id, ModuleId::Player))
            .map(|cell| (cell.x, cell.y));

        if let Some(start) = player_pos {
            let reachable = self.flood_fill_reachable(arena, start);

            let orb_positions: Vec<_> = arena.modules.iter()
                .filter(|cell| matches!(cell.module_id, ModuleId::OrbEnergy))
                .map(|cell| (cell.x, cell.y))
                .collect();

            let unreachable_orbs = orb_positions.iter()
                .filter(|pos| !reachable.contains(pos))
                .count();

            /*if unreachable_orbs > 0 {
                self.report_anomaly(
                    "REACHABILITY",
                    format!("{} energy orbs are unreachable from player spawn", unreachable_orbs),
                    AnomalySeverity::Critical,
                    None,
                );
            }*/
        }
    }

    fn flood_fill_reachable(&self, arena: &Arena, start: (i32, i32)) -> std::collections::HashSet<(i32, i32)> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![start];

        while let Some((x, y)) = stack.pop() {
            if visited.contains(&(x, y)) {
                continue;
            }
            visited.insert((x, y));

            // Check adjacent cells
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
                let nx = x + dx;
                let ny = y + dy;

                if !visited.contains(&(nx, ny)) {
                    if let Some(cell) = arena.get_cell(nx, ny) {
                        // Consider walkable or collectible surfaces as reachable
                        if matches!(cell.module_id,
                            ModuleId::FloorStd | ModuleId::FloorLarge |
                            ModuleId::RampSteep | ModuleId::OrbEnergy |
                            ModuleId::InteractButtonFloor) {
                            stack.push((nx, ny));
                        }
                    }
                }
            }
        }

        visited
    }

    pub fn report_anomaly(&mut self, category: &str, message: String, severity: AnomalySeverity, context: Option<serde_json::Value>) {
        let anomaly = Anomaly {
            category: category.to_string(),
            message,
            severity,
            detected_at: Instant::now(),
            context,
        };
        self.anomalies.push(anomaly);
    }

    pub fn get_anomalies(&self) -> &Vec<Anomaly> {
        &self.anomalies
    }

    pub fn get_metrics(&self) -> &HashMap<String, f64> {
        &self.metrics
    }

    pub fn has_critical_anomalies(&self) -> bool {
        self.anomalies.iter().any(|a| a.severity == AnomalySeverity::Critical)
    }

    pub fn merge(&mut self, other: AnomalyMonitor) {
        self.anomalies.extend(other.anomalies);

        for (key, value) in other.metrics {
            *self.metrics.entry(key).or_insert(0.0) += value;
        }
    }

    pub fn get_summary(&self) -> MonitoringSummary {
        let mut by_severity = HashMap::new();
        let mut by_category = HashMap::new();

        for anomaly in &self.anomalies {
            *by_severity.entry(anomaly.severity.clone()).or_insert(0) += 1;
            *by_category.entry(anomaly.category.clone()).or_insert(0) += 1;
        }

        MonitoringSummary {
            total_anomalies: self.anomalies.len(),
            by_severity,
            by_category,
            metrics: self.metrics.clone(),
        }
    }
}

#[derive(Debug)]
pub struct MonitoringSummary {
    pub total_anomalies: usize,
    pub by_severity: HashMap<AnomalySeverity, usize>,
    pub by_category: HashMap<String, usize>,
    pub metrics: HashMap<String, f64>,
}
