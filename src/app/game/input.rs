use bevy::prelude::*;
use crate::app::game::*;
use log::info;

pub fn handle_touch_input(
    touches: Res<Touches>,
    mut touch_input: ResMut<TouchInputState>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();
    
    // Nettoyer les anciens touches
    if let Some((_, touch_id)) = touch_input.movement_touch {
        if !touches.iter().any(|t| t.id() == touch_id) {
            touch_input.movement_touch = None;
        }
    }
    
    if let Some(touch_id) = touch_input.jump_touch {
        if !touches.iter().any(|t| t.id() == touch_id) {
            touch_input.jump_touch = None;
        }
    }

    for touch in touches.iter() {
        let touch_pos = touch.position();
        let touch_id = touch.id();

        if touches.just_pressed(touch_id) {
            // Déterminer si c'est un touch de mouvement ou de saut
            if touch_pos.x < 960.0 { // Moitié gauche de l'écran = mouvement
                touch_input.movement_touch = Some((touch_pos, touch_id));
            } else { // Moitié droite = saut
                touch_input.jump_touch = Some(touch_id);
            }
            
            // Détecter les double-taps
            if current_time - touch_input.last_tap_time < touch_input.tap_threshold {
                // Double tap détecté - pourrait déclencher une action spéciale
                info!("Double tap détecté!");
            }
            touch_input.last_tap_time = current_time;
        }

        // Mettre à jour la position du touch de mouvement
        if let Some((_, movement_id)) = touch_input.movement_touch {
            if touch_id == movement_id {
                touch_input.movement_touch = Some((touch_pos, touch_id));
            }
        }
    }
}

pub fn handle_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Gestion des touches globales
    match current_state.get() {
        GameState::MainMenu => {
            if keyboard_input.just_pressed(KeyCode::Enter) || 
               keyboard_input.just_pressed(KeyCode::Space) {
                next_state.set(GameState::Loading);
            }
            if keyboard_input.just_pressed(KeyCode::Escape) {
                // Quitter le jeu
                std::process::exit(0);
            }
        },
        
        GameState::Playing => {
            if keyboard_input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
            }
            if keyboard_input.just_pressed(KeyCode::KeyR) {
                // Restart rapide
                next_state.set(GameState::Loading);
            }
        },
        
        GameState::Paused => {
            if keyboard_input.just_pressed(KeyCode::Escape) || 
               keyboard_input.just_pressed(KeyCode::Space) {
                next_state.set(GameState::Playing);
            }
        },
        
        GameState::GameOver => {
            if keyboard_input.just_pressed(KeyCode::Space) || 
               keyboard_input.just_pressed(KeyCode::Enter) {
                next_state.set(GameState::MainMenu);
            }
        },
        
        _ => {}
    }
}

// ============================================================================
// CONTRÔLES ADAPTATIFS POUR MOBILE
// ============================================================================

#[derive(Component)]
pub struct VirtualJoystick {
    pub center: Vec2,
    pub radius: f32,
    pub current_offset: Vec2,
    pub is_active: bool,
}

#[derive(Component)]
pub struct VirtualButton {
    pub position: Vec2,
    pub radius: f32,
    pub is_pressed: bool,
    pub action: VirtualButtonAction,
}

#[derive(Debug, Clone)]
pub enum VirtualButtonAction {
    Jump,
    Dash,
    Interact,
}

pub fn setup_virtual_controls(mut commands: Commands) {
    info!("🎮 Configuration des contrôles virtuels");
    
    // Joystick virtuel (coin bas-gauche)
    commands.spawn((
        VirtualJoystick {
            center: Vec2::new(150.0, 150.0),
            radius: 80.0,
            current_offset: Vec2::ZERO,
            is_active: false,
        },
        NodeBundle {
            style: Style {
                width: Val::Px(160.0),
                height: Val::Px(160.0),
                position_type: PositionType::Absolute,
                left: Val::Px(70.0),
                bottom: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.1).into(),
            ..default()
        },
    )).with_children(|parent| {
        // Knob du joystick
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Px(60.0),
                height: Val::Px(60.0),
                ..default()
            },
            background_color: Color::srgba(1.0, 1.0, 1.0, 0.3).into(),
            ..default()
        });
    });

    // Bouton de saut (coin bas-droit)
    commands.spawn((
        VirtualButton {
            position: Vec2::new(1920.0 - 100.0, 100.0),
            radius: 50.0,
            is_pressed: false,
            action: VirtualButtonAction::Jump,
        },
        ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(50.0),
                bottom: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::srgba(0.0, 0.9, 1.0, 0.3).into(),
            ..default()
        },
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "↑",
            TextStyle {
                font_size: 40.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

pub fn virtual_joystick_system(
    touches: Res<Touches>,
    mut joystick_query: Query<(&mut VirtualJoystick, &Node, &GlobalTransform)>,
    mut touch_input: ResMut<TouchInputState>,
) {
    for (mut joystick, node, global_transform) in joystick_query.iter_mut() {
        let joystick_rect = node.logical_rect(global_transform);
        
        // Vérifier si un touch est dans la zone du joystick
        joystick.is_active = false;
        joystick.current_offset = Vec2::ZERO;
        
        for touch in touches.iter() {
            let touch_pos = touch.position();
            
            if joystick_rect.contains(touch_pos) {
                joystick.is_active = true;
                
                // Calculer l'offset par rapport au centre
                let center = joystick_rect.center();
                let offset = touch_pos - center;
                let clamped_offset = offset.clamp_length_max(joystick.radius);
                
                joystick.current_offset = clamped_offset;
                
                // Convertir en input de mouvement
                let normalized_input = clamped_offset / joystick.radius;
                touch_input.movement_touch = Some((
                    Vec2::new(960.0 + normalized_input.x * 960.0, 540.0 + normalized_input.y * 540.0),
                    touch.id()
                ));
                
                break;
            }
        }
    }
}

pub fn virtual_button_system(
    touches: Res<Touches>,
    mut button_query: Query<(&mut VirtualButton, &Node, &GlobalTransform)>,
    mut touch_input: ResMut<TouchInputState>,
) {
    for (mut button, node, global_transform) in button_query.iter_mut() {
        let button_rect = node.logical_rect(global_transform);
        
        let was_pressed = button.is_pressed;
        button.is_pressed = false;
        
        // Vérifier si un touch est sur le bouton
        for touch in touches.iter() {
            if button_rect.contains(touch.position()) {
                button.is_pressed = true;
                
                match button.action {
                    VirtualButtonAction::Jump => {
                        if !was_pressed { // Nouveau press
                            touch_input.jump_touch = Some(touch.id());
                        }
                    },
                    _ => {}
                }
                
                break;
            }
        }
    }
}

// ============================================================================
// SYSTÈME D'ADAPTATION DES CONTRÔLES
// ============================================================================

#[derive(Resource, Default)]
pub struct ControlSettings {
    pub touch_sensitivity: f32,
    pub virtual_controls_enabled: bool,
    pub haptic_feedback_enabled: bool,
}

pub fn adaptive_controls_system(
    touches: Res<Touches>,
    mut control_settings: ResMut<ControlSettings>,
) {
    // Détecter automatiquement si on est sur mobile/tactile
    if touches.iter().count() > 0 && !control_settings.virtual_controls_enabled {
        control_settings.virtual_controls_enabled = true;
        info!("🎮 Contrôles tactiles activés");
    }
}

// ============================================================================
// RETOUR HAPTIQUE (Android)
// ============================================================================

#[cfg(target_os = "android")]
pub fn haptic_feedback_system(
    orb_events: EventReader<OrbCollectedEvent>,
    death_events: EventReader<PlayerDeathEvent>,
    control_settings: Res<ControlSettings>,
) {
    if !control_settings.haptic_feedback_enabled {
        return;
    }

    // Vibration légère pour les orbes
    if !orb_events.is_empty() {
        trigger_haptic_feedback(HapticStrength::Light);
    }

    // Vibration forte pour la mort
    if !death_events.is_empty() {
        trigger_haptic_feedback(HapticStrength::Heavy);
    }
}

#[cfg(target_os = "android")]
pub enum HapticStrength {
    Light,
    Medium,
    Heavy,
}

#[cfg(target_os = "android")]
pub fn trigger_haptic_feedback(strength: HapticStrength) {
    use jni::objects::{JClass, JObject};
    use jni::sys::jlong;
    use jni::JNIEnv;

    // Note: Implémentation simplifiée du retour haptique Android
    // Dans une vraie implémentation, vous utiliseriez le VibrationEffect API
    let duration = match strength {
        HapticStrength::Light => 50,
        HapticStrength::Medium => 100,
        HapticStrength::Heavy => 200,
    };

    info!("📳 Vibration {}ms", duration);
    // TODO: Implémenter l'appel JNI pour la vibration
}

#[cfg(not(target_os = "android"))]
pub fn haptic_feedback_system() {
    // Pas de retour haptique sur les autres plateformes
}
