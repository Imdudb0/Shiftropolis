use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod states;
mod player;
mod arena;
mod mutations;
mod gameplay_systems;
mod input;

pub use states::*;
pub use player::*;
pub use arena::*;
pub use mutations::*;
pub use gameplay_systems::*;
pub use input::*;

use crate::core::*;
use crate::generation::*;
use crate::monitoring::*;
use crate::data::*;
use crate::mesh_generation::*;
use crate::camera::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // États du jeu
            .init_state::<GameState>()
            
            // Ressources du jeu
            .init_resource::<GameSession>()
            .init_resource::<ShiftManager>()
            .init_resource::<MutationEngine>()
            .init_resource::<DifficultySystem>()
            .init_resource::<DangerPressureSystem>()
            .init_resource::<ArenaManager>()
            .init_resource::<CameraContext>()
            .init_resource::<TouchInputState>()
            
            // Événements
            .add_event::<ShiftStartEvent>()
            .add_event::<ShiftEndEvent>()
            .add_event::<OrbCollectedEvent>()
            .add_event::<PlayerDeathEvent>()
            .add_event::<MutationAppliedEvent>()
            .add_event::<CameraModeChangeEvent>()
            
            // Systèmes par état
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            
            .add_systems(OnEnter(GameState::Loading), setup_loading)
            .add_systems(Update, loading_system.run_if(in_state(GameState::Loading)))
            .add_systems(OnExit(GameState::Loading), cleanup_loading)
            
            .add_systems(OnEnter(GameState::Playing), (
                setup_game_session,
                spawn_player,
                setup_camera_for_gameplay,
            ))
            .add_systems(Update, (
                // Systèmes d'entrée
                handle_touch_input,
                handle_keyboard_input,
                
                // Systèmes du joueur
                player_movement_system,
                player_jump_system,
                player_collision_system,
                
                // Systèmes de gameplay
                shift_management_system,
                countdown_system,
                orb_collection_system,
                pressure_system,
                mutation_application_system,
                
                // Systèmes de l'arène
                arena_regeneration_system,
                dynamic_hazards_system,
                
                // Systèmes de caméra
                update_camera_context_system,
                
                // Systèmes UI
                update_survival_ui,
                update_game_info_ui,
                
            ).run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup_game_session)
            
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over)
            
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(Update, pause_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu);
    }
}

// ============================================================================
// RESSOURCES ET COMPOSANTS
// ============================================================================

#[derive(Resource, Default)]
pub struct GameSession {
    pub current_shift: u32,
    pub total_orbs_collected: u32,
    pub total_time_survived: f32,
    pub difficulty_level: f32,
    pub mutations_applied: Vec<String>,
    pub high_score: u32,
}

#[derive(Resource, Default)]
pub struct ArenaManager {
    pub current_arena: Option<Arena>,
    pub arena_entity: Option<Entity>,
    pub arena_bounds: Vec3,
    pub spawned_modules: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct TouchInputState {
    pub movement_touch: Option<(Vec2, u64)>, // Position et ID du touch
    pub jump_touch: Option<u64>,
    pub last_tap_time: f32,
    pub tap_threshold: f32,
}

impl Default for TouchInputState {
    fn default() -> Self {
        Self {
            movement_touch: None,
            jump_touch: None,
            last_tap_time: 0.0,
            tap_threshold: 0.3,
        }
    }
}

// ============================================================================
// COMPOSANTS
// ============================================================================

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct LoadingUI;

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct PauseMenuUI;

#[derive(Component)]
pub struct SurvivalUI;

#[derive(Component)]
pub struct GameInfoUI;

#[derive(Component)]
pub struct Player {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
    pub last_position: Vec3,
}

#[derive(Component)]
pub struct EnergyOrb {
    pub time_value: f32,
    pub collected: bool,
}

#[derive(Component)]
pub struct ArenaModule {
    pub module_id: ModuleId,
    pub original_position: Vec3,
}

#[derive(Component)]
pub struct DynamicHazard {
    pub hazard_type: HazardType,
    pub intensity: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Component)]
pub struct FragileSurface {
    pub break_delay: f32,
    pub respawn_delay: f32,
    pub is_breaking: bool,
    pub break_timer: f32,
    pub is_broken: bool,
    pub respawn_timer: f32,
}

// ============================================================================
// ÉVÉNEMENTS
// ============================================================================

#[derive(Event)]
pub struct ShiftStartEvent {
    pub shift_number: u32,
    pub mutation_applied: String,
}

#[derive(Event)]
pub struct ShiftEndEvent {
    pub shift_number: u32,
    pub orbs_collected: u32,
    pub orbs_uncollected: u32,
}

#[derive(Event)]
pub struct OrbCollectedEvent {
    pub time_bonus: f32,
    pub orb_entity: Entity,
}

#[derive(Event)]
pub struct PlayerDeathEvent {
    pub cause: DeathCause,
}

#[derive(Event)]
pub struct MutationAppliedEvent {
    pub mutation_name: String,
    pub intensity: f32,
}

#[derive(Debug)]
pub enum DeathCause {
    CountdownExpired,
    Hazard(String),
    FallOffArena,
}

#[derive(Debug)]
pub enum HazardType {
    LavaPit,
    LaserBeam,
    RotatingTurret,
    MovingWall,
}

// ============================================================================
// SYSTÈMES DE CONFIGURATION INITIALE
// ============================================================================

fn setup_main_menu(mut commands: Commands) {
    info!("Affichage du menu principal");
    commands.spawn((
        MainMenuUI,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgb(0.1, 0.1, 0.2).into(),
            ..default()
        },
    )).with_children(|parent| {
        // Titre du jeu
        parent.spawn(TextBundle::from_section(
            "SME ARENA",
            TextStyle {
                font_size: 60.0,
                color: Color::WHITE,
                ..default()
            },
        ));
        
        // Sous-titre
        parent.spawn(TextBundle::from_section(
            "Survival Mutations Engine",
            TextStyle {
                font_size: 24.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
    });
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_loading(mut commands: Commands) {
    info!("Chargement en cours...");
    commands.spawn((
        LoadingUI,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgb(0.05, 0.05, 0.1).into(),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Génération de l'arène...",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn loading_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut arena_manager: ResMut<ArenaManager>,
    mut shift_manager: ResMut<ShiftManager>,
    mut commands: Commands,
) {
    // Générer la première arène
    let mut generator = ArenaGenerator::new(Some(42));
    let mut monitor = AnomalyMonitor::new();
    
    match generator.generate_with_monitoring(12, 3, &mut monitor) {
        Ok(arena) => {
            info!("✅ Arène générée avec succès");
            arena_manager.current_arena = Some(arena);
            arena_manager.arena_bounds = Vec3::new(12.0, 5.0, 12.0);
            
            // Préparer le premier shift
            shift_manager.start_shift();
            
            next_state.set(GameState::Playing);
        }
        Err(e) => {
            error!("❌ Erreur lors de la génération de l'arène: {}", e);
            // En cas d'erreur, retour au menu
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_loading(mut commands: Commands, query: Query<Entity, With<LoadingUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game_session(
    mut commands: Commands,
    mut game_session: ResMut<GameSession>,
    mut arena_manager: ResMut<ArenaManager>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("🎮 Démarrage de la session de jeu");
    
    // Réinitialiser la session
    *game_session = GameSession::default();
    
    // Générer l'arène visuellement
    if let Some(ref arena) = arena_manager.current_arena {
        spawn_arena_visuals(commands, arena_manager, meshes, materials);
    }
    
    // Ajouter l'UI de survie
    setup_survival_ui(&mut commands);
}

fn cleanup_game_session(
    mut commands: Commands,
    mut arena_manager: ResMut<ArenaManager>,
    ui_query: Query<Entity, Or<(With<SurvivalUI>, With<GameInfoUI>)>>,
) {
    // Nettoyer l'arène
    for entity in arena_manager.spawned_modules.drain(..) {
        commands.entity(entity).despawn_recursive();
    }
    
    if let Some(arena_entity) = arena_manager.arena_entity {
        commands.entity(arena_entity).despawn_recursive();
    }
    
    // Nettoyer l'UI
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game_over(mut commands: Commands) {
    commands.spawn((
        GameOverUI,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: Color::srgba(0.1, 0.0, 0.0, 0.8).into(),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "GAME OVER",
            TextStyle {
                font_size: 48.0,
                color: Color::srgb(1.0, 0.2, 0.2),
                ..default()
            },
        ));
    });
}

fn cleanup_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_pause_menu(mut commands: Commands) {
    commands.spawn((
        PauseMenuUI,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.5).into(),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "PAUSE",
            TextStyle {
                font_size: 36.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn cleanup_pause_menu(mut commands: Commands, query: Query<Entity, With<PauseMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn game_over_input(
    input: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Enter) || touches.any_just_pressed() {
        next_state.set(GameState::MainMenu);
    }
}

fn pause_input(
    input: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::Space) || touches.any_just_pressed() {
        next_state.set(GameState::Playing);
    }
}
