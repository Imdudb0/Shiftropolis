# SME Arena Generator - Système de Test

Un système de génération et de test d'arènes pour votre jeu de survie 3D avec la Machine à Mutations (SME).
                                                                ## 🚀 Installation

### Prérequis
- **Rust** (dernière version stable) : https://rustup.rs/       - **Ubuntu/Linux** (testé sur Ubuntu 20.04+)

### Installation de Rust (si pas déjà fait)                     ```bash                                                         curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  source ~/.cargo/env                                             rustc --version  # Vérifier l'installation
```

### Cloner et compiler le projet
```bash
# Créer le projet
cargo new sme-arena-generator --bin                             cd sme-arena-generator

# Remplacer le Cargo.toml et ajouter les fichiers source        # (coller le contenu des artifacts dans les fichiers appropriés)
# Compiler le projet                                            cargo build --release

# Lancer les tests                                              cargo test
```                                                             
## 📁 Structure du Projet                                                                                                       ```
sme-arena-generator/                                            ├── src/
│   ├── main.rs              # Application CLI principale       │   ├── core/
│   │   ├── mod.rs           # Types et structures de base      │   │   ├── types.rs         # Implémentations des types
│   │   └── arena.rs         # Validations avancées d'arène
│   ├── generation/                                             │   │   └── mod.rs           # Algorithmes de génération
│   ├── monitoring/                                             │   │   └── mod.rs           # Système de surveillance d'anomalies
│   └── data/
│       └── mod.rs           # Base de données des règles et modules                                                            ├── Cargo.toml               # Configuration du projet
└── README.md                # Ce fichier                       ```

## 🎮 Utilisation                                               
### Commandes Principales                                       
#### 1. Génération d'une arène simple
```bash                                                         # Arène basique 10x10 avec 3 règles
cargo run -- generate

# Arène personnalisée
cargo run -- generate --size 15 --rules 4 --verbose

# Arène reproductible avec seed
cargo run -- generate --size 12 --rules 2 --seed 12345 --verbose

# gameplay
cargo run gameplay --duration 120 --size 15 --difficulty 1.5 --countdown 25
```                                                             
**Exemple de sortie :**
```                                                             🚀 SME Arena Generator Test Suite
                                        
Generating 10x10 arena with 3 rules

📊 Generation Results:
  ⏱️  Time: 12.3ms                                                 📐 Size: 10x10
  🎯 Active Rules: 3                                              🔧 Modules: 67
  💎 Energy Orbs: 8                                             
🌍 Environmental Variables:
  GRAVITY = 0.85
  GAME_SPEED = 1.20

📋 Active Rules:
  • LOW_JUMP - The jump height is reduced.
  • LAVA_FLOOR - Dangerous lava pits appear throughout the arena.
  • ORB_COLLECTION - More energy orbs spawn in the arena.
                                                                ✅ No anomalies detected!                                       ```                                                             
#### 2. Test de stress
```bash                                                         # Test standard : 100 arènes
cargo run -- stress                                                                                                             # Test intensif : 500 arènes avec paramètres variables
cargo run -- stress --count 500 --size-range 8,20 --rules-range 1,6

# Test avec arrêt à la première anomalie                        cargo run -- stress --count 100 --fail-fast
```
                                                                **Exemple de sortie :**
```                                                             Running stress test: 100 arenas                                 🔄 Progress: 100% [100/100]

📊 Stress Test Results:                                           ✅ Successful: 97/100 (97.0%)
  ❌ Failed: 3
  ⏱️  Total Time: 1.234s                                           📈 Avg Time: 12.34ms
                                                                ❌ Failed Arenas:
  Arena 23: Arena generation failed due to critical anomalies
  Arena 67: Invalid configuration: incompatible rules detected
  Arena 89: Constraint violation: minimum walkable area         
⚠️  12 anomalies detected:
  🟡 Warning: 8
  🔵 Info: 4                                                    ```
                                                                #### 3. Benchmark de performance
```bash
# Benchmark de 30 secondes                                      cargo run -- benchmark

# Benchmark plus long                                           cargo run -- benchmark --duration 60
```                                                             
**Exemple de sortie :**
```
Running benchmark for 30 seconds
🚀 Generated: 2847 arenas (94.9/s)
                                                                🏁 Benchmark Results:
  🎯 Total Arenas: 2847
  ⏱️  Total Time: 30.001s
  📈 Rate: 94.90 arenas/sec
  🧩 Avg Modules: 52.3
  💾 Peak Memory: 45.2 MB                                       ```
                                                                ### Options Avancées                                                                                                            #### Debugging et Analyse
```bash                                                         # Mode verbose avec visualisation ASCII
cargo run -- generate --size 8 --rules 2 --verbose

# Génération avec seed spécifique pour reproduction de bugs
cargo run -- generate --seed 666 --verbose
```

#### Variables d'Environnement
```bash
# Activer les logs détaillés
RUST_LOG=debug cargo run -- generate --verbose
                                                                # Logs encore plus détaillés
RUST_LOG=trace cargo run -- stress --count 10
```

## 🔍 Système de Surveillance d'Anomalies

Le système surveille automatiquement :

### 🔴 Anomalies Critiques
- **Structural** : Spawn du joueur manquant, zones inaccessibles
- **Bounds** : Modules en dehors des limites de l'arène
- **Reachability** : Orbes d'énergie inaccessibles depuis le spawn

### 🟡 Anomalies d'Avertissement
- **Balance** : Densité excessive de dangers, ratio surface/danger déséquilibré
- **Rules** : Règles incompatibles actives, incohérences règle-environnement
- **Spatial** : Clustering excessif de dangers, éléments isolés

### 🔵 Anomalies Informatives
- **Performance** : Temps de génération élevé
- **Distribution** : Répartition sous-optimale des éléments     
### Métriques Surveillées
- Temps de génération
- Ratio surface walkable/totale
- Densité des dangers
- Densité des orbes d'énergie                                   - Connectivité des zones importantes
- Utilisation mémoire                                           
## 🧪 Tests et Validation                                       
### Tests Unitaires                                             ```bash
cargo test
```

### Tests d'Intégration
```bash
cargo test --test integration
```

### Benchmark de Performance                                    ```bash
cargo bench                                                     ```

## 🛠️ Configuration
                                                                ### Paramètres des Règles
Les règles sont définies dans `src/data/mod.rs` avec :
- **ID unique** et nom
- **Tags** pour la catégorisation                               - **Paramètres** optionnels (JSON)
- **Incompatibilités** avec d'autres règles

### Modules d'Arène                                             Chaque module a :                                               - **Type** (sol, danger, interactif, etc.)
- **Poids** pour l'algorithme WFC
- **Tags** pour les contraintes                                 - **Paramètres** spécifiques

### Variables d'Environnement
- **Gravité** : 0.2 - 3.0 (défaut: 1.0)
- **Vitesse du jeu** : 0.5 - 2.0 (défaut: 1.0)                  
## 🚨 Résolution de Problèmes
                                                                ### Erreurs Courantes

#### "Arena generation failed due to critical anomalies"
```bash
# Vérifier avec plus de détails
cargo run -- generate --verbose
```

#### Mémoire insuffisante
```bash
# Réduire la taille de l'arène
cargo run -- generate --size 8 --rules 2                        ```

#### Performance lente
```bash
# Vérifier en mode release
cargo build --release
cargo run --release -- benchmark --duration 10
```                                                             
### Logs de Debug
```bash
# Activer tous les logs                                         RUST_LOG=sme_arena_generator=trace cargo run -- generate --verbose                                                              ```

## 📈 Métriques de Performance Attendues

Sur un système moderne (Ubuntu, CPU récent) :
- **Génération simple** : < 50ms
- **Arène 15x15** : < 100ms
- **Rate de génération** : > 50 arènes/sec
- **Mémoire** : < 100MB pour 1000 arènes
                                                                ## 🔧 Développement                                             
### Ajouter une Nouvelle Règle
1. Ajouter l'ID dans `RuleId` enum
2. Implémenter dans `RulesDatabase::initialize()`
3. Ajouter la logique dans `ArenaGenerator::add_rule_constraints()`                   4. Tester avec `cargo test`

### Ajouter un Nouveau Module
1. Ajouter l'ID dans `ModuleId` enum
2. Définir dans `ModulesDatabase::initialize()`             3. Implémenter la logique de placement
4. Ajouter les vérifications d'anomalies
                
### Contributeurs
Voir les anomalies détectées comme des opportunités d'amélioration du système plutôt que des erreurs !



### camera
Le vecteur de contexte CameraContext inclut :
  
Position et vélocité du joueurDistance aux dangers
État du timer de survie
Statut de mutation
Proximité des plateformes
Nombre d'orbes visibles
État de saut
Facteur de gravité
Zones dangereuses

#Utilisation

// Dans votre main.rs
app.add_plugins(ContextualCameraPlugin);

// Mise à jour du contexte (dans vos systèmes de gameplay)
fn update_camera_context(mut context: ResMut<CameraContext>, /* autres queries */) {
    context.player_position = player_transform.translation;
    context.distance_to_danger = calculate_nearest_danger_distance();
    // ... etc
}

// Changement manuel de mode (optionnel)
mode_events.send(CameraModeChangeEvent {
    new_mode: CameraMode::Danger,
    transition_speed: Some(3.0),
});





Fonctionnalités implémentées



Gameplay complet :✅


Génération d'arène avec votre système WFC existant✅ 

Système SME (mutations, shifts, difficulté progressive)✅

Joueur 3D avec physique Rapier✅ 

Collecte d'orbes avec système de survie✅ 
Caméra contextuelle qui s'adapte au gameplay✅ 

Interface responsive avec countdown, barres de vie, notifications



Contrôles mobiles :✅ 


Joystick virtuel (moitié gauche d'écran)✅ 

Bouton de saut (moitié droite)✅ 

Retour haptique Android✅ 

Support clavier pour les tests

Génération dynamique :✅ 

Maillages procéduraux pour tous les modules (cubes, sphères, rampes, arches)✅ 

Matériaux avec votre palette (pastels désaturés + couleurs vives)✅ 

Colliders automatiques pour la physique✅ 

Système de tags flexible pour WFC







#### IDEAS ####

des le debut, un grand nombre de defis calcule pour chaque level (1 - 2 - 3 - 4 - 5).
dpendemment du level, on lui propose un defi, si les regles imposes par le SME ne lui permettra pas de passer ce defis, le joueur est oblige de
payer avec sa monnaie virtuelle pour obtenir une mutation specifique.
