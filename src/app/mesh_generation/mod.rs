use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy_rapier3d::prelude::*;
use crate::app::core::*;
use crate::app::game::{ArenaModule, EnergyOrb, DynamicHazard, HazardType, FragileSurface, ArenaManager};
use log::info;

pub struct MeshGenerationPlugin;

impl Plugin for MeshGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleMaterials>();
    }
}

#[derive(Resource)]
pub struct ModuleMaterials {
    pub floor_std: Handle<StandardMaterial>,
    pub floor_large: Handle<StandardMaterial>,
    pub floor_fragile: Handle<StandardMaterial>,
    pub wall_low: Handle<StandardMaterial>,
    pub wall_high: Handle<StandardMaterial>,
    pub panel_glass: Handle<StandardMaterial>,
    pub ramp: Handle<StandardMaterial>,
    pub orb_energy: Handle<StandardMaterial>,
    pub hazard_lava: Handle<StandardMaterial>,
    pub hazard_laser: Handle<StandardMaterial>,
    pub decor_metallic: Handle<StandardMaterial>,
}

impl FromWorld for ModuleMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();

        Self {
            // Couleurs pastel désaturées pour les structures de base
            floor_std: materials.add(StandardMaterial {
                base_color: Color::srgb(0.63, 0.73, 0.85), // #A0BBD8 - gris-bleu pâle
                roughness: 0.8,
                metallic: 0.1,
                ..default()
            }),
            floor_large: materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.79, 0.63), // #D8C9A0 - sable doux
                roughness: 0.7,
                metallic: 0.1,
                ..default()
            }),
            floor_fragile: materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.71, 0.42), // #D8B56A - moutarde claire
                roughness: 0.9,
                metallic: 0.0,
                emissive: Color::srgb(0.1, 0.05, 0.0).into(),
                ..default()
            }),
            wall_low: materials.add(StandardMaterial {
                base_color: Color::srgb(0.55, 0.60, 0.70),
                roughness: 0.9,
                metallic: 0.2,
                ..default()
            }),
            wall_high: materials.add(StandardMaterial {
                base_color: Color::srgb(0.50, 0.55, 0.65),
                roughness: 0.8,
                metallic: 0.3,
                ..default()
            }),
            panel_glass: materials.add(StandardMaterial {
                base_color: Color::srgba(0.7, 0.8, 0.9, 0.3),
                alpha_mode: AlphaMode::Blend,
                roughness: 0.1,
                metallic: 0.0,
                ..default()
            }),
            ramp: materials.add(StandardMaterial {
                base_color: Color::srgb(0.60, 0.65, 0.75),
                roughness: 0.7,
                metallic: 0.15,
                ..default()
            }),
            // Couleurs vives et saturées pour les éléments interactifs
            orb_energy: materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.9, 1.0), // #00E5FF - cyan électrique
                emissive: Color::srgb(0.0, 0.2, 0.3).into(),
                roughness: 0.2,
                metallic: 0.0,
                ..default()
            }),
            hazard_lava: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.0, 0.67), // #FF00AA - magenta fluo
                emissive: Color::srgb(0.3, 0.0, 0.2).into(),
                roughness: 0.8,
                metallic: 0.0,
                ..default()
            }),
            hazard_laser: materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.1, 0.1),
                emissive: Color::srgb(0.5, 0.0, 0.0).into(),
                roughness: 0.3,
                metallic: 0.7,
                ..default()
            }),
            decor_metallic: materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.9),
                roughness: 0.2,
                metallic: 0.9,
                ..default()
            }),
        }
    }
}

pub fn spawn_arena_visuals(
    mut commands: Commands,
    mut arena_manager: ResMut<ArenaManager>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<ModuleMaterials>,
) {
    if let Some(ref arena) = arena_manager.current_arena {
        info!("🏗️ Génération visuelle de l'arène avec {} modules", arena.modules.len());
        
        let mut spawned_entities = Vec::new();
        
        for module in &arena.modules {
            if let Some(entity) = spawn_module_visual(
                &mut commands,
                module,
                &mut meshes,
                &materials,
            ) {
                spawned_entities.push(entity);
            }
        }
        
        arena_manager.spawned_modules = spawned_entities;
        info!("✅ {} entités visuelles créées", arena_manager.spawned_modules.len());
    }
}

fn spawn_module_visual(
    commands: &mut Commands,
    cell: &ArenaCell,
    meshes: &mut Assets<Mesh>,
    materials: &ModuleMaterials,
) -> Option<Entity> {
    let position = Vec3::new(cell.x as f32, 0.0, cell.y as f32);
    
    let (mesh, material, scale, collider, additional_components) = match cell.module_id {
        ModuleId::FloorStd => (
            create_cube_mesh(meshes, 1.0, 0.1, 1.0),
            materials.floor_std.clone(),
            Vec3::new(1.0, 0.1, 1.0),
            Collider::cuboid(0.5, 0.05, 0.5),
            vec![]
        ),
        
        ModuleId::FloorLarge => (
            create_cube_mesh(meshes, 2.0, 0.1, 2.0),
            materials.floor_large.clone(),
            Vec3::new(2.0, 0.1, 2.0),
            Collider::cuboid(1.0, 0.05, 1.0),
            vec![]
        ),
        
        ModuleId::FloorFragile => {
            let params = cell.module_params.as_ref()
                .and_then(|p| p.as_object())
                .map(|obj| (
                    obj.get("breakDelay").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                    obj.get("respawnDelay").and_then(|v| v.as_f64()).unwrap_or(5.0) as f32,
                )).unwrap_or((0.5, 5.0));
            
            (
                create_cube_mesh(meshes, 1.0, 0.1, 1.0),
                materials.floor_fragile.clone(),
                Vec3::new(1.0, 0.1, 1.0),
                Collider::cuboid(0.5, 0.05, 0.5),
                vec![Box::new(FragileSurface {
                    break_delay: params.0,
                    respawn_delay: params.1,
                    is_breaking: false,
                    break_timer: 0.0,
                    is_broken: false,
                    respawn_timer: 0.0,
                }) as Box<dyn Component>]
            )
        },
        
        ModuleId::WallLow => (
            create_cube_mesh(meshes, 1.0, 1.0, 1.0),
            materials.wall_low.clone(),
            Vec3::new(1.0, 1.0, 1.0),
            Collider::cuboid(0.5, 0.5, 0.5),
            vec![]
        ),
        
        ModuleId::WallHigh => (
            create_cube_mesh(meshes, 1.0, 2.5, 1.0),
            materials.wall_high.clone(),
            Vec3::new(1.0, 2.5, 1.0),
            Collider::cuboid(0.5, 1.25, 0.5),
            vec![]
        ),
        
        ModuleId::PanelGlass => (
            create_cube_mesh(meshes, 1.0, 2.0, 0.1),
            materials.panel_glass.clone(),
            Vec3::new(1.0, 2.0, 0.1),
            Collider::cuboid(0.5, 1.0, 0.05),
            vec![]
        ),
        
        ModuleId::RampLow | ModuleId::RampSteep => {
            let angle = cell.module_params.as_ref()
                .and_then(|p| p.get("angle"))
                .and_then(|v| v.as_f64())
                .unwrap_or(30.0) as f32;
                
            (
                create_ramp_mesh(meshes, 1.0, angle.to_radians()),
                materials.ramp.clone(),
                Vec3::new(1.0, 0.5, 1.0),
                Collider::cuboid(0.5, 0.25, 0.5), // Simplifié pour le moment
                vec![]
            )
        },
        
        ModuleId::OrbEnergy => {
            let time_value = cell.module_params.as_ref()
                .and_then(|p| p.get("timeValue"))
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;
                
            (
                create_sphere_mesh(meshes, 0.3, 16),
                materials.orb_energy.clone(),
                Vec3::new(0.3, 0.3, 0.3),
                Collider::ball(0.3),
                vec![Box::new(EnergyOrb {
                    time_value,
                    collected: false,
                }) as Box<dyn Component>]
            )
        },
        
        ModuleId::HazardLavaPit => (
            create_cube_mesh(meshes, 1.0, 0.05, 1.0),
            materials.hazard_lava.clone(),
            Vec3::new(1.0, 0.05, 1.0),
            Collider::cuboid(0.5, 0.025, 0.5),
            vec![Box::new(DynamicHazard {
                hazard_type: HazardType::LavaPit,
                intensity: 1.0,
                lifetime: 0.0,
                max_lifetime: f32::MAX,
            }) as Box<dyn Component>]
        ),
        
        ModuleId::HazardLaserEmitterStatic => (
            create_cube_mesh(meshes, 0.5, 1.5, 0.5),
            materials.hazard_laser.clone(),
            Vec3::new(0.5, 1.5, 0.5),
            Collider::cuboid(0.25, 0.75, 0.25),
            vec![Box::new(DynamicHazard {
                hazard_type: HazardType::LaserBeam,
                intensity: 1.0,
                lifetime: 0.0,
                max_lifetime: f32::MAX,
            }) as Box<dyn Component>]
        ),
        
        ModuleId::HazardLaserTurretRotate => (
            create_cylinder_mesh(meshes, 0.4, 1.0, 12),
            materials.hazard_laser.clone(),
            Vec3::new(0.4, 1.0, 0.4),
            Collider::cylinder(0.5, 0.4),
            vec![Box::new(DynamicHazard {
                hazard_type: HazardType::RotatingTurret,
                intensity: 1.0,
                lifetime: 0.0,
                max_lifetime: f32::MAX,
            }) as Box<dyn Component>]
        ),
        
        ModuleId::DecorArchMetallic => {
            let color_variant = cell.module_params.as_ref()
                .and_then(|p| p.get("colorVariant"))
                .and_then(|v| v.as_u64())
                .unwrap_or(3);
                
            (
                create_arch_mesh(meshes, 2.0, 3.0),
                materials.decor_metallic.clone(),
                Vec3::new(2.0, 3.0, 0.3),
                Collider::cuboid(1.0, 1.5, 0.15),
                vec![]
            )
        },
        
        _ => return None, // Module non supporté pour l'instant
    };

    let final_position = position + Vec3::new(0.0, scale.y * 0.5, 0.0);
    
    let mut entity_commands = commands.spawn((
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_translation(final_position)
                .with_scale(scale),
            ..default()
        },
        RigidBody::Fixed,
        collider,
        ArenaModule {
            module_id: cell.module_id.clone(),
            original_position: final_position,
        },
    ));

    // Ajouter les composants supplémentaires
    for component in additional_components {
        // Note: En Rust, nous ne pouvons pas facilement ajouter des composants dynamiquement
        // Cette partie nécessiterait une approche différente avec des enums ou traits
        // Pour l'instant, on va gérer cela manuellement dans les cas spécifiques
    }

    Some(entity_commands.id())
}

// ============================================================================
// FONCTIONS DE CRÉATION DE MAILLAGES
// ============================================================================

fn create_cube_mesh(meshes: &mut Assets<Mesh>, width: f32, height: f32, depth: f32) -> Handle<Mesh> {
    let w = width * 0.5;
    let h = height * 0.5;
    let d = depth * 0.5;

    let vertices = vec![
        // Face avant (z+)
        [-w, -h,  d], [ w, -h,  d], [ w,  h,  d], [-w,  h,  d],
        // Face arrière (z-)
        [ w, -h, -d], [-w, -h, -d], [-w,  h, -d], [ w,  h, -d],
        // Face droite (x+)
        [ w, -h,  d], [ w, -h, -d], [ w,  h, -d], [ w,  h,  d],
        // Face gauche (x-)
        [-w, -h, -d], [-w, -h,  d], [-w,  h,  d], [-w,  h, -d],
        // Face haut (y+)
        [-w,  h,  d], [ w,  h,  d], [ w,  h, -d], [-w,  h, -d],
        // Face bas (y-)
        [-w, -h, -d], [ w, -h, -d], [ w, -h,  d], [-w, -h,  d],
    ];

    let indices = vec![
        0,1,2, 0,2,3,       // avant
        4,5,6, 4,6,7,       // arrière
        8,9,10, 8,10,11,    // droite
        12,13,14, 12,14,15, // gauche
        16,17,18, 16,18,19, // haut
        20,21,22, 20,22,23, // bas
    ];

    let normals = vec![
        // Face avant
        [0., 0., 1.], [0., 0., 1.], [0., 0., 1.], [0., 0., 1.],
        // Face arrière
        [0., 0., -1.], [0., 0., -1.], [0., 0., -1.], [0., 0., -1.],
        // Face droite
        [1., 0., 0.], [1., 0., 0.], [1., 0., 0.], [1., 0., 0.],
        // Face gauche
        [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.], [-1., 0., 0.],
        // Face haut
        [0., 1., 0.], [0., 1., 0.], [0., 1., 0.], [0., 1., 0.],
        // Face bas
        [0., -1., 0.], [0., -1., 0.], [0., -1., 0.], [0., -1., 0.],
    ];

    let uvs = vec![
        // Répéter les UVs pour chaque face
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh)
}

fn create_sphere_mesh(meshes: &mut Assets<Mesh>, radius: f32, resolution: usize) -> Handle<Mesh> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // Génération des vertices de la sphère
    for i in 0..=resolution {
        let theta = i as f32 * std::f32::consts::PI / resolution as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for j in 0..=resolution {
            let phi = j as f32 * 2.0 * std::f32::consts::PI / resolution as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;

            vertices.push([x, y, z]);
            normals.push([x / radius, y / radius, z / radius]);
            uvs.push([j as f32 / resolution as f32, i as f32 / resolution as f32]);
        }
    }

    // Génération des indices
    for i in 0..resolution {
        for j in 0..resolution {
            let first = i * (resolution + 1) + j;
            let second = first + resolution + 1;

            indices.extend_from_slice(&[
                first as u32, second as u32, (first + 1) as u32,
                second as u32, (second + 1) as u32, (first + 1) as u32,
            ]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh)
}

fn create_cylinder_mesh(meshes: &mut Assets<Mesh>, radius: f32, height: f32, resolution: usize) -> Handle<Mesh> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let half_height = height * 0.5;

    // Vertices du haut et du bas
    for i in 0..resolution {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / resolution as f32;
        let x = radius * angle.cos();
        let z = radius * angle.sin();

        // Vertex du haut
        vertices.push([x, half_height, z]);
        normals.push([x / radius, 0.0, z / radius]);
        uvs.push([angle / (2.0 * std::f32::consts::PI), 1.0]);

        // Vertex du bas
        vertices.push([x, -half_height, z]);
        normals.push([x / radius, 0.0, z / radius]);
        uvs.push([angle / (2.0 * std::f32::consts::PI), 0.0]);
    }

    // Indices pour les faces latérales
    for i in 0..resolution {
        let next = (i + 1) % resolution;
        
        let top_current = i * 2;
        let bottom_current = i * 2 + 1;
        let top_next = next * 2;
        let bottom_next = next * 2 + 1;

        // Triangles de la face latérale
        indices.extend_from_slice(&[
            top_current as u32, bottom_current as u32, top_next as u32,
            bottom_current as u32, bottom_next as u32, top_next as u32,
        ]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh)
}

fn create_ramp_mesh(meshes: &mut Assets<Mesh>, size: f32, angle: f32) -> Handle<Mesh> {
    let half_size = size * 0.5;
    let height = (size * angle.tan()).max(0.1);

    let vertices = vec![
        // Face inclinée (dessus)
        [-half_size, 0.0, -half_size],
        [half_size, 0.0, -half_size],
        [half_size, height, half_size],
        [-half_size, height, half_size],
        // Face avant (verticale)
        [-half_size, 0.0, half_size],
        [half_size, 0.0, half_size],
        [half_size, height, half_size],
        [-half_size, height, half_size],
        // Autres faces...
    ];

    let indices = vec![
        // Face inclinée
        0, 1, 2, 0, 2, 3,
        // Face avant
        4, 6, 5, 4, 7, 6,
        // (autres faces simplifiées)
    ];

    let normals = vec![
        // Normales calculées pour la face inclinée
        [0.0, angle.cos(), -angle.sin()]; 4,
        // Face avant
        [0.0, 0.0, 1.0]; 4,
    ];

    let uvs = vec![
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
        [0., 0.], [1., 0.], [1., 1.], [0., 1.],
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh)
}

fn create_arch_mesh(meshes: &mut Assets<Mesh>, width: f32, height: f32) -> Handle<Mesh> {
    // Maillage simplifié d'une arche (deux piliers + arc)
    let pillar_width = width * 0.1;
    let arch_thickness = width * 0.05;
    
    // Pour l'instant, on crée un maillage rectangulaire simple
    // Dans une version plus avancée, on pourrait créer une vraie forme d'arche
    create_cube_mesh(meshes, width, height, arch_thickness)
}

// ============================================================================
// SYSTÈMES D'ANIMATION ET D'EFFETS
// ============================================================================

pub fn animate_orbs_system(
    time: Res<Time>,
    mut orb_query: Query<(&mut Transform, &EnergyOrb), (With<EnergyOrb>, Without<Player>)>,
) {
    let time_secs = time.elapsed_seconds();
    
    for (mut transform, orb) in orb_query.iter_mut() {
        if !orb.collected {
            // Rotation et flottement
            transform.rotation = Quat::from_rotation_y(time_secs * 2.0);
            transform.translation.y = orb.time_value * 0.5 + (time_secs * 3.0).sin() * 0.2;
        }
    }
}

pub fn hazard_effects_system(
    time: Res<Time>,
    mut hazard_query: Query<(&mut Transform, &DynamicHazard), With<DynamicHazard>>,
) {
    let time_secs = time.elapsed_seconds();
    
    for (mut transform, hazard) in hazard_query.iter_mut() {
        match hazard.hazard_type {
            HazardType::LavaPit => {
                // Effet de pulsation pour la lave
                let pulse = (time_secs * 4.0).sin() * 0.1 + 1.0;
                transform.scale.y = pulse;
            },
            HazardType::RotatingTurret => {
                // Rotation constante
                transform.rotation = Quat::from_rotation_y(time_secs * 1.5);
            },
            _ => {}
        }
    }
}

pub fn fragile_surface_system(
    time: Res<Time>,
    mut commands: Commands,
    mut fragile_query: Query<(Entity, &mut FragileSurface, &mut Visibility), With<FragileSurface>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_seconds();
    
    if let Ok(player_transform) = player_query.get_single() {
        for (entity, mut fragile, mut visibility) in fragile_query.iter_mut() {
            if fragile.is_broken {
                fragile.respawn_timer += dt;
                if fragile.respawn_timer >= fragile.respawn_delay {
                    // Réapparaître
                    fragile.is_broken = false;
                    fragile.respawn_timer = 0.0;
                    fragile.is_breaking = false;
                    fragile.break_timer = 0.0;
                    *visibility = Visibility::Visible;
                }
            } else if fragile.is_breaking {
                fragile.break_timer += dt;
                if fragile.break_timer >= fragile.break_delay {
                    // Casser
                    fragile.is_broken = true;
                    fragile.respawn_timer = 0.0;
                    *visibility = Visibility::Hidden;
                    
                    // Retirer le collider
                    commands.entity(entity).remove::<Collider>();
                }
            }
            // Note: La détection de collision avec le joueur serait gérée
            // par un système de détection de collision séparé
        }
    }
}
