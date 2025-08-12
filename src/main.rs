// src/main.rs...

use bevy::prelude::*;
use log::info;

// Importe les plugins pour la physique avec Rapier3D.
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::render::RapierDebugRenderPlugin;

// Configuration spécifique pour la compilation sur Android.
#[cfg(target_os = "android")]
use bevy::winit::WinitSettings;

// Importe les plugins de VOTRE JEU depuis la librairie 'shiftropolis'.
// C'est la seule façon correcte d'accéder à votre code de jeu.
use shiftropolis::app::{
    camera::ContextualCameraPlugin,
    game::GamePlugin,
    ui::UIPlugin,
};


// --- FONCTION PRINCIPALE ---

fn main() {
    // Initialise le logger pour Android.
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info)
    );

    let mut app = App::new();

    // Ajoute les plugins par défaut de Bevy, en configurant la fenêtre.
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Shiftropolis".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));

    // Ajoute la configuration spécifique à Android.
    #[cfg(target_os = "android")]
    {
        app.insert_resource(WinitSettings::mobile_defaults());
    }
    
    // --- PLUGINS DU JEU ---

    // Ajoute les plugins pour la physique.
    app.add_plugins((
        RapierPhysicsPlugin::<NoUserData>::default(),
        // Le `RapierDebugRenderPlugin` dessine les "hitbox" des objets.
        // Utile pour le débogage, vous pouvez le désactiver pour la version finale.
        RapierDebugRenderPlugin::default(),
    ));

    // Ajoute les plugins de votre jeu.
    app.add_plugins((
        GamePlugin,
        UIPlugin,
        ContextualCameraPlugin,
    ));

    // Ajoute le système qui s'exécute au démarrage.
    app.add_systems(Startup, setup_scene);

    // Lance l'application !
    app.run();
}

fn setup_scene(mut commands: Commands) {
    info!("🚀 Préparation de la scène...");

    // Lumière d'ambiance pour éclairer globalement la scène.
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    // Lumière directionnelle, simulant le soleil.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 15.0, 10.0)
                   .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
