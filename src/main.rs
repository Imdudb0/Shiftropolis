//main.rs


use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use log::{info, warn, error};
use std::time::{Duration, Instant};
use rand::Rng;

use shiftropolis::app;

mod core;
mod generation;
mod monitoring;
mod data;


#[cfg(target_os = "android")]
use bevy::winit::WinitSettings;

#[derive(Parser)]
#[command(name = "sme-test")]
#[command(about = "SME Arena Generation Testing Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a single arena and analyze it
    Generate {
        /// Arena size (NxN)
        #[arg(short, long, default_value = "10")]
        size: u32,
        /// Number of active rules
        #[arg(short, long, default_value = "3")]
        rules: u32,
        /// Seed for reproducible generation
        #[arg(long)]
        seed: Option<u64>,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Run stress test with multiple generations
    Stress {
        /// Number of arenas to generate
        #[arg(short, long, default_value = "100")]
        count: u32,
        /// Arena size range (min,max)
        #[arg(long, default_value = "8,15")]
        size_range: String,
        /// Rules range (min,max)
        #[arg(long, default_value = "2,5")]
        rules_range: String,        /// Stop on first anomaly
        #[arg(long)]
        fail_fast: bool,
    },
    /// Benchmark generation performance
    Benchmark {
        /// Duration in seconds
        #[arg(short, long, default_value = "30")]
        duration: u64,
    },
    
    /// Test gameplay systems integration
    Gameplay {
        /// Simulation duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,
        /// Arena size
        #[arg(short, long, default_value = "12")]
        size: u32,
        /// Starting difficulty
        #[arg(long, default_value = "1.0")]
        difficulty: f32,
        /// Initial countdown in seconds
        #[arg(long, default_value = "20")]
        countdown: u64,
    },
}

fn setup_game(mut commands: Commands) {
    info!("🚀 SME Arena - Démarrage du jeu");
    
    // Configuration de base de l'environnement 3D
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.3,
    });

    // Lumière directionnelle
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });
}

fn main() -> Result<()> {
    #[cfg(target_os = "android")]
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));

    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "SME Arena - Survival Mutations Engine".to_string(),
            resolution: (1920.0, 1080.0).into(),
            present_mode: bevy::window::PresentMode::AutoVsync,
            prevent_default_event_handling: false,
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            resizable: false,
            ..default()
        }),
        ..default()
    }));

    // Configuration spécifique Android
    #[cfg(target_os = "android")]
    {
        app.insert_resource(WinitSettings::mobile_defaults());
    }

    // Plugins de physique
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
       .add_plugins(RapierDebugRenderPlugin::default());

    // Plugins du jeu
    app.add_plugins((
        GamePlugin,
        UIPlugin,
        ContextualCameraPlugin,
    ));

    // Système de démarrage
    app.add_systems(Startup, setup_game);
    
    app.run();
    
    
    env_logger::init();
    let cli = Cli::parse();

    println!("{}", "🚀 SME Arena Generator Test Suite".bright_blue().bold());
    println!();

    match cli.command {
        Commands::Generate { size, rules, seed, verbose } => {
            generate_single_arena(size, rules, seed, verbose)?;
        }
        Commands::Stress { count, size_range, rules_range, fail_fast } => {
            run_stress_test(count, &size_range, &rules_range, fail_fast)?;
        }
        Commands::Benchmark { duration } => {
            run_benchmark(duration)?;
        }
        Commands::Gameplay { duration, size, difficulty, countdown } => {
            run_gameplay_test(duration, size, difficulty, countdown)?;
        }
    }

    Ok(())
}

fn generate_single_arena(size: u32, rule_count: u32, seed: Option<u64>, verbose: bool) -> Result<()> {
    println!("{}", format!("Generating {}x{} arena with {} rules", size, size, rule_count).cyan());

    let start = Instant::now();
    let mut monitor = AnomalyMonitor::new();
    let mut generator = ArenaGenerator::new(seed);

    // Generate arena
    let arena = generator.generate_with_monitoring(size, rule_count, &mut monitor)?;
    let generation_time = start.elapsed();

    // Display results
    println!();
    println!("{}", "📊 Generation Results:".green().bold());
    println!("  ⏱️  Time: {:?}", generation_time);
    println!("  📐 Size: {}x{}", arena.width, arena.height);
    println!("  🎯 Active Rules: {}", arena.active_rules.len());
    println!("  🔧 Modules: {}", arena.modules.len());
    println!("  💎 Energy Orbs: {}", arena.count_modules_by_type(&ModuleId::OrbEnergy));

    // Environmental variables
    println!();
    println!("{}", "🌍 Environmental Variables:".yellow().bold());
    for (id, value) in &arena.env_variables {
        println!("  {} = {:.2}", id, value);
    }

    // Active rules
    println!();
    println!("{}", "📋 Active Rules:".blue().bold());
    for rule in &arena.active_rules {
        println!("  • {} - {}", rule.id.to_string().bright_white(), rule.description);
    }

    // Monitoring results
    println!();
    display_monitoring_results(&monitor, verbose);

    // Arena visualization (ASCII)
    if verbose {
        println!();
        println!("{}", "🗺️  Arena Layout:".magenta().bold());
        display_arena_ascii(&arena);
    }

    Ok(())
}

fn run_stress_test(count: u32, size_range: &str, rules_range: &str, fail_fast: bool) -> Result<()> {
    println!("{}", format!("Running stress test: {} arenas", count).cyan());

    let (size_min, size_max) = parse_range(size_range)?;
    let (rules_min, rules_max) = parse_range(rules_range)?;

    let mut total_monitor = AnomalyMonitor::new();
    let mut success_count = 0;
    let mut failed_arenas = Vec::new();

    let start = Instant::now();

    for i in 0..count {
        let progress = (i as f32 / count as f32 * 100.0) as u32;
        print!("\r🔄 Progress: {}% [{}/{}]", progress, i + 1, count);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let size = rand::random::<u32>() % (size_max - size_min + 1) + size_min;
        let rules = rand::random::<u32>() % (rules_max - rules_min + 1) + rules_min;

        let mut monitor = AnomalyMonitor::new();
        let mut generator = ArenaGenerator::new(Some(i as u64));

        match generator.generate_with_monitoring(size, rules, &mut monitor) {
            Ok(_) => {
                success_count += 1;
                total_monitor.merge(monitor);
            }
            Err(e) => {
                failed_arenas.push((i, e.to_string()));
                if fail_fast {
                    println!();
                    error!("Arena {} failed: {}", i, e);
                    break;
                }
            }
        }
    }

    println!();
    let total_time = start.elapsed();

    // Results summary
    println!();
    println!("{}", "📊 Stress Test Results:".green().bold());
    println!("  ✅ Successful: {}/{} ({:.1}%)",
             success_count, count, success_count as f32 / count as f32 * 100.0);
    println!("  ❌ Failed: {}", failed_arenas.len());
    println!("  ⏱️  Total Time: {:?}", total_time);
    println!("  📈 Avg Time: {:?}", total_time / count);

    // Display failures
    if !failed_arenas.is_empty() {
        println!();
        println!("{}", "❌ Failed Arenas:".red().bold());
        for (id, error) in failed_arenas {
            println!("  Arena {}: {}", id, error);
        }
    }

    // Overall monitoring results
    println!();
    display_monitoring_results(&total_monitor, true);

    Ok(())
}

fn run_benchmark(duration: u64) -> Result<()> {
    println!("{}", format!("Running benchmark for {} seconds", duration).cyan());

    let start = Instant::now();
    let mut count: u64 = 0;
    let mut total_modules = 0;
    let mut peak_memory = 0;

    while start.elapsed().as_secs() < duration {
        let size = 8 + (count % 8);
        let rules = 2 + (count % 4);
        let mut monitor = AnomalyMonitor::new();
        let mut generator = ArenaGenerator::new(Some(count));

        if let Ok(arena) = generator.generate_with_monitoring(size as u32, rules as u32, &mut monitor) {
            total_modules += arena.modules.len();

            // Memory monitoring
            let memory_usage = get_memory_usage();
            if memory_usage > peak_memory {
                peak_memory = memory_usage;
            }
        }

        count += 1;

        if count % 10 == 0 {
            if let Some(elapsed) = start.elapsed().as_secs().checked_sub(0) {
                if elapsed > 0 {
                    let rate = count as f64 / elapsed as f64;
                    print!("\r🚀 Generated: {} arenas ({:.1}/s)", count, rate);
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
        }
    }

    println!();
    let total_time = start.elapsed();

    println!();
    println!("{}", "🏁 Benchmark Results:".green().bold());
    println!("  🎯 Total Arenas: {}", count);
    println!("  ⏱️  Total Time: {:?}", total_time);
    println!("  📈 Rate: {:.2} arenas/sec", count as f64 / total_time.as_secs_f64());
    println!("  🧩 Avg Modules: {:.1}", total_modules as f64 / count as f64);
    println!("  💾 Peak Memory: {:.1} MB", peak_memory as f64 / 1024.0 / 1024.0);

    Ok(())
}

fn parse_range(range_str: &str) -> Result<(u32, u32)> {
    let parts: Vec<&str> = range_str.split(',').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid range format. Use 'min,max'");
    }

    let min = parts[0].parse::<u32>()?;
    let max = parts[1].parse::<u32>()?;

    if min > max {
        anyhow::bail!("Min cannot be greater than max");
    }

    Ok((min, max))
}

fn display_monitoring_results(monitor: &AnomalyMonitor, verbose: bool) {
    let anomalies = monitor.get_anomalies();

    if anomalies.is_empty() {
        println!("{}", "✅ No anomalies detected!".green().bold());
    } else {
        println!("{}", format!("⚠️  {} anomalies detected:", anomalies.len()).yellow().bold());
        
        let mut by_severity = std::collections::HashMap::new();
        for anomaly in anomalies {
            *by_severity.entry(anomaly.severity.clone()).or_insert(0) += 1;
        }
        
        for (severity, count) in by_severity {
            let color = match severity {
                AnomalySeverity::Critical => "red",
                AnomalySeverity::Warning => "yellow",
                AnomalySeverity::Info => "blue",
            };
            println!("  {} {}: {}",
                     match severity {
                         AnomalySeverity::Critical => "🔴",
                         AnomalySeverity::Warning => "🟡",
                         AnomalySeverity::Info => "🔵",
                     },
                     format!("{:?}", severity).color(color),
                     count);
        }

        if verbose {
            println!();
            for anomaly in anomalies {
                let color = match anomaly.severity {
                    AnomalySeverity::Critical => "red",
                    AnomalySeverity::Warning => "yellow",
                    AnomalySeverity::Info => "blue",
                };
                println!("  {} {}: {}",
                         match anomaly.severity {
                             AnomalySeverity::Critical => "🔴",
                             AnomalySeverity::Warning => "🟡",
                             AnomalySeverity::Info => "🔵",
                         },
                         anomaly.category.color(color),
                         anomaly.message);
            }
        }
    }
}

fn display_arena_ascii(arena: &Arena) {
    // Simple ASCII representation
    for y in 0..arena.height {
        for x in 0..arena.width {
            if let Some(cell) = arena.get_cell(x as i32, y as i32) {
                let symbol = match cell.module_id {
                    ModuleId::Player => "P".bright_green(),
                    ModuleId::OrbEnergy => "O".bright_yellow(),
                    ModuleId::FloorStd => "█".white(),
                    ModuleId::FloorLarge => "▓".bright_white(),
                    ModuleId::RampSteep => "▲".cyan(),
                    ModuleId::RampLow => "△".cyan(),
                    ModuleId::WallLow => "▄".white(),
                    ModuleId::WallHigh => "█".yellow(),
                    ModuleId::PanelGlass => "▒".bright_blue(),
                    ModuleId::DecorArchMetallic => "∩".magenta(),
                    // ...
                    ModuleId::HazardLavaPit => "~".bright_red(),
                    ModuleId::HazardLaserEmitterStatic => "┃".red(),
                    _ => "?".bright_magenta(),
                };
                print!("{}", symbol);
            } else {
                print!(" ");
            }
        }
        println!();
    }
}

fn get_memory_usage() -> u64 {
    use sysinfo::{System, SystemExt, ProcessExt};
    let mut system = System::new();
    system.refresh_process(sysinfo::get_current_pid().unwrap());

    if let Some(process) = system.process(sysinfo::get_current_pid().unwrap()) {
        process.memory()
    } else {
        0
    }
}

fn run_gameplay_test(duration: u64, arena_size: u32, start_difficulty: f32, initial_countdown: u64) -> Result<()> {
    println!("{}", "🎮 SME Gameplay Systems Test".bright_cyan().bold());
    
    // Initialisation des systèmes
    let mut arena_generator = ArenaGenerator::new(Some(42));
    let mut shift_manager = ShiftManager::new(Duration::from_secs(initial_countdown));
    let mut mutation_engine = MutationEngine::new(Some(42));
    let mut difficulty_system = DifficultySystem::new();
    let mut pressure_system = DangerPressureSystem::new();
    
    // Ajuster la difficulté de départ
    difficulty_system.current_level = start_difficulty;
    
    println!("⚙️  Configuration:");
    println!("  Arena: {}x{}", arena_size, arena_size);
    println!("  Difficulté: {:.1}", start_difficulty);
    println!("  Countdown initial: {}s", initial_countdown);
    
    // Génération arène
    let start_gen = Instant::now();
    let mut monitor = AnomalyMonitor::new();
    let mut arena = arena_generator.generate_with_monitoring(arena_size, 3, &mut monitor)?;
    let gen_time = start_gen.elapsed();
    
    println!("✅ Arena générée en {:?}", gen_time);
    println!("  Modules: {}", arena.modules.len());
    println!("  Orbes: {}", arena.count_modules_by_type(&ModuleId::OrbEnergy));
    
    // Boucle principale de simulation
    let simulation_start = Instant::now();
    let mut last_update = Instant::now();
    let mut player_pos = (arena_size as f32 / 2.0, arena_size as f32 / 2.0);
    let mut shift_count = 0;
    let mut total_orbs_collected = 0;
    let mut rng = rand::thread_rng();
    
    // Démarrage du premier shift
    shift_manager.start_shift();
    shift_count += 1;
    
    println!("\n🚀 Shift #{} démarré!\n", shift_count);
    
    while simulation_start.elapsed() < Duration::from_secs(duration) {
        let now = Instant::now();
        let delta = now - last_update;
        last_update = now;
        
        // === SHIFT MANAGEMENT ===
        match shift_manager.update(delta) {
            ShiftUpdateResult::Active { countdown_result, shift_complete, time_remaining } => {
                // Gestion du countdown
                match countdown_result {
                    CountdownResult::Expired => {
                        println!("💀 {} - Countdown expiré!", "GAME OVER".red().bold());
                        break;
                    }
                    CountdownResult::Running { time_left, percentage } => {
                        // Affichage périodique du countdown
                        if time_left.as_secs() % 10 == 0 && time_left.as_millis() % 1000 < 100 {
                            let color = if percentage > 0.5 { "green" } else if percentage > 0.2 { "yellow" } else { "red" };
                            println!("⏱️  Survie: {:2}s ({:3.0}%)", 
                                     time_left.as_secs(), 
                                     percentage * 100.0);
                        }
                    }
                    CountdownResult::Paused => {}
                }
                
                // Fin de shift
                if shift_complete {
                    let uncollected = rng.gen_range(0..=3);
                    let survived_full = true;
                    
                    let shift_data = ShiftEndData {
                        uncollected_orbs: uncollected,
                        survived_full_shift: survived_full,
                        // Corrigé avec le cast 'as u32'
                        total_orbs_spawned: arena.count_modules_by_type(&ModuleId::OrbEnergy) as u32,
                    };
                    
                    // Mise à jour difficulté
                    let difficulty_update = difficulty_system.calculate_next_difficulty(&shift_data);
                    
                    println!("\n🔄 {} terminé!", format!("Shift #{}", shift_count).cyan());
                    println!("  Orbes non collectés: {}", uncollected);
                    println!("  Difficulté: {:.2} → {:.2}", 
                             difficulty_update.previous_level, 
                             difficulty_update.new_level);
                    
                    // Application mutation
                    let mutation_result = mutation_engine.apply_mutation(
                        difficulty_system.get_mutation_intensity(), 
                        &mut arena
                    );
                    
                    match mutation_result {
                        MutationResult::Applied { mutation, intensity, orbs_spawned } => {
                            println!("  🔥 Mutation: {} ({:.1}%)", 
                                     mutation.name, intensity * 100.0);
                            if orbs_spawned > 0 {
                                println!("  💎 +{} orbes ajoutés", orbs_spawned);
                                shift_manager.add_survival_time(Duration::from_secs(orbs_spawned as u64 * 2));
                            }
                        }
                        MutationResult::NoMutation => {
                            println!("  ⚠️  Aucune mutation disponible");
                        }
                    }
                    
                    // Nouveau shift
                    shift_count += 1;
                    shift_manager.start_shift();
                    println!("🚀 Shift #{} démarré!", shift_count);
                    println!();
                }
            }
            ShiftUpdateResult::Inactive => {}
        }
        
        // === MOUVEMENT JOUEUR ===
        if rng.gen_bool(0.3) { // 30% chance de mouvement par update
            let old_pos = player_pos;
            player_pos.0 += rng.gen_range(-0.8..0.8);
            player_pos.1 += rng.gen_range(-0.8..0.8);
            player_pos.0 = player_pos.0.clamp(0.0, arena_size as f32 - 1.0);
            player_pos.1 = player_pos.1.clamp(0.0, arena_size as f32 - 1.0);
            
            pressure_system.update_player_position(player_pos);
        }
        
        // === SYSTÈME DE PRESSION ===
        let pressure_result = pressure_system.update_pressure(delta);
        
        match pressure_result.pressure_level {
            PressureLevel::Critical => {
                println!("🔴 {} - Pression: {:.0} - BOUGEZ!", 
                         "DANGER CRITIQUE".red().bold(),
                         pressure_result.current_pressure);
            }
            PressureLevel::Danger => {
                if pressure_result.current_pressure as u32 % 15 == 0 {
                    println!("🟡 Pression élevée: {:.0} ({} menaces)", 
                             pressure_result.current_pressure,
                             pressure_result.active_threats);
                }
            }
            _ => {}
        }
        
        // === COLLECTE D'ORBES ===
        if rng.gen_bool(0.08) { // 8% chance par update
            let bonus = rng.gen_range(3..8);
            shift_manager.add_survival_time(Duration::from_secs(bonus));
            total_orbs_collected += 1;
            println!("💎 Orbe! +{}s de survie (total: {})", bonus, total_orbs_collected);
        }
        
        // Pause pour simulation temps réel
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // === STATISTIQUES FINALES ===
    println!("\n{}", "🏁 SIMULATION TERMINÉE".green().bold());
    println!("{}", "=".repeat(40));
    
    let final_stats = arena.get_statistics();
    let total_time = simulation_start.elapsed();
    
    println!("📊 Résultats:");
    println!("  Durée totale: {:?}", total_time);
    println!("  Shifts complétés: {}", shift_count);
    println!("  Difficulté finale: {:.2}", difficulty_system.current_level);
    println!("  Orbes collectés: {}", total_orbs_collected);
    println!("  Pression finale: {:.1}/100", pressure_system.current_pressure);
    println!("  Mutations actives: {}", mutation_engine.active_mutations.len());
    
    println!("\n🗺️  État final de l'arène:");
    println!("  Taille: {}x{}", arena.width, arena.height);
    println!("  Modules: {}", final_stats.filled_cells);
    println!("  Orbes restants: {}", final_stats.energy_orbs);
    println!("  Zones dangereuses: {}", final_stats.hazard_cells);
    println!("  Densité dangers: {:.1}%", final_stats.hazard_density * 100.0);
    
    // Affichage règles actives
    if !arena.active_rules.is_empty() {
        println!("\n📋 Règles actives:");
        for rule in &arena.active_rules {
            println!("  • {}", rule.id);
        }
    }
    
    // Affichage variables environnementales
    if !arena.env_variables.is_empty() {
        println!("\n🌍 Variables d'environnement:");
        for (var, value) in &arena.env_variables {
            println!("  {} = {:.2}", var, value);
        }
    }
    
    Ok(())
}

// Fonction utilitaire pour afficher l'état du jeu
pub fn display_game_state(
    shift_manager: &ShiftManager,
    difficulty: f32,
    pressure: f32,
    arena_stats: &ArenaStatistics,
) {
    println!("\n{}", "📊 État du jeu:".blue().bold());
    println!("  Shift: #{}", shift_manager.current_shift);
    println!("  Countdown: {:?}", shift_manager.survival_countdown.remaining_time);
    println!("  Difficulté: {:.2}", difficulty);
    println!("  Pression: {:.1}/100", pressure);
    println!("  Orbes: {}", arena_stats.energy_orbs);
    println!("  Dangers: {}", arena_stats.hazard_cells);
}
