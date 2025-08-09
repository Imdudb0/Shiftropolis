//generation/mod.rs


use crate::app::core::*;
use crate::app::data::*;
use crate::app::monitoring::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use anyhow::Result;

pub struct ArenaGenerator {
    rng: StdRng,
    rules_db: RulesDatabase,
    modules_db: ModulesDatabase,
    env_vars_db: EnvVarsDatabase,
}

impl ArenaGenerator {
    pub fn new(seed: Option<u64>) -> Self {
        let actual_seed = seed.unwrap_or_else(|| rand::random());
        
        Self {
            rng: StdRng::seed_from_u64(actual_seed),
            rules_db: RulesDatabase::new(),
            modules_db: ModulesDatabase::new(),
            env_vars_db: EnvVarsDatabase::new(),
        }
    }
    
    pub fn generate_with_monitoring(&mut self, size: u32, rule_count: u32, monitor: &mut AnomalyMonitor) -> Result<Arena> {
        monitor.start_generation();
        let start_time = Instant::now();
        
        // Create base arena
        let mut arena = Arena::new(size, size, self.rng.r#gen());
        
        // Step 1: Select and apply rules
        let selected_rules = self.select_compatible_rules(rule_count, monitor)?;
        arena.active_rules = selected_rules;
        
        // Step 2: Configure environmental variables
        self.configure_environment(&mut arena, monitor)?;
        
        // Step 3: Generate base layout using WFC
        self.generate_base_layout(&mut arena, monitor)?;
        
        // Step 4: Apply rule-specific modifications
        self.apply_rule_modifications(&mut arena, monitor)?;
        
        // Step 5: Place interactive elements
        self.place_interactive_elements(&mut arena, monitor)?;
        
        // Step 6: Balance and optimize
        self.balance_arena(&mut arena, monitor)?;
        
        // Record generation time
        let generation_time = start_time.elapsed();
        arena.generation_metadata.generation_time_ms = generation_time.as_millis() as u64;
        
        // Final validation
        monitor.validate_arena(&arena);
        monitor.check_generation_time();
        
        if monitor.has_critical_anomalies() {
            anyhow::bail!("Arena generation failed due to critical anomalies: {:?}", monitor.get_anomalies());
        }
        
        Ok(arena)
    }
    
    fn select_compatible_rules(&mut self, count: u32, monitor: &mut AnomalyMonitor) -> Result<Vec<Rule>> {
        // Clone the rules to release the immutable borrow on `self`
        let all_rules = self.rules_db.get_all_rules().clone();
        let mut selected = Vec::new();
        let mut incompatible_set = HashSet::new();

        // Use weighted selection based on difficulty and tags
        for _ in 0..count {
            let available_rules: Vec<Rule> = all_rules.iter()
            .filter(|rule| !incompatible_set.contains(&rule.id))
            .filter(|rule| !selected.iter().any(|s: &Rule| s.id == rule.id))
            // The Fix: Call .cloned() twice to resolve the double reference.
            .cloned() // Converts Iterator<Item=&&Rule> to Iterator<Item=&Rule>
            .cloned() // Converts Iterator<Item=&Rule> to Iterator<Item=Rule>
            .collect();

            if available_rules.is_empty() {
                monitor.report_anomaly(
                    "RULES", 
                    format!("Could only select {} rules out of {} requested", selected.len(), count),
                    AnomalySeverity::Warning,
                    None
                );
                break;
            }

            let weights: Vec<f32> = available_rules.iter()
            .map(|rule| self.calculate_rule_weight(rule, &selected))
            .collect();

            // Fix: Move the mutable borrow after all immutable borrows are done
            let selected_index = {
                let total_weight: f32 = weights.iter().sum();
                let mut target = self.rng.r#gen::<f32>() * total_weight;
            
                let mut index = 0;
                for (i, &weight) in weights.iter().enumerate() {
                    target -= weight;
                    if target <= 0.0 {
                        index = i;
                        break;
                    }
                }
                index
            };

            let selected_rule = &available_rules[selected_index];

            for incompatible in &selected_rule.incompatible_with {
                incompatible_set.insert(incompatible.clone());
            }

            monitor.record_rule_application(selected_rule.id.clone());
            selected.push(selected_rule.clone());
        }

        Ok(selected)
    }
       
    fn calculate_rule_weight(&self, rule: &Rule, current_rules: &[Rule]) -> f32 {
        let mut weight = 1.0;
        
        // Prefer diverse rule types
        let movement_rules = current_rules.iter()
            .filter(|r| r.tags.contains(&"movement".to_string()))
            .count();
        let hazard_rules = current_rules.iter()
            .filter(|r| r.tags.contains(&"hazard".to_string()))
            .count();
            
        if rule.tags.contains(&"movement".to_string()) && movement_rules > 1 {
            weight *= 0.5;
        }
        if rule.tags.contains(&"hazard".to_string()) && hazard_rules > 1 {
            weight *= 0.5;
        }
        
        // Difficulty balancing
        if rule.tags.contains(&"difficulty_easy".to_string()) {
            weight *= 1.2;
        } else if rule.tags.contains(&"difficulty_hard".to_string()) {
            weight *= 0.8;
        }
        
        weight
    }
    
    fn weighted_select<'a, T>(&mut self, items: &'a [T], weights: &[f32]) -> Result<&'a T> {
        let total_weight: f32 = weights.iter().sum();
        let mut target = self.rng.r#gen::<f32>() * total_weight;
    
        for (item, &weight) in items.iter().zip(weights.iter()) {
            target -= weight;
            if target <= 0.0 {
                return Ok(item);
            }
        }
    
        // Fallback to last item
        items.last().ok_or_else(|| anyhow::anyhow!("No items to select from"))
    }
    
    fn configure_environment(&mut self, arena: &mut Arena,     monitor: &mut AnomalyMonitor) -> Result<()> {
        let env_vars = self.env_vars_db.get_all_variables();
    
        for env_var in env_vars {
            let mut value = env_var.default_value;
            let mut rule_applied = false; // Flag pour savoir si une règle a modifié la valeur
        
            // Modify based on active rules
            for rule in &arena.active_rules {
                match (&rule.id, &env_var.id) {
                    (RuleId::MoonGravity, EnvVarId::Gravity) => {
                        value = 0.3 + self.rng.r#gen::<f64>() * 0.2; // 0.3-0.5
                        rule_applied = true;
                    }
                    (RuleId::SpeedUp, EnvVarId::GameSpeed) => {
                        value *= 1.2 + self.rng.r#gen::<f64>() * 0.3; // 1.2-1.5x
                        rule_applied = true;
                    }
                    _ => {}
                }
            }
        
            // N'ajoutez de la variance que si aucune règle spécifique n'a été appliquée
            if !rule_applied {
                let variance = (env_var.range.1 - env_var.range.0) * 0.1;
                value += (self.rng.r#gen::<f64>() - 0.5) * variance;
            }
        
            // Clamp to valid range
            value = value.clamp(env_var.range.0, env_var.range.1);
        
            arena.env_variables.insert(env_var.id.clone(), value);
            monitor.record_metric(&format!("env_{:?}", env_var.id), value);
        }
    
        Ok(())
    }
    
    fn generate_base_layout(&mut self, arena: &mut Arena, monitor: &mut AnomalyMonitor) -> Result<()> {
        // Simple WFC-inspired algorithm
        let mut wfc = WFCGenerator::new(arena.width, arena.height, &mut self.rng);
        
        // Initialize constraints based on modules database
        let modules = self.modules_db.get_all_modules();
        for module in modules {
            if let Some(weight) = module.wfc_weight {
                wfc.add_module_constraint(module.id.clone(), weight, &module.tags);
            }
        }
        
        // Add rule-specific constraints
        for rule in &arena.active_rules {
            self.add_rule_constraints(&mut wfc, rule);
        }
        
        // Generate layout
        let layout = wfc.generate(monitor)?;
        
        // Convert WFC output to arena modules
        for (pos, module_id, params) in layout {
            arena.add_module(pos.0, pos.1, module_id, params);
        }
        
        // Ensure player spawn exists
        if !arena.modules.iter().any(|m| matches!(m.module_id, ModuleId::Player)) {
            let spawn_pos = self.find_safe_spawn_location(arena);
            arena.add_module(spawn_pos.0, spawn_pos.1, ModuleId::Player, None);
        }
        
        Ok(())
    }
    
    fn add_rule_constraints(&self, wfc: &mut WFCGenerator, rule: &Rule) {
        match rule.id {
            RuleId::LavaFloor => {
                wfc.increase_module_weight(&ModuleId::HazardLavaPit, 3.0);
                wfc.add_constraint("min_lava_pits", 2);
            }
            RuleId::NoJump => {
                // Reduce vertical elements, increase ramps
                wfc.increase_module_weight(&ModuleId::RampSteep, 2.0);
                wfc.decrease_module_weight(&ModuleId::FloorLarge, 0.5);
            }
            RuleId::OrbCollection => {
                wfc.increase_module_weight(&ModuleId::OrbEnergy, 2.5);
                wfc.add_constraint("min_orbs", 5);
            }
            _ => {}
        }
    }
    
    fn find_safe_spawn_location(&mut self, arena: &Arena) -> (i32, i32) {
        // Find a floor tile that's not adjacent to hazards
        for cell in &arena.modules {
            if matches!(cell.module_id, ModuleId::FloorStd | ModuleId::FloorLarge) {
                let is_safe = !self.has_adjacent_hazard(arena, cell.x, cell.y);
                if is_safe {
                    return (cell.x, cell.y);
                }
            }
        }
        
        // Fallback: center of arena
        (arena.width as i32 / 2, arena.height as i32 / 2)
    }
    
    fn has_adjacent_hazard(&self, arena: &Arena, x: i32, y: i32) -> bool {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        
        for (dx, dy) in directions.iter() {
            let nx = x + dx;
            let ny = y + dy;
            
            if let Some(cell) = arena.get_cell(nx, ny) {
                if matches!(cell.module_id, 
                    ModuleId::HazardLavaPit | 
                    ModuleId::HazardLaserEmitterStatic) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn apply_rule_modifications(&mut self, arena: &mut Arena, monitor: &mut AnomalyMonitor) -> Result<()> {
        for rule in arena.active_rules.clone() {
            match rule.id {
                RuleId::ProjectileRain => {
                    self.add_projectile_hazards(arena)?;
                }
                RuleId::LavaFloor => {
                    self.enhance_lava_hazards(arena)?;
                }
                _ => {}
            }
            
            monitor.record_rule_application(rule.id);
        }
        
        Ok(())
    }
    
    fn add_projectile_hazards(&mut self, arena: &mut Arena) -> Result<()> {
        let count = 2 + self.rng.gen_range(0..3);
        
        for _ in 0..count {
            let x = self.rng.gen_range(0..arena.width as i32);
            let y = self.rng.gen_range(0..arena.height as i32);
            
            // Only place if position is free
            if arena.get_cell(x, y).is_none() {
                arena.add_module(x, y, ModuleId::HazardLaserTurretRotate, None);
            }
        }
        
        Ok(())
    }
    
    fn enhance_lava_hazards(&mut self, arena: &mut Arena) -> Result<()> {
        // Find existing lava pits and potentially expand them
        let lava_positions: Vec<_> = arena.modules.iter()
            .filter(|cell| matches!(cell.module_id, ModuleId::HazardLavaPit))
            .map(|cell| (cell.x, cell.y))
            .collect();
            
        for (x, y) in lava_positions {
            // 50% chance to expand each lava pit
            if self.rng.r#gen::<f32>() < 0.5 {
                let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                let (dx, dy) = directions[self.rng.gen_range(0..directions.len())];
                let nx = x + dx;
                let ny = y + dy;
                
                if nx >= 0 && ny >= 0 && 
                   nx < arena.width as i32 && ny < arena.height as i32 &&
                   arena.get_cell(nx, ny).is_none() {
                    arena.add_module(nx, ny, ModuleId::HazardLavaPit, None);
                }
            }
        }
        
        Ok(())
    }
    
    fn find_spawnable_locations(&self, arena: &Arena) -> Vec<(i32, i32)> {
        let mut valid_positions = Vec::new();
        let walkable_surfaces: std::collections::HashSet<(i32, i32)> = arena.modules.iter()
            .filter(|c| matches!(c.module_id, ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep))
            .map(|c| (c.x, c.y))
            .collect();

        for y in 0..arena.height as i32 {
            for x in 0..arena.width as i32 {
                // L'emplacement doit être vide
                if arena.get_cell(x, y).is_none() {
                    // Vérifier les voisins
                    let neighbors = [(x, y + 1), (x, y - 1), (x + 1, y), (x - 1, y)];
                    let has_walkable_neighbor = neighbors.iter().any(|pos| walkable_surfaces.contains(pos));

                    if has_walkable_neighbor {
                        valid_positions.push((x, y));
                    }
                }
            }
        }
        valid_positions
    }

    fn place_interactive_elements(&mut self, arena: &mut Arena, _monitor: &mut AnomalyMonitor) -> Result<()> {
        // Trouvons tous les emplacements possibles une seule fois.
        let mut spawnable_locations = self.find_spawnable_locations(arena);
        // Mélangeons pour obtenir des placements aléatoires
        use rand::seq::SliceRandom;
        spawnable_locations.shuffle(&mut self.rng);

        // Place energy orbs
        let orb_count = (arena.width * arena.height / 20).max(3);
        for _ in 0..orb_count {
            if let Some(pos) = spawnable_locations.pop() {
                arena.add_module(pos.0, pos.1, ModuleId::OrbEnergy, None);
            } else {
                // Plus de place disponible, on arrête
                break;
            }
        }
    
        // Place some interactive elements based on arena size
        let interactive_count = (arena.width * arena.height / 40).max(1);
        for _ in 0..interactive_count {
            if let Some(pos) = spawnable_locations.pop() {
                let element = match self.rng.gen_range(0..3) {
                    0 => ModuleId::InteractButtonFloor,
                    1 => ModuleId::InteractLever,
                    _ => ModuleId::InteractBarrierEnergy,
                };
            
                arena.add_module(pos.0, pos.1, element, None);
            } else {
                // Plus de place
                break;
            }
        }
    
        Ok(())
    }

    fn find_free_position(&mut self, arena: &Arena) -> Option<(i32, i32)> {
        for _ in 0..100 { // Max 100 attempts
            let x = self.rng.gen_range(0..arena.width as i32);
            let y = self.rng.gen_range(0..arena.height as i32);
            
            if arena.get_cell(x, y).is_none() {
                return Some((x, y));
            }
        }
        
        None
    }
    
    fn balance_arena(&mut self, arena: &mut Arena, monitor: &mut AnomalyMonitor) -> Result<()> {
        // Ensure minimum walkable area
        let walkable_count = arena.modules.iter()
            .filter(|cell| matches!(cell.module_id, 
                ModuleId::FloorStd | ModuleId::FloorLarge | ModuleId::RampSteep))
            .count();
            
        let required_walkable = (arena.width * arena.height / 3) as usize; // At least 1/3 walkable
        
        if walkable_count < required_walkable {
            let needed = required_walkable - walkable_count;
            
            for _ in 0..needed {
                if let Some(pos) = self.find_free_position(arena) {
                    arena.add_module(pos.0, pos.1, ModuleId::FloorStd, None);
                }
            }
            
            monitor.report_anomaly(
                "BALANCE",
                format!("Added {} floor tiles to meet minimum walkable area", needed),
                AnomalySeverity::Info,
                None,
            );
        }
        
        Ok(())
    }
}

// Simplified WFC generator
struct WFCGenerator {
    width: u32,
    height: u32,
    rng: *mut StdRng,
    module_weights: HashMap<ModuleId, f32>,
    constraints: HashMap<String, i32>,
}

impl WFCGenerator {
    fn new(width: u32, height: u32, rng: &mut StdRng) -> Self {
        Self {
            width,
            height,
            rng,
            module_weights: HashMap::new(),
            constraints: HashMap::new(),
        }
    }
    
    fn add_module_constraint(&mut self, module_id: ModuleId, weight: u32, _tags: &[String]) {
        self.module_weights.insert(module_id, weight as f32);
    }
    
    fn increase_module_weight(&mut self, module_id: &ModuleId, multiplier: f32) {
        let current = self.module_weights.get(module_id).unwrap_or(&1.0);
        self.module_weights.insert(module_id.clone(), current * multiplier);
    }
    
    fn decrease_module_weight(&mut self, module_id: &ModuleId, multiplier: f32) {
        let current = self.module_weights.get(module_id).unwrap_or(&1.0);
        self.module_weights.insert(module_id.clone(), current * multiplier);
    }
    
    fn add_constraint(&mut self, name: &str, value: i32) {
        self.constraints.insert(name.to_string(), value);
    }
    
    fn generate(&mut self, _monitor: &mut AnomalyMonitor) -> Result<Vec<((i32, i32), ModuleId, Option<serde_json::Value>)>> {
        let mut result = Vec::new();
        let rng = unsafe { &mut *self.rng };
        
        // Simple fill algorithm - place floors in majority of cells
        let density = 0.6; // 60% filled
        
        for y in 0..self.height {
            for x in 0..self.width {
                if rng.r#gen::<f32>() < density {
                    let module_id = self.select_weighted_module(rng);
                    result.push(((x as i32, y as i32), module_id, None));
                }
            }
        }
        
        // Ensure minimum constraints are met
        self.enforce_constraints(&mut result, rng);
        
        Ok(result)
    }
    
    fn select_weighted_module(&self, rng: &mut StdRng) -> ModuleId {
        // Store owned f32 values, not references, by dereferencing with *.
        let weights = vec![
        (ModuleId::FloorStd, *self.module_weights.get(&ModuleId::FloorStd).unwrap_or(&10.0)),
        (ModuleId::FloorLarge, *self.module_weights.get(&ModuleId::FloorLarge).unwrap_or(&5.0)),
        (ModuleId::RampSteep, *self.module_weights.get(&ModuleId::RampSteep).unwrap_or(&3.0)),
        (ModuleId::HazardLavaPit, *self.module_weights.get(&ModuleId::HazardLavaPit).unwrap_or(&2.0)),
        (ModuleId::HazardLaserEmitterStatic, *self.module_weights.get(&ModuleId::HazardLaserEmitterStatic).unwrap_or(&1.0)),
        ];
    
        // The iterator yields `&(ModuleId, f32)`, so item.1 is the f32 weight.
        let total_weight: f32 = weights.iter().map(|item| item.1).sum();
        let mut target = rng.r#gen::<f32>() * total_weight;
    
        // Use a simple pattern. `weight` is now a `&f32`.
        for (module_id, weight) in &weights {
            // Dereference the weight for the calculation.
            target -= *weight;
            if target <= 0.0 {
                return module_id.clone();
            }
        }
    
        ModuleId::FloorStd // Fallback
    }
    
    fn enforce_constraints(&self, result: &mut Vec<((i32, i32), ModuleId, Option<serde_json::Value>)>, rng: &mut StdRng) {
        // Enforce minimum lava pits
        if let Some(&min_lava) = self.constraints.get("min_lava_pits") {
            let current_lava = result.iter()
                .filter(|(_, module_id, _)| matches!(module_id, ModuleId::HazardLavaPit))
                .count();
                
            if (current_lava as i32) < min_lava {
                let needed = min_lava - current_lava as i32;
                for _ in 0..needed {
                    let x = rng.gen_range(0..self.width as i32);
                    let y = rng.gen_range(0..self.height as i32);
                    result.push(((x, y), ModuleId::HazardLavaPit, None));
                }
            }
        }
        
        // Enforce minimum orbs
        if let Some(&min_orbs) = self.constraints.get("min_orbs") {
            let current_orbs = result.iter()
                .filter(|(_, module_id, _)| matches!(module_id, ModuleId::OrbEnergy))
                .count();
                
            if (current_orbs as i32) < min_orbs {
                let needed = min_orbs - current_orbs as i32;
                for _ in 0..needed {
                    let x = rng.gen_range(0..self.width as i32);
                    let y = rng.gen_range(0..self.height as i32);
                    result.push(((x, y), ModuleId::OrbEnergy, None));
                }
            }
        }
    }
}
