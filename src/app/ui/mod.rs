use bevy::prelude::*;
use crate::app::game::*;
use log::info;

use crate::app::core::gameplay::ShiftManager;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_ui_resources)
            .add_systems(Update, (
                update_survival_ui,
                update_game_info_ui,
                mutation_notification_system,
                screen_flash_system,
            ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct UIResources {
    pub font: Handle<Font>,
    pub survival_ui_entity: Option<Entity>,
    pub game_info_entity: Option<Entity>,
    pub notification_entity: Option<Entity>,
}

#[derive(Component)]
pub struct SurvivalCountdownText;

#[derive(Component)]
pub struct SurvivalBarFill;

#[derive(Component)]
pub struct ShiftNumberText;

#[derive(Component)]
pub struct DifficultyText;

#[derive(Component)]
pub struct OrbCountText;

#[derive(Component)]
pub struct MutationNotification {
    pub duration: f32,
    pub remaining_time: f32,
}

#[derive(Component)]
pub struct ScreenFlash {
    pub color: Color,
    pub intensity: f32,
    pub duration: f32,
    pub remaining_time: f32,
}

impl Default for UIResources {
    fn default() -> Self {
        Self {
            font: Handle::default(),
            survival_ui_entity: None,
            game_info_entity: None,
            notification_entity: None,
        }
    }
}

pub fn setup_ui_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ui_resources = UIResources {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"), // Font par défaut de Bevy
        survival_ui_entity: None,
        game_info_entity: None,
        notification_entity: None,
    };
    
    commands.insert_resource(ui_resources);
}

pub fn setup_survival_ui(commands: &mut Commands) {
    info!("🎨 Configuration de l'UI de survie");
    
    let survival_ui = commands.spawn((
        SurvivalUI,
        Node::default(),
        Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
    )).with_children(|parent| {
        // Barre de survie (haut de l'écran)
        parent.spawn((
            Node::default(),
            Style {
                width: Val::Percent(80.0),
                height: Val::Px(60.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(10.0),
                top: Val::Px(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        )).with_children(|parent| {
            // Texte countdown
            parent.spawn((
                SurvivalCountdownText,
                Text::new("30.0s"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        // Barre de vie visuelle
        parent.spawn((
            Node::default(),
            Style {
                width: Val::Percent(60.0),
                height: Val::Px(20.0),
                position_type: PositionType::Absolute,
                left: Val::Percent(20.0),
                top: Val::Px(90.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
        )).with_children(|parent| {
            parent.spawn((
                SurvivalBarFill,
                Node::default(),
                Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.9, 1.0)), // Cyan électrique
            ));
        });

        // Informations de jeu (coin haut-droit)
        parent.spawn((
            GameInfoUI,
            Node::default(),
            Style {
                width: Val::Px(300.0),
                height: Val::Px(150.0),
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        )).with_children(|parent| {
            // Numéro de shift
            parent.spawn((
                ShiftNumberText,
                Text::new("SHIFT #1"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.9, 1.0)),
            ));

            // Niveau de difficulté
            parent.spawn((
                DifficultyText,
                Text::new("DIFFICULTY: 1.0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.0)),
            ));

            // Nombre d'orbes
            parent.spawn((
                OrbCountText,
                Text::new("ORBS: 0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 1.0, 0.0)),
            ));
        });
    }).id();
}

pub fn update_survival_ui(
    shift_manager: Res<ShiftManager>,
    mut countdown_query: Query<(&mut Text, &mut TextColor), With<SurvivalCountdownText>>,
    mut bar_query: Query<&mut Style, With<SurvivalBarFill>>,
) {
    // Mettre à jour le texte du countdown
    for (mut text, mut text_color) in countdown_query.iter_mut() {
        let remaining = shift_manager.survival_countdown.remaining_time.as_secs_f32();
        let color = if remaining > 10.0 {
            Color::WHITE
        } else if remaining > 5.0 {
            Color::srgb(1.0, 0.8, 0.0) // Orange
        } else {
            Color::srgb(1.0, 0.2, 0.2) // Rouge
        };

        **text = format!("{:.1}s", remaining);
        text_color.0 = color;
    }

    // Mettre à jour la barre de survie
    for mut style in bar_query.iter_mut() {
        let percentage = shift_manager.survival_countdown.get_percentage();
        style.width = Val::Percent(percentage * 100.0);
    }
}

pub fn update_game_info_ui(
    game_session: Res<GameSession>,
    shift_manager: Res<ShiftManager>,
    mut shift_query: Query<&mut Text, With<ShiftNumberText>>,
    mut difficulty_query: Query<&mut Text, (With<DifficultyText>, Without<ShiftNumberText>)>,
    mut orb_query: Query<&mut Text, (With<OrbCountText>, Without<ShiftNumberText>, Without<DifficultyText>)>,
) {
    // Mettre à jour le numéro de shift
    for mut text in shift_query.iter_mut() {
        **text = format!("SHIFT #{}", shift_manager.current_shift);
    }

    // Mettre à jour la difficulté
    for mut text in difficulty_query.iter_mut() {
        **text = format!("DIFFICULTY: {:.1}", game_session.difficulty_level);
    }

    // Mettre à jour le nombre d'orbes
    for mut text in orb_query.iter_mut() {
        **text = format!("ORBS: {}", game_session.total_orbs_collected);
    }
}

// ============================================================================
// SYSTÈMES DE NOTIFICATIONS ET EFFETS VISUELS
// ============================================================================

pub fn mutation_notification_system(
    mut commands: Commands,
    mut mutation_events: EventReader<MutationAppliedEvent>,
    time: Res<Time>,
    mut notification_query: Query<(Entity, &mut MutationNotification, &mut Text, &mut Style)>,
) {
    let dt = time.delta_seconds();

    // Mettre à jour les notifications existantes
    for (entity, mut notification, mut text, mut style) in notification_query.iter_mut() {
        notification.remaining_time -= dt;
        
        if notification.remaining_time <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            // Effet de fade out
            let alpha = notification.remaining_time / notification.duration;
            style.left = Val::Percent(50.0 - (1.0 - alpha) * 20.0); // Glissement vers la gauche
        }
    }

    // Créer de nouvelles notifications
    for event in mutation_events.read() {
        commands.spawn((
            MutationNotification {
                duration: 3.0,
                remaining_time: 3.0,
            },
            Node::default(),
            Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new(format!("🔥 {} 🔥", event.mutation_name.to_uppercase())),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.67)), // Magenta fluo
            ));
            
            parent.spawn((
                Text::new(format!("Intensité: {:.0}%", event.intensity * 100.0)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.9, 1.0)), // Cyan électrique
            ));
        });

        info!("🎆 Notification de mutation: {}", event.mutation_name);
    }
}

pub fn screen_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut flash_query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
    mut death_events: EventReader<PlayerDeathEvent>,
    mut orb_events: EventReader<OrbCollectedEvent>,
) {
    let dt = time.delta_seconds();

    // Mettre à jour les flashs existants
    for (entity, mut flash, mut bg_color) in flash_query.iter_mut() {
        flash.remaining_time -= dt;
        
        if flash.remaining_time <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Diminuer l'intensité progressivement
            let alpha = (flash.remaining_time / flash.duration) * flash.intensity;
            let mut color = flash.color;
            color.set_alpha(alpha);
            bg_color.0 = color;
        }
    }

    // Créer des flashs pour les événements
    for event in death_events.read() {
        // Flash rouge pour la mort
        commands.spawn((
            ScreenFlash {
                color: Color::srgba(1.0, 0.0, 0.0, 0.5),
                intensity: 0.8,
                duration: 0.5,
                remaining_time: 0.5,
            },
            Node::default(),
            Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.5)),
            ZIndex(1000),
        ));
        
        info!("💥 Flash d'écran rouge - Mort: {:?}", event.cause);
    }

    for _event in orb_events.read() {
        // Flash cyan pour les orbes collectés
        commands.spawn((
            ScreenFlash {
                color: Color::srgba(0.0, 0.9, 1.0, 0.2),
                intensity: 0.3,
                duration: 0.2,
                remaining_time: 0.2,
            },
            Node::default(),
            Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.9, 1.0, 0.2)),
            ZIndex(999),
        ));
    }
}

// ============================================================================
// SYSTÈMES D'UI CONTEXTUELLE
// ============================================================================

#[derive(Component)]
pub struct ContextualHint {
    pub hint_type: HintType,
    pub duration: f32,
    pub remaining_time: f32,
}

#[derive(Debug, Clone)]
pub enum HintType {
    FirstOrb,
    DangerApproaching,
    MutationIncoming,
    LowHealth,
    DoubleJump,
}

pub fn contextual_hints_system(
    mut commands: Commands,
    time: Res<Time>,
    game_session: Res<GameSession>,
    shift_manager: Res<ShiftManager>,
    player_query: Query<&Player>,
    mut hint_query: Query<(Entity, &mut ContextualHint, &mut TextColor)>,
) {
    let dt = time.delta_seconds();

    // Mettre à jour les hints existants
    for (entity, mut hint, mut text_color) in hint_query.iter_mut() {
        hint.remaining_time -= dt;
        
        if hint.remaining_time <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            // Effet de clignotement
            let alpha = (hint.remaining_time * 3.0).sin().abs();
            text_color.0.set_alpha(alpha);
        }
    }

    // Vérifier les conditions pour de nouveaux hints
    if let Ok(player) = player_query.get_single() {
        // Hint de santé faible
        if player.health < 30.0 && !hint_query.iter().any(|(_, hint, _)| matches!(hint.hint_type, HintType::LowHealth)) {
            spawn_hint(&mut commands, HintType::LowHealth, "⚠️ SANTÉ FAIBLE ⚠️", 2.0);
        }

        // Hint de countdown faible
        let remaining_time = shift_manager.survival_countdown.remaining_time.as_secs_f32();
        if remaining_time < 5.0 && remaining_time > 0.0 && 
           !hint_query.iter().any(|(_, hint, _)| matches!(hint.hint_type, HintType::DangerApproaching)) {
            spawn_hint(&mut commands, HintType::DangerApproaching, "⏰ TEMPS CRITIQUE!", 1.5);
        }

        // Hint de premier orbe
        if game_session.total_orbs_collected == 0 && game_session.current_shift == 1 &&
           !hint_query.iter().any(|(_, hint, _)| matches!(hint.hint_type, HintType::FirstOrb)) {
            spawn_hint(&mut commands, HintType::FirstOrb, "💎 COLLECTEZ LES ORBES POUR SURVIVRE!", 4.0);
        }
    }
}

fn spawn_hint(commands: &mut Commands, hint_type: HintType, text: &str, duration: f32) {
    commands.spawn((
        ContextualHint {
            hint_type,
            duration,
            remaining_time: duration,
        },
        Node::default(),
        Style {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 0.0)), // Jaune vif
        ));
    });
}

// ============================================================================
// SYSTÈMES DE MINIMAP ET RADAR
// ============================================================================

#[derive(Component)]
pub struct Minimap;

#[derive(Component)]
pub struct MinimapElement {
    pub element_type: MinimapElementType,
    pub world_position: Vec3,
}

#[derive(Debug, Clone)]
pub enum MinimapElementType {
    Player,
    EnergyOrb,
    Hazard,
    Exit,
}

pub fn setup_minimap(mut commands: Commands) {
    commands.spawn((
        Minimap,
        Node::default(),
        Style {
            width: Val::Px(150.0),
            height: Val::Px(150.0),
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(180.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        BorderColor(Color::srgb(0.0, 0.9, 1.0)),
    )).with_children(|parent| {
        // Zone de contenu de la minimap
        parent.spawn((
            Node::default(),
            Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
    });
}

pub fn update_minimap_system(
    player_query: Query<&Transform, (With<Player>, Without<MinimapElement>)>,
    orb_query: Query<&Transform, (With<EnergyOrb>, Without<Player>, Without<MinimapElement>)>,
    hazard_query: Query<&Transform, (With<DynamicHazard>, Without<Player>, Without<EnergyOrb>, Without<MinimapElement>)>,
    mut minimap_query: Query<&Children, With<Minimap>>,
    mut element_query: Query<(&mut MinimapElement, &mut Style)>,
    mut commands: Commands,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let player_pos = player_transform.translation;
        
        // Mettre à jour les éléments existants
        for children in minimap_query.iter_mut() {
            for &child in children.iter() {
                if let Ok((mut element, mut style)) = element_query.get_mut(child) {
                    // Convertir la position monde en position minimap
                    let relative_pos = element.world_position - player_pos;
                    let minimap_scale = 0.1; // 10% de la taille réelle
                    
                    let minimap_x = (relative_pos.x * minimap_scale + 75.0).clamp(0.0, 150.0);
                    let minimap_z = (relative_pos.z * minimap_scale + 75.0).clamp(0.0, 150.0);
                    
                    style.left = Val::Px(minimap_x);
                    style.top = Val::Px(minimap_z);
                }
            }
        }
    }
}

// ============================================================================
// SYSTÈMES D'EFFETS PARTICULLAIRES UI
// ============================================================================

#[derive(Component)]
pub struct UIParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub remaining_time: f32,
    pub start_size: f32,
    pub end_size: f32,
    pub start_color: Color,
    pub end_color: Color,
}

pub fn ui_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut particle_query: Query<(Entity, &mut UIParticle, &mut Style, &mut BackgroundColor)>,
    mut orb_events: EventReader<OrbCollectedEvent>,
) {
    let dt = time.delta_seconds();

    // Mettre à jour les particules existantes
    for (entity, mut particle, mut style, mut bg_color) in particle_query.iter_mut() {
        particle.remaining_time -= dt;
        
        if particle.remaining_time <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Progression de 0.0 à 1.0
        let progress = 1.0 - (particle.remaining_time / particle.lifetime);
        
        // Interpolation des propriétés
        let current_size = particle.start_size + (particle.end_size - particle.start_size) * progress;
        let current_color = particle.start_color.mix(&particle.end_color, progress);
        
        // Mise à jour du style
        style.width = Val::Px(current_size);
        style.height = Val::Px(current_size);
        bg_color.0 = current_color;
        
        // Mouvement
        if let Val::Px(left) = style.left {
            style.left = Val::Px(left + particle.velocity.x * dt);
        }
        if let Val::Px(top) = style.top {
            style.top = Val::Px(top + particle.velocity.y * dt);
        }
    }

    // Créer des particules pour les événements
    for event in orb_events.read() {
        // Explosion de particules cyan pour les orbes collectés
        for _ in 0..8 {
            use std::f32::consts::PI;
            let angle = fastrand::f32() * 2.0 * PI;
            let speed = 50.0 + fastrand::f32() * 100.0;
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            
            commands.spawn((
                UIParticle {
                    velocity,
                    lifetime: 1.0,
                    remaining_time: 1.0,
                    start_size: 8.0,
                    end_size: 2.0,
                    start_color: Color::srgb(0.0, 0.9, 1.0),
                    end_color: Color::srgba(0.0, 0.9, 1.0, 0.0),
                },
                Node::default(),
                Style {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(960.0), // Centre de l'écran
                    top: Val::Px(540.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.0, 0.9, 1.0)),
                ZIndex(500),
            ));
        }
    }
}