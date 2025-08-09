use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::app::game::*;
use crate::app::camera::*;
use log::info;
use crate::app::game::gameplay::ShiftManager;

pub fn spawn_player(mut commands: Commands) {
    info!("👤 Spawn du joueur");
    
    commands.spawn((
        Player {
            health: 100.0,
            max_health: 100.0,
            speed: 5.0,
            jump_force: 15.0,
            is_grounded: false,
            last_position: Vec3::ZERO,
        },
        PbrBundle {
            mesh: Default::default(), // Sera remplacé par un mesh généré
            material: Default::default(),
            transform: Transform::from_xyz(6.0, 2.0, 6.0), // Centre de l'arène
            ..default()
        },
        RigidBody::Dynamic,
        Collider::capsule_y(0.5, 0.4),
        Velocity::default(),
        LockedAxes::ROTATION_LOCKED, // Empêcher la rotation du joueur
        Friction::coefficient(0.7),
        Restitution::coefficient(0.3),
        CollisionGroups::new(Group::GROUP_1, Group::ALL),
        ActiveEvents::COLLISION_EVENTS,
        Name::new("Player"),
    ));
}

pub fn setup_camera_for_gameplay(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
) {
    if let Ok(player_entity) = player_query.get_single() {
        // Caméra contextuelle qui suit le joueur
        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 8.0, 12.0)
                    .looking_at(Vec3::new(6.0, 0.0, 6.0), Vec3::Y),
                ..default()
            },
            ContextualCamera {
                target_entity: Some(player_entity),
                current_mode: CameraMode::Exploration,
                target_mode: CameraMode::Exploration,
                transition_speed: 2.0,
                transition_progress: 1.0,
            },
        ));

        info!("📷 Caméra contextuelle configurée");
    }
}

pub fn player_movement_system(
    time: Res<Time>,
    touch_input: Res<TouchInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &mut Transform, &mut Player), With<Player>>,
    arena_manager: Res<ArenaManager>,
) {
    if let Ok((mut velocity, mut transform, mut player)) = player_query.get_single_mut() {
        let dt = time.delta_seconds();
        let mut movement = Vec3::ZERO;

        // Input clavier (pour les tests)
        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            movement.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            movement.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }

        // Input tactile
        if let Some((touch_pos, _)) = touch_input.movement_touch {
            // Convertir la position tactile en mouvement
            // Note: Cela nécessiterait une conversion d'écran vers monde
            // Pour l'instant, on utilise un contrôle simplifié
            let center_x = 960.0; // Milieu de l'écran (1920/2)
            let center_y = 540.0; // Milieu de l'écran (1080/2)

            let delta_x = (touch_pos.x - center_x) / center_x;
            let delta_y = (touch_pos.y - center_y) / center_y;

            movement.x += delta_x;
            movement.z += delta_y; // Inverser Y car l'écran a Y vers le bas
        }

        // Normaliser le mouvement et appliquer la vitesse
        if movement.length() > 0.0 {
            movement = movement.normalize() * player.speed;
            velocity.linvel.x = movement.x;
            velocity.linvel.z = movement.z;
        } else {
            // Friction au sol
            velocity.linvel.x *= 0.8;
            velocity.linvel.z *= 0.8;
        }

        // Vérifier les limites de l'arène
        let bounds = arena_manager.arena_bounds;
        let pos = transform.translation;

        if pos.x < -bounds.x * 0.5 || pos.x > bounds.x * 0.5 ||
           pos.z < -bounds.z * 0.5 || pos.z > bounds.z * 0.5 {
            // Joueur sort de l'arène - téléporter ou appliquer des dégâts
            if pos.y < -5.0 {
                // Chute mortelle
                // TODO: Déclencher événement de mort
                transform.translation = Vec3::new(6.0, 2.0, 6.0); // Respawn au centre
            }
        }

        // Mettre à jour la dernière position
        player.last_position = transform.translation;
    }
}

pub fn player_jump_system(
    touch_input: Res<TouchInputState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &mut Player), With<Player>>,
) {
    if let Ok((mut velocity, mut player)) = player_query.get_single_mut() {
        let jump_input = keyboard_input.just_pressed(KeyCode::Space) ||
                        touch_input.jump_touch.is_some();

        if jump_input && player.is_grounded {
            velocity.linvel.y = player.jump_force;
            player.is_grounded = false;
            info!("🦘 Joueur saute!");
        }
    }
}

pub fn player_collision_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<(Entity, &mut Player), With<Player>>,
    orb_query: Query<&EnergyOrb, With<EnergyOrb>>,
    hazard_query: Query<&DynamicHazard, With<DynamicHazard>>,
    fragile_query: Query<&mut FragileSurface, With<FragileSurface>>,
    mut orb_collected_events: EventWriter<OrbCollectedEvent>,
    mut player_death_events: EventWriter<PlayerDeathEvent>,
    mut commands: Commands,
) {
    if let Ok((player_entity, mut player)) = player_query.get_single_mut() {
        for collision_event in collision_events.read() {
            match collision_event {
                CollisionEvent::Started(entity1, entity2, _) => {
                    let (player_entity, other_entity) = if *entity1 == player_entity {
                        (*entity1, *entity2)
                    } else if *entity2 == player_entity {
                        (*entity2, *entity1)
                    } else {
                        continue;
                    };

                    // Collision avec orbe d'énergie
                    if let Ok(orb) = orb_query.get(other_entity) {
                        if !orb.collected {
                            orb_collected_events.send(OrbCollectedEvent {
                                time_bonus: orb.time_value,
                                orb_entity: other_entity,
                            });

                            // Marquer l'orbe comme collecté et le faire disparaître
                            commands.entity(other_entity).despawn();
                            info!("💎 Orbe collecté! +{}s", orb.time_value);
                        }
                    }

                    // Collision avec danger
                    if let Ok(hazard) = hazard_query.get(other_entity) {
                        match hazard.hazard_type {
                            HazardType::LavaPit => {
                                player.health -= 25.0 * hazard.intensity;
                                if player.health <= 0.0 {
                                    player_death_events.send(PlayerDeathEvent {
                                        cause: DeathCause::Hazard("Lava Pit".to_string()),
                                    });
                                }
                            },
                            HazardType::LaserBeam => {
                                player.health -= 30.0 * hazard.intensity;
                                if player.health <= 0.0 {
                                    player_death_events.send(PlayerDeathEvent {
                                        cause: DeathCause::Hazard("Laser Beam".to_string()),
                                    });
                                }
                            },
                            _ => {},
                        }
                    }

                    // Collision avec surface fragile
                    if let Ok(mut fragile) = fragile_query.get_mut(other_entity) {
                        if !fragile.is_breaking && !fragile.is_broken {
                            fragile.is_breaking = true;
                            info!("💥 Surface fragile activée!");
                        }
                    }

                    // Détection du sol pour le saut
                    // TODO: Améliorer la détection du sol avec les normales de collision
                    player.is_grounded = true;
                },
                CollisionEvent::Stopped(entity1, entity2, _) => {
                    let (_player_entity, _other_entity) = if *entity1 == player_entity {
                        (*entity1, *entity2)
                    } else if *entity2 == player_entity {
                        (*entity2, *entity1)
                    } else {
                        continue;
                    };

                    // Le joueur quitte le sol
                    // Note: Cette logique est simplifiée, il faudrait vérifier
                    // si on quitte vraiment le sol ou juste un autre objet
                    player.is_grounded = false;
                },
            }
        }
    }
}

pub fn update_camera_context_system(
    player_query: Query<(&Transform, &Velocity, &Player), With<Player>>,
    orb_query: Query<&Transform, (With<EnergyOrb>, Without<Player>)>,
    hazard_query: Query<&Transform, (With<DynamicHazard>, Without<Player>)>,
    shift_manager: Res<ShiftManager>,
    mut camera_context: ResMut<CameraContext>,
) {
    if let Ok((player_transform, player_velocity, player)) = player_query.get_single() {
        // Mise à jour de la position et vitesse du joueur
        camera_context.player_position = player_transform.translation;
        camera_context.player_velocity = player_velocity.linvel;
        camera_context.is_jumping = !player.is_grounded;

        // Calculer la distance au danger le plus proche
        let mut min_danger_distance = f32::MAX;
        for hazard_transform in hazard_query.iter() {
            let distance = player_transform.translation.distance(hazard_transform.translation);
            if distance < min_danger_distance {
                min_danger_distance = distance;
            }
        }
        camera_context.distance_to_danger = min_danger_distance;

        // Compter les orbes visibles (simplifié - dans portée de vision)
        let mut visible_orbs = 0;
        for orb_transform in orb_query.iter() {
            let distance = player_transform.translation.distance(orb_transform.translation);
            if distance < 10.0 { // Portée de vision de 10 unités
                visible_orbs += 1;
            }
        }
        camera_context.orbs_in_sight = visible_orbs;

        // Données du countdown de survie
        camera_context.survival_timer = shift_manager.survival_countdown.remaining_time.as_secs_f32();
        camera_context.max_survival_timer = shift_manager.survival_countdown.initial_time.as_secs_f32();

        // État de mutation (sera mis à jour par le système de mutations)
        // camera_context.is_in_mutation = ...;
    }
}

// ============================================================================
// SYSTÈMES DE RESPAWN ET RÉCUPÉRATION
// ============================================================================

pub fn player_respawn_system(
    mut player_death_events: EventReader<PlayerDeathEvent>,
    mut player_query: Query<(&mut Transform, &mut Player), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for death_event in player_death_events.read() {
        info!("💀 Mort du joueur: {:?}", death_event.cause);

        match death_event.cause {
            DeathCause::CountdownExpired => {
                // Game Over immédiat
                next_state.set(GameState::GameOver);
            },
            DeathCause::Hazard(_) | DeathCause::FallOffArena => {
                // Respawn rapide avec perte de vie
                if let Ok((mut transform, mut player)) = player_query.get_single_mut() {
                    if player.health > 0.0 {
                        // Respawn au centre de l'arène
                        transform.translation = Vec3::new(6.0, 2.0, 6.0);
                        player.health = (player.health + 20.0).min(player.max_health);
                        info!("🚑 Respawn du joueur - Santé: {}", player.health);
                    } else {
                        next_state.set(GameState::GameOver);
                    }
                }
            },
        }
    }
}

pub fn player_health_regeneration_system(
    time: Res<Time>,
    mut player_query: Query<&mut Player, With<Player>>,
) {
    if let Ok(mut player) = player_query.get_single_mut() {
        let dt = time.delta_seconds();

        // Régénération lente de santé si pas au maximum
        if player.health < player.max_health && player.health > 0.0 {
            player.health += 5.0 * dt; // 5 HP par seconde
            player.health = player.health.min(player.max_health);
        }
    }
}
