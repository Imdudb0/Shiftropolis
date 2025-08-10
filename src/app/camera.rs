use bevy::prelude::*;
use std::collections::HashMap;

// ============================================================================
// COMPOSANTS
// ============================================================================

/// Composant principal qui gère la caméra contextuelle
#[derive(Component)]
pub struct ContextualCamera {
    pub current_mode: CameraMode,
    pub target_mode: CameraMode,
    pub transition_speed: f32,
    pub transition_progress: f32,
    pub target_entity: Option<Entity>, // L'entité à suivre (joueur)
}

/// Les différents modes de caméra disponibles
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CameraMode {
    Exploration,
    Platforming,
    Danger,
    Spectacular,
    Custom(u8), // Pour des modes personnalisés
}

/// Configuration d'un mode de caméra
#[derive(Clone, Debug)]
pub struct CameraModeConfig {
    pub distance: f32,
    pub height: f32,
    pub side_offset: f32,
    pub follow_angle: f32, // En radians
    pub lerp_speed_position: f32,
    pub lerp_speed_rotation: f32,
    pub fov: f32,
    pub dutch_angle: f32, // Inclinaison de la caméra en radians
    pub look_ahead_factor: f32, // Facteur de prédiction du mouvement
}

/// Ressource qui contient toutes les configurations des modes
#[derive(Resource)]
pub struct CameraModeRegistry {
    pub configs: HashMap<CameraMode, CameraModeConfig>,
}

/// Données de contexte passées au système pour déterminer le mode approprié
#[derive(Resource, Clone, Debug, Default)]
pub struct CameraContext {
    pub player_velocity: Vec3,
    pub player_position: Vec3,
    pub distance_to_danger: f32,
    pub survival_timer: f32,
    pub max_survival_timer: f32,
    pub is_in_mutation: bool,
    pub is_near_platform: bool,
    pub orbs_in_sight: u32,
    pub is_jumping: bool,
    pub gravity_factor: f32,
    pub arena_bounds: Vec3,
    pub danger_zones: Vec<Vec3>, // Positions des zones dangereuses
}

/// Événement pour forcer un changement de mode de caméra
#[derive(Event)]
pub struct CameraModeChangeEvent {
    pub new_mode: CameraMode,
    pub transition_speed: Option<f32>,
}

// ============================================================================
// IMPLÉMENTATION DES CONFIGURATIONS PAR DÉFAUT
// ============================================================================

impl Default for CameraModeRegistry {
    fn default() -> Self {
        let mut configs = HashMap::new();

        // Mode Exploration & Collecte
        configs.insert(CameraMode::Exploration, CameraModeConfig {
            distance: 8.0,
            height: 4.0,
            side_offset: 0.0,
            follow_angle: 0.0,
            lerp_speed_position: 2.0,
            lerp_speed_rotation: 3.0,
            fov: 75.0_f32.to_radians(),
            dutch_angle: 0.0,
            look_ahead_factor: 1.5,
        });

        // Mode Plateforme & Précision
        configs.insert(CameraMode::Platforming, CameraModeConfig {
            distance: 6.0,
            height: 2.0,
            side_offset: 1.5,
            follow_angle: 0.3,
            lerp_speed_position: 4.0,
            lerp_speed_rotation: 5.0,
            fov: 70.0_f32.to_radians(),
            dutch_angle: 0.0,
            look_ahead_factor: 2.0,
        });

        // Mode Tension & Danger Imminent
        configs.insert(CameraMode::Danger, CameraModeConfig {
            distance: 5.0,
            height: 1.5,
            side_offset: 0.0,
            follow_angle: 0.0,
            lerp_speed_position: 6.0,
            lerp_speed_rotation: 8.0,
            fov: 65.0_f32.to_radians(),
            dutch_angle: 0.1, // Légère inclinaison pour le malaise
            look_ahead_factor: 0.5,
        });

        // Mode Spectaculaire & Mutation
        configs.insert(CameraMode::Spectacular, CameraModeConfig {
            distance: 15.0,
            height: 10.0,
            side_offset: 0.0,
            follow_angle: 0.0,
            lerp_speed_position: 1.0,
            lerp_speed_rotation: 1.5,
            fov: 85.0_f32.to_radians(),
            dutch_angle: 0.0,
            look_ahead_factor: 0.0,
        });

        Self { configs }
    }
}

impl Default for ContextualCamera {
    fn default() -> Self {
        Self {
            current_mode: CameraMode::Exploration,
            target_mode: CameraMode::Exploration,
            transition_speed: 2.0,
            transition_progress: 1.0,
            target_entity: None,
        }
    }
}

// ============================================================================
// LOGIQUE DE DÉTERMINATION DU MODE AUTOMATIQUE
// ============================================================================

pub struct CameraModeSelector;

impl CameraModeSelector {
    /// Détermine le mode de caméra optimal basé sur le contexte
    pub fn select_mode(context: &CameraContext) -> CameraMode {
        // Priorité 1: Mutation en cours
        if context.is_in_mutation {
            return CameraMode::Spectacular;
        }

        // Priorité 2: Danger imminent
        if context.distance_to_danger < 3.0 || 
           context.survival_timer / context.max_survival_timer < 0.2 {
            return CameraMode::Danger;
        }

        // Priorité 3: Plateforme ou saut
        if context.is_near_platform || 
           context.is_jumping || 
           context.gravity_factor.abs() > 1.5 {
            return CameraMode::Platforming;
        }

        // Mode par défaut: Exploration
        CameraMode::Exploration
    }

    /// Calcule un score de priorité pour un mode donné (plus élevé = plus prioritaire)
    pub fn calculate_mode_priority(mode: CameraMode, context: &CameraContext) -> f32 {
        match mode {
            CameraMode::Spectacular => {
                if context.is_in_mutation { 100.0 } else { 0.0 }
            }
            CameraMode::Danger => {
                let timer_urgency = 1.0 - (context.survival_timer / context.max_survival_timer);
                let danger_proximity = (5.0 - context.distance_to_danger).max(0.0) / 5.0;
                (timer_urgency * 50.0) + (danger_proximity * 30.0)
            }
            CameraMode::Platforming => {
                let mut score = 0.0;
                if context.is_near_platform { score += 20.0; }
                if context.is_jumping { score += 15.0; }
                if context.gravity_factor.abs() > 1.0 { 
                    score += context.gravity_factor.abs() * 10.0; 
                }
                score
            }
            CameraMode::Exploration => 10.0, // Score de base
            CameraMode::Custom(_) => 0.0,
        }
    }
}

// ============================================================================
// SYSTÈMES BEVY
// ============================================================================

/// Système principal qui met à jour la caméra contextuelle
pub fn update_contextual_camera_system(
    time: Res<Time>,
    registry: Res<CameraModeRegistry>,
    mut camera_query: Query<(
        &mut ContextualCamera,
        &mut Transform,
        &mut Projection,
    )>,
    target_query: Query<&Transform, (With<Transform>, Without<ContextualCamera>)>,
    mut mode_events: EventReader<CameraModeChangeEvent>,
) {
    let dt = time.delta_secs();

    for (mut contextual_camera, mut camera_transform, mut projection) in camera_query.iter_mut() {
        // Traitement des événements de changement de mode
        for event in mode_events.read() {
            contextual_camera.target_mode = event.new_mode;
            contextual_camera.transition_progress = 0.0;
            if let Some(speed) = event.transition_speed {
                contextual_camera.transition_speed = speed;
            }
        }

        // Vérification de l'entité cible
        let target_transform = if let Some(target_entity) = contextual_camera.target_entity {
            target_query.get(target_entity).ok()
        } else {
            None
        };

        if let Some(target_transform) = target_transform {
            update_camera_position_and_rotation(
                &mut contextual_camera,
                &mut camera_transform,
                &mut projection,
                target_transform,
                &registry,
                dt,
            );
        }
    }
}

fn update_camera_position_and_rotation(
    contextual_camera: &mut ContextualCamera,
    camera_transform: &mut Transform,
    projection: &mut Projection,
    target_transform: &Transform,
    registry: &CameraModeRegistry,
    dt: f32,
) {
    // Progression de la transition
    if contextual_camera.current_mode != contextual_camera.target_mode {
        contextual_camera.transition_progress += dt * contextual_camera.transition_speed;
        contextual_camera.transition_progress = contextual_camera.transition_progress.min(1.0);

        if contextual_camera.transition_progress >= 1.0 {
            contextual_camera.current_mode = contextual_camera.target_mode;
        }
    }

    // Obtention des configurations
    let current_config = registry.configs.get(&contextual_camera.current_mode);
    let target_config = registry.configs.get(&contextual_camera.target_mode);

    if let (Some(current_config), Some(target_config)) = (current_config, target_config) {
        // Interpolation des paramètres
        let lerp_factor = smooth_step(contextual_camera.transition_progress);
        let config = interpolate_camera_config(current_config, target_config, lerp_factor);

        // Calcul de la position cible
        let target_position = calculate_target_camera_position(
            target_transform.translation,
            &config,
        );

        // Mise à jour de la position de la caméra
        camera_transform.translation = camera_transform.translation.lerp(
            target_position,
            config.lerp_speed_position * dt,
        );

        // Calcul et application de la rotation
        let look_target = target_transform.translation + 
            Vec3::new(0.0, config.height * 0.3, 0.0); // Légèrement au-dessus du joueur

        let look_direction = (look_target - camera_transform.translation).normalize();
        let target_rotation = Quat::from_rotation_z(config.dutch_angle) *
            Quat::from_mat3(&Mat3::look_to_lh(look_direction, Vec3::Y));

        camera_transform.rotation = camera_transform.rotation.slerp(
            target_rotation,
            config.lerp_speed_rotation * dt,
        );

        // Mise à jour du FOV si c'est une caméra perspective
        if let Projection::Perspective(ref mut persp) = projection {
            persp.fov = persp.fov + (config.fov - persp.fov) * config.lerp_speed_rotation * dt;
        }
    }
}

fn calculate_target_camera_position(
    target_pos: Vec3,
    config: &CameraModeConfig,
) -> Vec3 {
    let angle = config.follow_angle;
    let offset = Vec3::new(
        config.distance * angle.cos() + config.side_offset,
        config.height,
        config.distance * angle.sin(),
    );
    
    target_pos - offset
}

fn interpolate_camera_config(
    from: &CameraModeConfig,
    to: &CameraModeConfig,
    t: f32,
) -> CameraModeConfig {
    CameraModeConfig {
        distance: from.distance.lerp(to.distance, t),
        height: from.height.lerp(to.height, t),
        side_offset: from.side_offset.lerp(to.side_offset, t),
        follow_angle: from.follow_angle.lerp(to.follow_angle, t),
        lerp_speed_position: from.lerp_speed_position.lerp(to.lerp_speed_position, t),
        lerp_speed_rotation: from.lerp_speed_rotation.lerp(to.lerp_speed_rotation, t),
        fov: from.fov.lerp(to.fov, t),
        dutch_angle: from.dutch_angle.lerp(to.dutch_angle, t),
        look_ahead_factor: from.look_ahead_factor.lerp(to.look_ahead_factor, t),
    }
}

/// Système pour la détermination automatique du mode de caméra
pub fn auto_camera_mode_system(
    mut camera_query: Query<&mut ContextualCamera>,
    context: Res<CameraContext>, // Supposé mis à jour par un autre système
) {
    for mut contextual_camera in camera_query.iter_mut() {
        let optimal_mode = CameraModeSelector::select_mode(&context);
        
        if optimal_mode != contextual_camera.target_mode {
            contextual_camera.target_mode = optimal_mode;
            contextual_camera.transition_progress = 0.0;
        }
    }
}

// ============================================================================
// UTILITAIRES
// ============================================================================

/// Fonction de lissage pour des transitions plus naturelles
fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Trait d'extension pour les interpolations Vec3
trait Vec3Lerp {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Vec3Lerp for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

// ============================================================================
// PLUGIN BEVY
// ============================================================================

pub struct ContextualCameraPlugin;

impl Plugin for ContextualCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CameraModeRegistry>()
            .init_resource::<CameraContext>()
            .add_event::<CameraModeChangeEvent>()
            .add_systems(Update, (
                update_contextual_camera_system,
                auto_camera_mode_system,
            ).chain());
    }
}

// ============================================================================
// EXEMPLE D'UTILISATION
// ============================================================================

#[cfg(feature = "example")]
mod example {
    use super::*;

    pub fn setup_example(mut commands: Commands) {
        // Création d'une caméra contextuelle
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 10.0),
            ContextualCamera {
                target_entity: None, // À définir avec l'entity du joueur
                ..default()
            },
        ));

        // Exemple de mise à jour du contexte
        commands.insert_resource(CameraContext {
            player_position: Vec3::ZERO,
            player_velocity: Vec3::ZERO,
            distance_to_danger: 10.0,
            survival_timer: 30.0,
            max_survival_timer: 60.0,
            is_in_mutation: false,
            is_near_platform: false,
            orbs_in_sight: 3,
            is_jumping: false,
            gravity_factor: 1.0,
            arena_bounds: Vec3::new(50.0, 20.0, 50.0),
            danger_zones: vec![],
        });
    }

    /// Système d'exemple pour changer manuellement de mode
    pub fn manual_camera_mode_change(
        input: Res<ButtonInput<KeyCode>>,
        mut mode_events: EventWriter<CameraModeChangeEvent>,
    ) {
        if input.just_pressed(KeyCode::Digit1) {
            mode_events.send(CameraModeChangeEvent {
                new_mode: CameraMode::Exploration,
                transition_speed: Some(3.0),
            });
        }
        if input.just_pressed(KeyCode::Digit2) {
            mode_events.send(CameraModeChangeEvent {
                new_mode: CameraMode::Platforming,
                transition_speed: Some(3.0),
            });
        }
        if input.just_pressed(KeyCode::Digit3) {
            mode_events.send(CameraModeChangeEvent {
                new_mode: CameraMode::Danger,
                transition_speed: Some(5.0),
            });
        }
        if input.just_pressed(KeyCode::Digit4) {
            mode_events.send(CameraModeChangeEvent {
                new_mode: CameraMode::Spectacular,
                transition_speed: Some(1.0),
            });
        }
    }
}
