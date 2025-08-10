// core/gameplay.rs - Nouveau module pour les systèmes de jeu

use bevy::prelude::*;
use crate::app::core::*;
use std::time::{Duration, Instant};
use rand::prelude::*;

// 1. GESTIONNAIRE DE SHIFT ET COMPTE À REBOURS
#[derive(Debug, Clone, Resource)]
pub struct ShiftManager {
    pub current_shift: u32,
    pub shift_duration: Duration,
    pub shift_start_time: Instant,
    pub survival_countdown: SurvivalCountdown,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct SurvivalCountdown {
    pub remaining_time: Duration,
    pub initial_time: Duration,
    pub is_running: bool,
    pub last_update: Instant,
}

impl ShiftManager {
    pub fn new(initial_countdown: Duration) -> Self {
        Self {
            current_shift: 0,
            shift_duration: Duration::from_secs(60), // 60s par shift
            shift_start_time: Instant::now(),
            survival_countdown: SurvivalCountdown::new(initial_countdown),
            is_active: false,
        }
    }

    pub fn start_shift(&mut self) -> ShiftStartResult {
        self.current_shift += 1;
        self.shift_start_time = Instant::now();
        self.survival_countdown.start();
        self.is_active = true;

        ShiftStartResult {
            shift_number: self.current_shift,
            countdown_time: self.survival_countdown.remaining_time,
            mutation_applied: true,
        }
    }

    pub fn update(&mut self, delta_time: Duration) -> ShiftUpdateResult {
        if !self.is_active {
            return ShiftUpdateResult::Inactive;
        }

        // Mise à jour du countdown de survie
        let countdown_result = self.survival_countdown.update(delta_time);

        // Vérification fin de shift
        let shift_elapsed = self.shift_start_time.elapsed();
        let shift_complete = shift_elapsed >= self.shift_duration;

        ShiftUpdateResult::Active {
            countdown_result,
            shift_complete,
            time_remaining: self.shift_duration.saturating_sub(shift_elapsed),
        }
    }

    pub fn add_survival_time(&mut self, bonus_time: Duration) {
        self.survival_countdown.add_time(bonus_time);
    }

    pub fn end_shift(&mut self) -> ShiftEndResult {
        self.is_active = false;
        let final_time = self.survival_countdown.remaining_time;
        
        ShiftEndResult {
            shift_number: self.current_shift,
            time_survived: self.shift_start_time.elapsed(),
            remaining_countdown: final_time,
        }
    }
}

impl SurvivalCountdown {
    pub fn new(initial_time: Duration) -> Self {
        Self {
            remaining_time: initial_time,
            initial_time,
            is_running: false,
            last_update: Instant::now(),
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.last_update = Instant::now();
    }

    pub fn update(&mut self, delta_time: Duration) -> CountdownResult {
        if !self.is_running {
            return CountdownResult::Paused;
        }

        if delta_time >= self.remaining_time {
            self.remaining_time = Duration::ZERO;
            self.is_running = false;
            CountdownResult::Expired
        } else {
            self.remaining_time -= delta_time;
            CountdownResult::Running {
                time_left: self.remaining_time,
                percentage: self.get_percentage(),
            }
        }
    }

    pub fn add_time(&mut self, bonus_time: Duration) {
        self.remaining_time += bonus_time;
    }

    pub fn get_percentage(&self) -> f32 {
        if self.initial_time.as_secs_f32() > 0.0 {
            self.remaining_time.as_secs_f32() / self.initial_time.as_secs_f32()
        } else {
            0.0
        }
    }
}

// 2. MACHINE À MUTATIONS (SME)
#[derive(Debug, Clone)]
pub struct MutationEngine {
    pub active_mutations: Vec<ActiveMutation>,
    pub mutation_pool: Vec<MutationTemplate>,
    pub rng: StdRng,
}

#[derive(Debug, Clone)]
pub struct ActiveMutation {
    pub template: MutationTemplate,
    pub intensity: f32, // 0.0 à 1.0
    pub start_time: Instant,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct MutationTemplate {
    pub id: MutationId,
    pub name: String,
    pub description: String,
    pub arena_changes: ArenaChanges,
    pub env_var_changes: Vec<(EnvVarId, f32)>,
    pub rule_additions: Vec<RuleId>,
    pub rule_removals: Vec<RuleId>,
}

#[derive(Debug, Clone)]
pub struct ArenaChanges {
    pub spawn_hazards: Vec<(ModuleId, u32)>, // Type, nombre à spawner
    pub remove_modules: Vec<ModuleId>,
    pub transform_modules: Vec<(ModuleId, ModuleId)>, // From, To
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationId {
    GravityInvert,
    SpeedWarp,
    LaserStorm,
    FloorIsLava,
    JumpLock,
    TimeWarp,
    OrbBlitz,
    WallPhase,
}

impl MutationEngine {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        Self {
            active_mutations: Vec::new(),
            mutation_pool: Self::create_mutation_pool(),
            rng,
        }
    }

    pub fn apply_mutation(&mut self, difficulty: f32, arena: &mut Arena) -> MutationResult {
        // Sélectionner une mutation basée sur la difficulté
        let available_mutations: Vec<_> = self.mutation_pool
            .iter()
            .filter(|m| self.is_mutation_applicable(m, difficulty))
            .collect();

        if available_mutations.is_empty() {
            return MutationResult::NoMutation;
        }

        let chosen = available_mutations
            .choose(&mut self.rng)
            .unwrap()
            .clone();
            
        let chosen = (*available_mutations.choose(&mut self.rng).unwrap()).clone();

        // Calculer l'intensité basée sur la difficulté
        let intensity = (difficulty * self.rng.gen_range(0.5..1.2)).clamp(0.0, 1.0);

        // Appliquer les changements à l'arène
        self.apply_arena_changes(&chosen.arena_changes, intensity, arena);

        // Appliquer les changements d'environnement
        for (env_var, base_value) in &chosen.env_var_changes {
            let modified_value = base_value * intensity;
            arena.env_variables.insert(env_var.clone(), modified_value.into());
        }

        // Ajouter/retirer des règles
        for rule_id in &chosen.rule_additions {
            if !arena.active_rules.iter().any(|r| &r.id == rule_id) {
                if let Some(rule) = self.create_rule_from_id(rule_id) {
                    arena.active_rules.push(rule);
                }
            }
        }

        arena.active_rules.retain(|r| !chosen.rule_removals.contains(&r.id));

        // Enregistrer la mutation active
        let active_mutation = ActiveMutation {
            template: chosen.clone(),
            intensity,
            start_time: Instant::now(),
            duration: Duration::from_secs(60),
        };

        self.active_mutations.push(active_mutation);

        MutationResult::Applied {
            mutation: chosen.clone(), 
            intensity,
            orbs_spawned: self.count_orbs_spawned(&chosen.arena_changes),
        }

    }

    fn apply_arena_changes(&mut self, changes: &ArenaChanges, intensity: f32, arena: &mut Arena) {
        // Spawn de nouveaux hazards/orbes
        for (module_type, base_count) in &changes.spawn_hazards {
            let count = (*base_count as f32 * intensity).ceil() as u32;
            self.spawn_modules_in_arena(arena, module_type.clone(), count);
        }

        // Suppression de modules
        for module_type in &changes.remove_modules {
            arena.modules.retain(|cell| &cell.module_id != module_type);
        }

        // Transformation de modules
        for (from_type, to_type) in &changes.transform_modules {
            for cell in &mut arena.modules {
                if &cell.module_id == from_type {
                    cell.module_id = to_type.clone();
                }
            }
        }
    }

    fn spawn_modules_in_arena(&mut self, arena: &mut Arena, module_type: ModuleId, count: u32) {
        let mut spawned = 0;
        let max_attempts = count * 10;
        let mut attempts = 0;

        while spawned < count && attempts < max_attempts {
            let x = self.rng.gen_range(0..arena.width as i32);
            let y = self.rng.gen_range(0..arena.height as i32);

            // Vérifier si la position est libre
            if arena.get_cell(x, y).is_none() {
                arena.modules.push(ArenaCell {
                    x,
                    y,
                    module_id: module_type.clone(),
                    module_params: None,
                    connections: Vec::new(),
                });

                spawned += 1;
            }
            attempts += 1;
        }
    }

    fn create_mutation_pool() -> Vec<MutationTemplate> {
        vec![
            MutationTemplate {
                id: MutationId::GravityInvert,
                name: "Gravity Invert".to_string(),
                description: "Gravity is inverted!".to_string(),
                arena_changes: ArenaChanges {
                    spawn_hazards: vec![(ModuleId::OrbEnergy, 3)],
                    remove_modules: vec![],
                    transform_modules: vec![],
                },
                env_var_changes: vec![(EnvVarId::Gravity, -0.8)],
                rule_additions: vec![RuleId::MoonGravity],
                rule_removals: vec![],
            },
            MutationTemplate {
                id: MutationId::LaserStorm,
                name: "Laser Storm".to_string(),
                description: "Laser turrets everywhere!".to_string(),
                arena_changes: ArenaChanges {
                    spawn_hazards: vec![
                        (ModuleId::HazardLaserTurretRotate, 4),
                        (ModuleId::OrbEnergy, 5),
                    ],
                    remove_modules: vec![],
                    transform_modules: vec![],
                },
                env_var_changes: vec![],
                rule_additions: vec![RuleId::ProjectileRain],
                rule_removals: vec![],
            },
            MutationTemplate {
                id: MutationId::FloorIsLava,
                name: "Floor is Lava".to_string(),
                description: "The floor burns!".to_string(),
                arena_changes: ArenaChanges {
                    spawn_hazards: vec![(ModuleId::HazardLavaPit, 6)],
                    remove_modules: vec![],
                    transform_modules: vec![
                        (ModuleId::FloorStd, ModuleId::HazardLavaPit),
                    ],
                },
                env_var_changes: vec![],
                rule_additions: vec![RuleId::LavaFloor],
                rule_removals: vec![],
            },
        ]
    }

    fn is_mutation_applicable(&self, _mutation: &MutationTemplate, _difficulty: f32) -> bool {
        true // Simplifié pour l'exemple
    }

    fn create_rule_from_id(&self, rule_id: &RuleId) -> Option<Rule> {
        // Créer une règle basique - dans un vrai jeu, vous auriez une base de données de règles
        Some(Rule {
            id: rule_id.clone(),
            name: format!("{:?} rule", rule_id),
            description: format!("Dynamic rule for {:?}", rule_id),
            tags: vec![],
            parameters: None,
            incompatible_with: vec![],
        })
    }

    fn count_orbs_spawned(&self, changes: &ArenaChanges) -> u32 {
        changes.spawn_hazards
            .iter()
            .filter(|(module_type, _)| matches!(module_type, ModuleId::OrbEnergy))
            .map(|(_, count)| *count)
            .sum()
    }
}

// 3. SYSTÈME DE DIFFICULTÉ PROGRESSIVE
#[derive(Debug, Clone)]
pub struct DifficultySystem {
    pub current_level: f32,
    pub base_level: f32,
    pub escalation_rate: f32,
    pub uncollected_orb_penalty: f32,
    pub shift_survival_bonus: f32,
}

impl DifficultySystem {
    pub fn new() -> Self {
        Self {
            current_level: 1.0,
            base_level: 1.0,
            escalation_rate: 0.2,
            uncollected_orb_penalty: 0.15,
            shift_survival_bonus: -0.1, // Réduction si survie complète
        }
    }

    pub fn calculate_next_difficulty(&mut self, shift_result: &ShiftEndData) -> DifficultyUpdate {
        let previous_level = self.current_level;

        // Escalade de base
        self.current_level += self.escalation_rate;

        // Pénalité pour orbes non collectés
        let orb_penalty = shift_result.uncollected_orbs as f32 * self.uncollected_orb_penalty;
        self.current_level += orb_penalty;

        // Bonus si survie complète du shift
        if shift_result.survived_full_shift {
            self.current_level += self.shift_survival_bonus;
        }

        // Empêcher la difficulté de descendre trop bas
        self.current_level = self.current_level.max(self.base_level);

        DifficultyUpdate {
            previous_level: previous_level,
            new_level: self.current_level,
            orb_penalty_applied: orb_penalty,
            survival_bonus_applied: shift_result.survived_full_shift,
        }
    }

    pub fn get_mutation_intensity(&self) -> f32 {
        (self.current_level - 1.0).clamp(0.0, 3.0) / 3.0
    }

    pub fn get_spawn_multiplier(&self) -> f32 {
        1.0 + (self.current_level - 1.0) * 0.3
    }
}

// 4. SYSTÈME DE PRESSION DES DANGERS ACCRUS
#[derive(Debug, Clone)]
pub struct DangerPressureSystem {
    pub player_position: (f32, f32),
    pub last_movement_time: Instant,
    pub stationary_threshold: Duration,
    pub pressure_buildup_rate: f32,
    pub current_pressure: f32,
    pub max_pressure: f32,
    pub active_threats: Vec<ActiveThreat>,
}

#[derive(Debug, Clone)]
pub struct ActiveThreat {
    pub threat_type: ThreatType,
    pub position: (f32, f32),
    pub intensity: f32,
    pub spawn_time: Instant,
    pub lifetime: Duration,
}

#[derive(Debug, Clone)]
pub enum ThreatType {
    LaserSweep,
    LavaSpread,
    MovingWall,
    EnergyDrain,
}

impl DangerPressureSystem {
    pub fn new() -> Self {
        Self {
            player_position: (0.0, 0.0),
            last_movement_time: Instant::now(),
            stationary_threshold: Duration::from_secs(3),
            pressure_buildup_rate: 0.1,
            current_pressure: 0.0,
            max_pressure: 100.0,
            active_threats: Vec::new(),
        }
    }

    pub fn update_player_position(&mut self, new_position: (f32, f32)) -> PressureUpdateResult {
        let distance_moved = self.calculate_distance(self.player_position, new_position);
        
        if distance_moved > 0.5 { // Mouvement significatif
            self.player_position = new_position;
            self.last_movement_time = Instant::now();
            
            // Réduction de pression pour mouvement
            self.current_pressure = (self.current_pressure - 5.0).max(0.0);
            
            PressureUpdateResult::MovementDetected { pressure_reduced: true }
        } else {
            PressureUpdateResult::NoMovement
        }
    }

    pub fn update_pressure(&mut self, delta_time: Duration) -> PressureSystemResult {
        let stationary_time = self.last_movement_time.elapsed();
        
        // Augmentation de pression si stationnaire
        if stationary_time > self.stationary_threshold {
            let pressure_increase = self.pressure_buildup_rate * delta_time.as_secs_f32();
            self.current_pressure = (self.current_pressure + pressure_increase).min(self.max_pressure);
        }

        // Mise à jour des menaces actives
        self.update_threats(delta_time);

        // Spawn de nouvelles menaces si pression élevée
        let mut new_threats = Vec::new();
        if self.current_pressure > 30.0 && self.active_threats.len() < 3 {
            if let Some(threat) = self.spawn_pressure_threat() {
                new_threats.push(threat);
            }
        }

        for threat in new_threats {
            self.active_threats.push(threat);
        }

        PressureSystemResult {
            current_pressure: self.current_pressure,
            stationary_time,
            active_threats: self.active_threats.len(),
            pressure_level: self.get_pressure_level(),
        }
    }

    fn update_threats(&mut self, delta_time: Duration) {
        // Supprimer les menaces expirées
        self.active_threats.retain(|threat| {
            threat.spawn_time.elapsed() < threat.lifetime
        });

        // Mise à jour de l'intensité des menaces
        for threat in &mut self.active_threats {
            let age_ratio = threat.spawn_time.elapsed().as_secs_f32() / threat.lifetime.as_secs_f32();
            threat.intensity = (1.0 - age_ratio).max(0.0);
        }
    }

    fn spawn_pressure_threat(&mut self) -> Option<ActiveThreat> {
        use rand::seq::SliceRandom;
        let mut rng = thread_rng();
        
        let threat_types = vec![
            ThreatType::LaserSweep,
            ThreatType::LavaSpread,
            ThreatType::EnergyDrain,
        ];

        if let Some(threat_type) = threat_types.choose(&mut rng) {
            Some(ActiveThreat {
                threat_type: threat_type.clone(),
                position: self.player_position, // Centré sur le joueur
                intensity: 1.0,
                spawn_time: Instant::now(),
                lifetime: Duration::from_secs(10),
            })
        } else {
            None
        }
    }

    fn calculate_distance(&self, pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        ((pos1.0 - pos2.0).powi(2) + (pos1.1 - pos2.1).powi(2)).sqrt()
    }

    fn get_pressure_level(&self) -> PressureLevel {
        match self.current_pressure {
            p if p < 20.0 => PressureLevel::Safe,
            p if p < 50.0 => PressureLevel::Caution,
            p if p < 80.0 => PressureLevel::Danger,
            _ => PressureLevel::Critical,
        }
    }
}

// TYPES DE RÉSULTATS ET D'ÉVÉNEMENTS
#[derive(Debug, Clone)]
pub struct ShiftStartResult {
    pub shift_number: u32,
    pub countdown_time: Duration,
    pub mutation_applied: bool,
}

#[derive(Debug, Clone)]
pub enum ShiftUpdateResult {
    Inactive,
    Active {
        countdown_result: CountdownResult,
        shift_complete: bool,
        time_remaining: Duration,
    },
}

#[derive(Debug, Clone)]
pub struct ShiftEndResult {
    pub shift_number: u32,
    pub time_survived: Duration,
    pub remaining_countdown: Duration,
}

#[derive(Debug, Clone)]
pub enum CountdownResult {
    Running { time_left: Duration, percentage: f32 },
    Expired,
    Paused,
}

#[derive(Debug, Clone)]
pub enum MutationResult {
    Applied { mutation: MutationTemplate, intensity: f32, orbs_spawned: u32 },
    NoMutation,
}

#[derive(Debug, Clone)]
pub struct ShiftEndData {
    pub uncollected_orbs: u32,
    pub survived_full_shift: bool,
    pub total_orbs_spawned: u32,
}

#[derive(Debug, Clone)]
pub struct DifficultyUpdate {
    pub previous_level: f32,
    pub new_level: f32,
    pub orb_penalty_applied: f32,
    pub survival_bonus_applied: bool,
}

#[derive(Debug, Clone)]
pub enum PressureUpdateResult {
    MovementDetected { pressure_reduced: bool },
    NoMovement,
}

#[derive(Debug, Clone)]
pub struct PressureSystemResult {
    pub current_pressure: f32,
    pub stationary_time: Duration,
    pub active_threats: usize,
    pub pressure_level: PressureLevel,
}

#[derive(Debug, Clone)]
pub enum PressureLevel {
    Safe,
    Caution,
    Danger,
    Critical,
}
