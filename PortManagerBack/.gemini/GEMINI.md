# Directives Backend (Rust) - PortManager

Ce document définit les standards et les meilleures pratiques pour le développement de l'agent backend Rust.

## Architecture & Design
- **API REST :** Utiliser un framework moderne et performant (ex: `axum` ou `actix-web`).
- **Modularité :** Séparer la logique métier en services (Docker, Nginx, Firewall).
- **Modèles de Données :** Utiliser `serde` pour la sérialisation/désérialisation JSON.
- **Structure du Code :** Organiser le code en modules clairs et cohérents.
  - Le controller ne possède pas de logique métier, elle délègue à des services dédiés.
  - Les services sont responsables de l'interaction avec les systèmes externes (Docker, Nginx, Firewall).
  - Si nécessaire, on peut introduire une couche de repository pour l'accès aux données.

## Meilleures Pratiques Rust
- **Sécurité Mémoire :** Exploiter le système de types et le borrow checker pour garantir la sécurité.
- **Gestion d'Erreurs :** Utiliser `Result` et `anyhow` ou `thiserror` pour une gestion d'erreurs robuste et explicite.
- **Asynchronisme :** Utiliser `tokio` pour les opérations I/O non bloquantes.
- **Tests :** Écrire des tests unitaires et d'intégration pour assurer la fiabilité du code.
- **Typage :** Les types doivent toujours être explicites.
- 
## Sécurité & Système (CRITIQUE)
- **Injection de Commandes :** NE JAMAIS concaténer des chaînes de caractères pour construire des commandes shell. Utiliser `std::process::Command` avec des arguments séparés.
- **Privilèges :** L'agent ne doit pas tourner en `root`. Utiliser des capacités Linux ou des permissions sudo spécifiques.
- **Docker :** Préférer l'utilisation de crates comme `bollard` pour interagir avec l'API Docker via le socket.
- **Nginx :**
  - Écrire les configurations dans des fichiers temporaires.
  - Toujours valider avec `nginx -t` avant de remplacer le fichier actif.
  - Recharger via `systemctl reload nginx`.

## Conventions de Code
- Suivre les recommandations de `clippy`.
- Documenter les fonctions publiques avec des Doc-comments (`///`).
- Utiliser `cargo fmt` pour le formatage.

## Structure du Projet
- `src/api/` : Handlers et routes REST.
- `src/models/` : Structures de données (DTOs).
- `src/services/` : Logique d'interaction avec Docker, Nginx et le système.
