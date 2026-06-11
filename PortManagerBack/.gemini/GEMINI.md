# Directives Backend (Rust) - PortManager

Ce document définit les standards et les meilleures pratiques pour le développement de l'agent backend Rust.

## Architecture & Design
- **API REST :** Utiliser un framework moderne et performant (ex: `axum` ou `actix-web`).
- **Modularité :** Séparer la logique métier en services (Docker, Nginx, Firewall).
- **Modèles de Données :** Utiliser `serde` pour la sérialisation/désérialisation JSON.

## Meilleures Pratiques Rust
- **Sécurité Mémoire :** Exploiter le système de types et le borrow checker pour garantir la sécurité.
- **Gestion d'Erreurs :** Utiliser `Result` et `anyhow` ou `thiserror` pour une gestion d'erreurs robuste et explicite.
- **Asynchronisme :** Utiliser `tokio` pour les opérations I/O non bloquantes.

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
