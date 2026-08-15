[PRD]

# Backend libghostty natif pour Paneflow sous macOS

**Status:** READY  
**Version:** 1.0  
**Author:** Arthur Jean  
**Created:** 2026-08-15  
**Target:** 2026-Q3  
**Scope:** macOS 13 Ventura+ sur Apple Silicon, `aarch64-apple-darwin`  
**Related:** `tasks/prd-linux-libghostty-backend-2026-Q3.md`, `tasks/prd-linux-libghostty-promotion-2026-Q3.md`, `tasks/prd-windows-libghostty-backend-2026-Q3.md`

## Changelog

| Version | Date | Status | Changes |
|---|---|---|---|
| 1.0 | 2026-08-15 | READY | PRD initial fondé sur les intégrations Linux et Windows livrées et sur l'audit des frontières macOS encore absentes. |

## Problem Statement

Paneflow exécute déjà libghostty-vt en production sur Linux et sur Windows x64 MSVC. macOS est aujourd'hui la seule des trois plateformes supportées à rester sur `alacritty_terminal`. Le résultat est un produit à deux moteurs VT selon la machine de l'utilisateur : divergences de comportement possibles sur Unicode, graphemes, protocoles clavier, scrollback et OSC, corpus différentiel non exécuté sur un tiers du parc, et double coût de maintenance sur chaque évolution terminal.

Le blocage n'est pas une limite de libghostty-vt. La révision Ghostty épinglée par Paneflow est celle du terminal Ghostty officiel, dont macOS est la plateforme historique de référence : le moteur VT y est le mieux exercé de toutes les cibles. Le blocage se trouve dans quatre frontières Paneflow encore Linux/Windows-only :

1. La distribution d'un artefact Mach-O statique reproductible, vérifié et hashé pour `aarch64-apple-darwin`.
2. Les gates Cargo, FFI et build scripts, qui n'acceptent aujourd'hui que `linux` et `windows`, dupliqués sur environ 344 sites `cfg` répartis sur neuf fichiers.
3. Le host de session, dont les primitives POSIX sont gatées `target_os = "linux"` par accident d'historique plutôt que par nécessité technique.
4. La qualification fonctionnelle Darwin : conventions clavier macOS, IME Core Text, presse-papiers, `.app` signé et notarisé, cask Homebrew.

Une démonstration affichant `zsh` ne suffirait pas. Le résultat doit pouvoir devenir le backend macOS par défaut sans régression sur les workflows réels, sans processus orphelin, sans dépendance dylib fragile, sans rupture de signature ou de notarisation, et avec un rollback Alacritty immédiat avant le spawn d'un child.

## Overview

Ce projet étend l'intégration libghostty-vt existante à macOS Apple Silicon. Il ne porte ni l'application Ghostty officielle ni son interface. Paneflow conserve GPUI pour la fenêtre, le rendu, l'IME, les panes, la configuration et les interactions produit. Il conserve `portable-pty` comme host PTY, ce qui sélectionne le PTY BSD natif sur Darwin. Libghostty-vt reste responsable du parsing VT, de l'état terminal, des snapshots et de l'encodage des protocoles terminal.

Une story de préparation neutre plateforme ouvre le chantier. Les gates Ghostty actuels sont un `any(all(linux, feature), all(windows, arch, env, feature))` recopié à l'identique sur environ 344 sites, dans des fichiers allant jusqu'à 244 Ko. Ajouter une troisième branche à la main sur chacun d'eux serait la principale source de risque du projet : illisible en review, et une branche oubliée produit soit une erreur de compilation obscure, soit du code mort silencieux. EP-001 remplace donc ce motif par un unique alias `cfg` émis par `src-app/build.rs`, avant tout ajout macOS. Cette story est un gain indépendamment de macOS et doit être livrable et mergeable seule.

Le rollout comporte ensuite les deux mêmes états successifs que le PRD Windows :

1. Qualification : libghostty est compilé dans les builds macOS standards, mais seul un choix explicite active Ghostty. La valeur `auto` continue de sélectionner Alacritty.
2. Promotion : après passage de tous les quality gates, `auto` sélectionne Ghostty sous macOS. Alacritty reste disponible comme rollback explicite. Une session ne change jamais de backend après le spawn de son processus.

La cible de production est une archive statique `libghostty-vt.a` pour `aarch64-apple-darwin`. Une dylib n'est qu'un plan de contingence si le linkage statique est démontré impossible, et exigerait une décision séparée couvrant `@rpath`, signature, notarisation et cask.

## Goals

| ID | Goal | Target |
|---|---|---|
| G-001 | Produire et consommer un artefact libghostty-vt Darwin natif, hermétique et traçable. | Archive statique `aarch64-apple-darwin` au SHA Ghostty `ae52f97dcac558735cfa916ea3965f247e5c6e9e` et Zig 0.15.2, hashée et vérifiée en CI. |
| G-002 | Supprimer la duplication des gates de compilation avant d'ajouter une troisième plateforme. | Un alias `cfg` unique remplace le motif deux-branches sur 100 % des sites concernés, sans changement de comportement Linux ni Windows. |
| G-003 | Exécuter un terminal Ghostty complet sur le PTY Darwin sans modifier le renderer GPUI. | zsh, bash, fish et nushell passent la matrice fonctionnelle. |
| G-004 | Atteindre la parité de comportement avec le backend macOS Alacritty sur les fonctions Paneflow supportées. | Zéro divergence inexpliquée dans le corpus différentiel et 100 % des cas input, IME, clipboard, resize et lifecycle validés. |
| G-005 | Garantir un lifecycle sûr et déterministe sur Darwin. | Zéro deadlock, zéro child ou descendant orphelin et zéro double-spawn sur les campagnes de stress définies dans les quality gates. |
| G-006 | Distribuer Ghostty dans le `.app` et le cask sans dépendance native fragile. | Le binaire contient le linkage statique, aucune `libghostty-vt.dylib` n'est requise, installée ou chargée ; signature et notarisation inchangées. |
| G-007 | Promouvoir Ghostty sans rendre le rollback risqué. | `auto` sélectionne Ghostty après qualification ; `alacritty` reste sélectionnable par configuration et le fallback automatique n'arrive qu'avant tout spawn. |
| G-008 | Préserver les autres plateformes. | Les comportements et gates Linux et Windows restent verts, sans changement de leur sélection `auto`. |

## Target Users

### Utilisateur primaire : développeur Paneflow sous macOS

Il utilise un Mac Apple Silicon comme machine de développement principale, ouvre plusieurs panes et mélange zsh, fish, agents de code et outils TUI. Il attend un terminal rapide, correct sur Unicode et les graphemes, compatible avec ses raccourcis, son clavier local, l'IME et le presse-papiers. Il ne doit pas connaître l'existence du linkage natif pour utiliser l'application.

### Utilisateur secondaire : utilisateur avancé et contributeur Paneflow

Il veut choisir explicitement Ghostty ou Alacritty, diagnostiquer un échec de backend sans exposer le contenu de son terminal, reproduire un problème et revenir à Alacritty sans réinstaller Paneflow.

### Mainteneur Paneflow

Il doit pouvoir reconstruire l'artefact natif à partir du SHA épinglé, vérifier sa provenance, détecter une dérive ABI, exécuter une matrice macOS reproductible et publier un `.app` notarisé sans installer Zig ou Ghostty sur la machine de consommation standard.

## Research Findings

### Codebase Paneflow

| Finding | Evidence | Product implication |
|---|---|---|
| L'abstraction backend et le snapshot `Content` sont déjà neutres. | `src-app/src/terminal/pty_session.rs`, `src-app/src/terminal/types.rs` | Il faut étendre le backend existant, pas créer un troisième terminal ni brancher le renderer sur l'OS. |
| Le crate wrapper n'est déclaré que pour Linux et Windows. | `crates/paneflow-terminal-ghostty/Cargo.toml`, section `[target.'cfg(any(target_os = "linux", target_os = "windows"))'.dependencies]` | L'activation macOS est un ajout de cible, pas une réécriture du wrapper. |
| Le `build.rs` du crate `-sys` retourne `Ok(None)` sur toute cible non Linux/Windows. | `crates/paneflow-libghostty-sys/build.rs`, fonction `target_spec` | macOS ne linke rien aujourd'hui ; il faut une entrée `TargetSpec` et des clés manifest dédiées. |
| Le lifecycle Ghostty n'utilise que `getpgid`, `waitid(P_PID, WEXITED\|WNOHANG\|WNOWAIT)`, `kill(-pid, …)` et `strsignal`. | `src-app/src/terminal/ghostty_session.rs` | Toutes ces primitives existent sur Darwin. Le port du host est un élargissement de gate, pas une réécriture comme l'a exigé ConPTY. |
| Ces primitives sont gatées `target_os = "linux"` (27 sites) alors qu'un seul `cfg(unix)` existe dans le fichier. | `src-app/src/terminal/ghostty_session.rs:1738` | Les gates sont trop étroites par héritage, pas par contrainte technique. |
| Le motif `any(all(linux, feature), all(windows, arch, env, feature))` est recopié sur environ 344 sites. | 176 dans `pty_session.rs`, 54 `view.rs`, 42 `backend_corpus.rs`, 40 `input.rs`, 26 `service_detector.rs` | Une troisième branche manuelle est le principal risque d'exécution du projet ; l'alias `cfg` doit précéder l'ajout macOS. |
| `auto` ne sélectionne Ghostty que sur Linux et Windows x64 MSVC. | `src-app/src/terminal/view.rs`, `auto_selects_ghostty_for_target()` | Le point de promotion est déjà isolé dans une fonction unique. |
| Le crate `-sys` vérifie déjà header, bindings, archive et build-info sans téléchargement au build Cargo. | `crates/paneflow-libghostty-sys/build.rs`, `native/libghostty/` | La chaîne Darwin doit rester hermétique et reproduire ce contrat. |
| L'énumération de processus macOS existe déjà via libproc. | `src-app/src/workspace/ports.rs` | Toute observation de descendants doit réutiliser cette voie plutôt qu'inventer une troisième politique. |
| La normalisation d'archive Linux repose sur `eu-strip` puis `ar -D`. | `native/libghostty/manifest.toml`, clé `archive_normalization` | elfutils n'existe pas sur Darwin ; une recette Mach-O équivalente doit être définie et prouvée, c'est l'inconnue principale du projet. |
| Les releases macOS ne livrent aujourd'hui qu'Apple Silicon. | `.github/workflows/release.yml`, `.github/workflows/update_cask.yml` | Cibler `aarch64-apple-darwin` seul évite de doubler artefact, matrice CI et décision `lipo` pour zéro utilisateur actuel. |

### Sources primaires externes

- La révision Ghostty épinglée expose un build `lib-vt` et un header C stable ; macOS est la plateforme de référence historique du projet : [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty).
- Zig 0.15.2 sait produire des objets et archives Mach-O pour `aarch64-macos` et embarque les stubs libSystem nécessaires : [Zig documentation](https://ziglang.org/documentation/0.15.2/).
- Cargo attend qu'un build script déclare explicitement les chemins et bibliothèques de linkage natif, et recommande la clé `links` pour coordonner une bibliothèque native : [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
- Les `cfg` personnalisés émis par un build script doivent être déclarés via `cargo::rustc-check-cfg` pour rester compatibles avec le lint `unexpected_cfgs` : [Cargo build script `rustc-check-cfg`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-check-cfg).
- La notarisation Apple exige que tout code exécutable du bundle soit signé avec un identifiant valide et le hardened runtime. Un `.a` lié statiquement ne crée pas de code signable additionnel, contrairement à une dylib embarquée : [Notarizing macOS software before distribution](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution).

### Competitive context

iTerm2, WezTerm, Alacritty, Kitty et Ghostty lui-même démontrent que sur macOS l'Unicode correct, les protocoles clavier modernes et le rendu GPU sont des attentes de base. La différenciation de Paneflow ne vient pas d'une nouvelle fenêtre terminal, mais de l'intégration d'un moteur VT moderne et embeddable à son produit multi-pane et agent-first. Aligner macOS sur libghostty-vt supprime la dernière plateforme à deux moteurs et rend le corpus différentiel représentatif du parc réel.

## Assumptions & Constraints

1. La base de référence est libghostty-vt au commit `ae52f97dcac558735cfa916ea3965f247e5c6e9e`, API 0.1.0, Ghostty 1.3.2-dev et Zig 0.15.2. Tout changement de SHA, header, bindings ou flags invalide les hashes et relance la qualification.
2. La v1 cible `aarch64-apple-darwin`. `x86_64-apple-darwin` reste architecturalement possible mais son artefact et sa matrice sont hors périmètre, car les releases macOS actuelles ne livrent qu'Apple Silicon.
3. Le socle système est macOS 13 Ventura ou supérieur, aligné sur le minimum déjà exigé par le bundle Paneflow existant. Aucune extension de compatibilité vers une version antérieure n'est faite par ce PRD.
4. La consommation standard du workspace et le build Cargo ne téléchargent rien et ne requièrent ni Zig ni un checkout Ghostty. La reconstruction de provenance se déroule dans une lane CI dédiée.
5. Le linkage statique est la cible. Un résultat dylib ne peut pas remplacer cette cible silencieusement.
6. Paneflow reste propriétaire de GPUI, des panes, de l'IME, du clipboard, des keybindings, de la configuration, de la télémétrie, du PTY et du packaging.
7. Libghostty-vt reste propriétaire du parsing VT, de l'état terminal, des snapshots et de l'encodage terminal spécifique au backend.
8. Alacritty reste compilé et disponible sur macOS pendant et après la promotion. Une session ayant créé un child ne peut pas changer de backend.
9. Le fallback automatique est permis seulement si Ghostty échoue avant le spawn. Après spawn, l'erreur est visible et la session est arrêtée proprement au lieu de créer un second child.
10. Le contenu terminal, les commandes, le clipboard et les séquences OSC ne sont jamais ajoutés à la télémétrie ou aux logs de production.
11. L'alias `cfg` d'EP-001 ne change ni la sélection de backend, ni le comportement runtime, ni la surface publique. C'est un refactor à comportement identique sur Linux et Windows.
12. Les changements macOS sont isolés par `cfg` et ne modifient pas le comportement Linux ou Windows.
13. Les scripts, artefacts et workflows natifs doivent fonctionner depuis un chemin macOS contenant des espaces et des caractères non ASCII, et sur un volume APFS sensible à la casse comme insensible.
14. La signature, la notarisation et le format de cask existants ne sont pas modifiés au-delà des notices et métadonnées rendues nécessaires par le composant natif.

## Quality Gates

Tous les gates ci-dessous doivent passer avant US-017. Ils constituent la définition unique de qualification du PRD.

| ID | Gate | Pass condition |
|---|---|---|
| QG-001 | Format, lint et tests workspace sur macOS | `cargo fmt --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` et `cargo test --workspace --locked` passent sur la lane `aarch64-apple-darwin`. |
| QG-002 | Neutralité du refactor de gates | Sur le commit d'EP-001, les builds Linux et Windows produisent le même ensemble de modules et de symboles Ghostty qu'avant le refactor, et leurs suites restent vertes sans modification d'attendu. |
| QG-003 | Build release Ghostty macOS | `cargo build -p paneflow-app --release --target aarch64-apple-darwin --features libghostty-macos --locked` passe depuis un checkout standard sans Zig ni Ghostty local. |
| QG-004 | Provenance et ABI | Le SHA source, la version Zig, les flags, le header, les bindings, le build-info et l'archive correspondent au manifest ; les tests de taille, alignement, symboles et allocation passent ; aucune récupération réseau n'est effectuée par `build.rs`. |
| QG-005 | Linkage statique | L'inspection Mach-O de `paneflow` confirme l'architecture `arm64`, les bibliothèques système attendues et l'absence de toute `libghostty-vt.dylib` ou chemin de chargement Ghostty dynamique. |
| QG-006 | Corpus différentiel | Tous les chunks du corpus backend passent sous macOS avec zéro divergence inexpliquée. Une différence intentionnelle possède une fixture distincte et une justification liée à libghostty-vt. |
| QG-007 | Lifecycle et ressources | 200 cycles spawn-resize-close consécutifs et un scénario de 32 panes concurrents terminent sans deadlock, double-spawn ni processus orphelin. Après warmup, le nombre de descripteurs et le RSS résiduel reviennent à moins de 5 % du niveau de référence. |
| QG-008 | Performance | Sur le même runner release, le débit médian du corpus Ghostty n'est pas inférieur de plus de 10 % à Alacritty, le P95 de création du host avant init shell reste sous 500 ms, et le binaire release augmente de 15 MiB maximum face au build Alacritty-only. |
| QG-009 | Input et protocoles | La matrice US-009 et US-010 passe à 100 % sur clavier US, AZERTY et au moins un layout à dead keys, avec IME, Kitty keyboard, bracketed paste, souris, focus, OSC 52 et hyperlinks. La convention Option-as-Meta est validée dans ses deux réglages. |
| QG-010 | Shells et workflows Paneflow | zsh, bash, fish et nushell passent la matrice complète. Les hooks et agents Paneflow conservent cwd, env, sortie et fermeture. |
| QG-011 | Compatibilité macOS | Le runbook passe sur macOS 13 et sur la version majeure courante, avec resize storms, gros débit, Unicode, Ctrl-C, fermeture de pane, fermeture d'application, veille/reprise, écran Retina et non-Retina, et chemins utilisateur non ASCII. |
| QG-012 | Packaging, signature et notarisation | Le `.app` se signe, se notarise et se staple sans avertissement nouveau ; installation propre, upgrade depuis la dernière release Alacritty-only, lancement, rollback Alacritty et désinstallation passent, y compris via le cask Homebrew. Aucun fichier Ghostty natif résiduel n'est installé. |
| QG-013 | Sécurité et confidentialité | OSC 52 est limité, policy-gated et focus-gated ; les URI ne sont jamais exécutées implicitement ; les queues et payloads sont bornés ; les logs de test et production ne capturent aucun contenu terminal utilisateur. |

## Epics & User Stories

### EP-001: Unifier les gates de compilation du backend Ghostty

**Goal:** Remplacer le motif `cfg` deux-branches dupliqué par un alias unique, afin qu'ajouter une plateforme devienne un changement localisé et reviewable.

**Definition of Done:** US-001 est DONE, aucun site ne recopie plus la disjonction plateforme/feature, et les builds Linux et Windows sont prouvés inchangés.

#### US-001: Introduire l'alias `cfg` unique du backend Ghostty

**Description:** En tant que mainteneur Paneflow, je veux un seul prédicat de compilation exprimant « le backend Ghostty natif est disponible sur cette cible », afin de ne pas recopier une disjonction plateforme/feature sur des centaines de sites à chaque nouvelle plateforme.

**Priority:** P0  
**Size:** M  
**Dependencies:** None

**Acceptance Criteria:**

- [ ] `src-app/build.rs` calcule le prédicat depuis les variables `CARGO_CFG_TARGET_*` et `CARGO_FEATURE_*` et émet un `cfg` nommé, accompagné du `cargo::rustc-check-cfg` correspondant, sans réseau ni dépendance nouvelle.
- [ ] Tous les sites reproduisant la disjonction complète plateforme/feature utilisent l'alias. Les gates volontairement mono-plateforme restent explicites et sont recensées dans la story pour traitement en EP-003.
- [ ] Le lint `unexpected_cfgs` reste silencieux et aucune allocation de `#[allow]` n'est ajoutée pour le contourner.
- [ ] Un test ou une assertion de compilation vérifie que l'alias et la disponibilité effective du backend ne peuvent pas diverger.
- [ ] Les builds `--no-default-features` Alacritty-only et les builds features par défaut produisent exactement les mêmes ensembles de modules qu'avant sur Linux et Windows.
- [ ] **Unhappy path:** une combinaison cible/feature non prévue, par exemple une feature native activée sur une cible sans artefact, produit une erreur de build explicite nommant la cible et la feature, et jamais un backend silencieusement absent.

### EP-002: Établir la fondation native aarch64-apple-darwin

**Goal:** Produire une frontière libghostty-vt Darwin statique, hermétique, sûre et testable avant toute intégration UI.

**Definition of Done:** US-002 à US-004 sont DONE, l'artefact canonique et ses métadonnées sont vérifiés, et un smoke headless consomme le wrapper macOS.

#### US-002: Fermer le spike de build Mach-O reproductible

**Description:** En tant que mainteneur Paneflow, je veux prouver que la révision Ghostty épinglée produit une archive statique `aarch64-apple-darwin` reproductible bit à bit, afin d'éviter de bâtir le port sur une hypothèse de normalisation non vérifiée.

**Priority:** P0  
**Size:** L  
**Dependencies:** None

**Acceptance Criteria:**

- [ ] Un build propre produit une archive Mach-O `arm64` statique depuis le SHA et la version Zig épinglés, avec les flags et commandes consignés dans la documentation native Paneflow.
- [ ] Une recette de normalisation Darwin est définie et documentée en remplacement de `eu-strip` + `ar -D`, en identifiant précisément chaque source de non-déterminisme neutralisée : horodatages d'en-tête d'archive, chemins de cache Zig dans les données de debug, ordre des membres, identifiants de build.
- [ ] Deux builds propres depuis deux caches Zig vierges produisent le même inventaire de symboles et le même hash canonique, ou toute source de non-déterminisme résiduelle est supprimée avant clôture.
- [ ] Les dépendances système transitives et le modèle de runtime C/C++ sont listés avec leur origine, notamment la nécessité éventuelle de lier `c++`.
- [ ] Les symboles C libghostty-vt attendus sont présents et un exécutable minimal lie, initialise, parse puis libère un terminal.
- [ ] **Unhappy path:** si la reproductibilité n'est pas atteinte, la story capture la différence exacte entre les deux builds et un reproducer minimal ; elle ne valide pas un artefact non reproductible comme production par défaut et ne promeut pas une dylib pour contourner le problème.

#### US-003: Distribuer l'artefact macOS de façon hermétique

**Description:** En tant que contributeur, je veux que le crate `-sys` sélectionne un artefact `aarch64-apple-darwin` vérifié depuis le repository, afin qu'un build Paneflow standard n'ait besoin ni de Zig, ni de Ghostty, ni du réseau.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-002

**Acceptance Criteria:**

- [ ] `native/libghostty/` contient une entrée `aarch64-apple-darwin` avec archive, header, bindings, build-info, SHA source, version Zig, flags et SHA-256 de chaque entrée vérifiée.
- [ ] `scripts/build-libghostty-macos.sh` reconstruit l'artefact depuis un checkout propre du SHA épinglé et expose un mode de vérification de reproductibilité, sur le modèle du script Linux existant.
- [ ] Le build script sélectionne l'artefact par target triple et émet les directives Cargo de linkage statique et système requises sans modifier les chemins Linux et Windows.
- [ ] Une reconstruction CI depuis la source épinglée régénère l'artefact canonique et compare ses métadonnées au manifest.
- [ ] Un checkout standard compile avec l'artefact préconstruit sans accès réseau et sans Ghostty ou Zig installés.
- [ ] **Unhappy path:** archive absente, corrompue, d'une mauvaise architecture ou avec un build-info incohérent provoque un échec immédiat contenant target triple, fichier attendu et action corrective, sans fallback silencieux.

#### US-004: Rendre la frontière FFI sûre sur macOS et ajouter le smoke headless

**Description:** En tant que développeur backend, je veux activer le crate `-sys` et le wrapper libghostty-vt sur macOS avec les mêmes invariants RAII que Linux, et valider la frontière sans GPUI ni PTY, afin de séparer les défauts d'artefact et d'ABI des défauts de session ou de rendu.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-003

**Acceptance Criteria:**

- [ ] Les gates `cfg` exposent les crates Ghostty sur Linux, Windows et macOS via l'alias d'US-001, sans réintroduire de disjonction recopiée.
- [ ] Les tailles, alignements, discriminants, signatures et symboles requis sont validés contre le header épinglé sur la cible Darwin.
- [ ] Toute mémoire allouée par libghostty-vt est libérée par l'API Ghostty correspondante ; Rust ou la libc système ne libère jamais directement une allocation Zig.
- [ ] Les handles opaques possèdent un ownership unique, un `Drop` idempotent, et aucune donnée empruntée à durée limitée n'est conservée après le callback ; les callbacks FFI empêchent tout unwind Rust de traverser la frontière native.
- [ ] Un test macOS headless crée un terminal, injecte un fixture VT déterministe, produit un snapshot, encode un input puis détruit toutes les ressources, en couvrant création/destruction répétée, resize, palette, Unicode, alternate screen et lecture de snapshot.
- [ ] Les tests Linux et Windows existants continuent de passer sans changement de leurs résultats attendus.
- [ ] **Unhappy path:** pointeur nul, enum inconnu, buffer invalide, fixture malformée, taille zéro ou initialisation native en échec retourne une erreur structurée ou une absence explicite, sans panic, sans accès mémoire et sans laisser de handle natif vivant.

### EP-003: Porter le host Ghostty sur le PTY Darwin

**Goal:** Réutiliser le worker Ghostty et `portable-pty` avec des primitives POSIX correctement gatées et une politique de processus Darwin déterministe.

**Definition of Done:** Un pane Ghostty macOS exécute un shell réel, termine proprement dans tous les ordres de fermeture et passe les campagnes de stress.

#### US-005: Élargir le host POSIX de Linux à Darwin

**Description:** En tant qu'utilisateur macOS, je veux ouvrir un pane Ghostty qui lance mon shell, afin d'utiliser le backend sans modifier le modèle de pane ou le renderer Paneflow.

**Priority:** P0  
**Size:** M  
**Dependencies:** Blocked by US-001, US-004

**Acceptance Criteria:**

- [ ] Les gates POSIX de la session Ghostty expriment la famille `unix` plutôt que `target_os = "linux"` partout où la primitive concernée existe sur Darwin, et chaque gate restée spécifique à Linux porte une justification explicite.
- [ ] La feature `libghostty-macos` active les modules et dépendances Ghostty sur `aarch64-apple-darwin` sans compiler de primitive Windows.
- [ ] `GhosttySession` utilise `portable_pty::native_pty_system`, `CommandBuilder`, le cwd, l'environnement et la sélection de shell déjà définis par Paneflow.
- [ ] Le master possède un reader clonable et un writer unique ; l'I/O reste hors du thread GPUI et alimente le moteur Ghostty en bytes.
- [ ] Un succès crée exactement un child et publie le backend Ghostty au modèle de session sans changer le format de session persisté.
- [ ] **Unhappy path:** si artefact, PTY ou spawn échoue avant la création du child, la vue peut revenir à Alacritty une seule fois avec un diagnostic sans contenu terminal ; après création du child, aucun second backend n'est lancé.

#### US-006: Valider l'observation et l'arrêt de processus sur Darwin

**Description:** En tant qu'utilisateur multi-pane, je veux que fermer un pane Ghostty arrête son shell et ses descendants sans affecter les autres panes, afin d'éviter processus orphelins, blocages et perte de contrôle.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-005

**Acceptance Criteria:**

- [ ] Le comportement de `waitid` avec `WNOWAIT`, de `getpgid` et des signaux de groupe est vérifié explicitement sur Darwin par des tests, et toute divergence de sémantique face à Linux est traitée dans le code plutôt que supposée absente.
- [ ] L'ordre de shutdown arrête les writes, demande ou force la fin du child, attend ou reap, draine la sortie finale, puis libère PTY et moteur.
- [ ] L'énumération de descendants réutilise la voie libproc déjà présente dans Paneflow au lieu d'introduire une troisième politique de processus.
- [ ] Fermer un pane termine ses descendants tandis que les processus des autres panes restent vivants.
- [ ] Kill, wait, EOF et fermeture de fenêtre concurrents convergent vers un seul état terminal.
- [ ] **Unhappy path:** si une observation de processus échoue ou si le process group a déjà disparu, un fallback borné tente la terminaison disponible, journalise seulement les métadonnées de processus nécessaires et ne bloque pas l'arrêt de Paneflow.

#### US-007: Couvrir drain final, backpressure, resize et stress macOS

**Description:** En tant que mainteneur, je veux une suite déterministe de stress Darwin, afin de détecter les races de lifecycle et les fuites avant la qualification produit.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-005, US-006

**Acceptance Criteria:**

- [ ] Le reader draine continuellement la sortie hors du thread UI et accepte les séquences UTF-8 ou VT découpées entre plusieurs reads ; le final drain traite tous les bytes disponibles avant de publier l'état exited.
- [ ] Les resizes lignes/colonnes sont coalescés, ordonnés et transmis au PTY et à libghostty-vt sans reordering visible ; les queues appliquent les caps de NFR-005.
- [ ] La suite automatise 200 cycles spawn-resize-output-close avec comptage des children, descendants, descripteurs et mémoire avant et après warmup.
- [ ] Un scénario ouvre 32 panes, injecte des resize storms et du gros débit, puis ferme panes et application dans plusieurs ordres.
- [ ] Les scénarios couvrent shell qui quitte immédiatement, shell bloqué, descendant long-lived, Ctrl-C, crash simulé du worker et fermeture brutale de l'application.
- [ ] **Unhappy path:** broken pipe, EOF précoce, resize zéro, resize pendant shutdown, consumer en retard, timeout ou fuite détectée se termine par un état borné et observable, fait échouer la suite avec des métadonnées diagnostiques sans contenu utilisateur, et force un cleanup borné du runner.

### EP-004: Atteindre la parité terminal et workflow macOS

**Goal:** Prouver que le backend Ghostty se comporte comme un terminal Paneflow complet sur le rendu, l'input, le clipboard et les shells macOS.

**Definition of Done:** Le corpus, la matrice input/protocoles et les workflows shell passent sans branche backend dans le renderer GPUI.

#### US-008: Étendre le corpus rendu et interaction à macOS

**Description:** En tant qu'utilisateur, je veux que les contenus complexes s'affichent et se manipulent correctement avec Ghostty, afin que le changement de moteur n'altère pas mon travail.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-004, US-005

**Acceptance Criteria:**

- [ ] Le corpus couvre ASCII, UTF-8 fragmenté, combining marks, emoji, séquences ZWJ, CJK, wide cells, true color, palette, styles, tabs, wrapping, reflow et alternate screen.
- [ ] Les snapshots couvrent cursor, scrollback, viewport, selection, search, hyperlinks et damage après resize.
- [ ] Les chunks différentiels macOS s'exécutent dans le plan CI existant et distinguent divergence Ghostty intentionnelle de régression Paneflow.
- [ ] `terminal/element/paint` reste backend-neutre et aucune branche macOS ou Ghostty n'est ajoutée à la géométrie, aux couleurs ou aux fonts.
- [ ] Le rendu est validé sur écran Retina et non-Retina, la mise à l'échelle des cellules restant identique entre backends.
- [ ] **Unhappy path:** séquence VT inconnue, OSC surdimensionné, grapheme incomplet ou resize extrême ne panic pas, ne lit pas hors limites et respecte les caps de ressources.

#### US-009: Intégrer le clavier macOS, Option-as-Meta, dead keys et IME

**Description:** En tant qu'utilisateur d'un clavier international sur macOS, je veux que raccourcis, texte composé et protocoles clavier soient encodés correctement, afin que Ghostty soit utilisable au quotidien hors clavier US.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-005

**Acceptance Criteria:**

- [ ] Les événements clavier destinés au terminal Ghostty utilisent une voie d'encodage backend-aware couvrant legacy VT et Kitty keyboard sans contourner les keybindings Paneflow.
- [ ] La convention macOS d'Option est traitée explicitement dans ses deux réglages : Option produisant le caractère composé, et Option traité comme Meta ou ESC-prefix, avec le même résultat que le backend Alacritty pour la configuration équivalente.
- [ ] Les touches mortes et au moins un layout AZERTY produisent exactement le texte ou la séquence attendue sans générer de raccourci parasite.
- [ ] L'IME Core Text conserve son preedit visuel dans GPUI, et seul le commit écrit l'UTF-8 final une fois dans le PTY.
- [ ] Les raccourcis Command restent traités par Paneflow et ne sont jamais encodés vers le PTY, tandis que Control conserve sa sémantique terminal.
- [ ] **Unhappy path:** touche non mappable, séquence IME annulée, changement de layout ou encodeur Ghostty refusant un événement n'écrit aucun byte corrompu et ne déclenche pas de panic.

#### US-010: Valider paste, clipboard, souris, focus et liens sur macOS

**Description:** En tant qu'utilisateur de TUI, je veux que les protocoles d'interaction terminal fonctionnent avec les mêmes politiques Paneflow, afin de conserver sécurité et ergonomie.

**Priority:** P0  
**Size:** M  
**Dependencies:** Blocked by US-005, US-009

**Acceptance Criteria:**

- [ ] Normal paste et bracketed paste préservent bytes, retours de ligne et Unicode selon les conventions macOS existantes, y compris depuis une source produisant des fins de ligne CR.
- [ ] Les modes souris, wheel, drag, motion et focus report sont encodés par le backend actif sans doubler les événements GPUI ; le scroll inertiel ne produit pas de rafale d'événements non bornée.
- [ ] Une écriture OSC 52 n'est acceptée que pour un terminal focalisé, selon la policy Paneflow, et avec un payload décodé maximal de 100 KiB.
- [ ] Les hyperlinks sont exposés comme données et ne déclenchent jamais une ouverture ou exécution sans action utilisateur explicite et validation de protocole.
- [ ] La selection et la copie Paneflow restent fonctionnelles dans main screen, alternate screen et scrollback.
- [ ] **Unhappy path:** base64 invalide, OSC 52 hors focus ou surdimensionné, URI invalide, paste pendant shutdown ou événement souris hors viewport est ignoré ou rejeté proprement sans modifier le presse-papiers.

#### US-011: Qualifier les shells et workflows Paneflow réels sur macOS

**Description:** En tant que développeur macOS, je veux utiliser Ghostty avec mes shells, TUIs et agents habituels, afin que le backend couvre le produit plutôt qu'une démo synthétique.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-007, US-008, US-009, US-010

**Acceptance Criteria:**

- [ ] zsh, bash, fish et nushell valident lancement, cwd, environnement, Unicode, couleurs, Ctrl-C, exit code, resize, scrollback et fermeture.
- [ ] Les hooks shell, OSC 133, OSC 7, commandes Paneflow, agents et injection de contexte conservent leur comportement et n'écrivent pas deux fois.
- [ ] La restauration de workspace relance un nouveau child avec le backend configuré sans changer le format de session persistant.
- [ ] Les chemins utilisateur contenant espaces et caractères non ASCII fonctionnent pour cwd, config, shell et assets, y compris sous une forme Unicode décomposée telle que produite par le Finder.
- [ ] Le bridge MCP et les binaires helper continuent de lire les panes Ghostty comme les panes Alacritty.
- [ ] **Unhappy path:** shell configuré absent, hook en échec ou agent qui ne termine pas produit un état explicite et récupérable sans second child ni blocage du pane.

### EP-005: Industrialiser CI, sécurité, performance et packaging

**Goal:** Transformer le backend fonctionnel en composant publiable, reproductible et observable dans la release macOS.

**Definition of Done:** La CI reconstruit et qualifie l'intégration, les budgets sont respectés et un `.app` notarisé installe la version statique avec ses notices.

#### US-012: Ajouter la lane CI native macOS et les contrôles supply-chain

**Description:** En tant que mainteneur release, je veux reconstruire et vérifier l'artefact Ghostty macOS séparément de sa consommation Cargo, afin de détecter toute dérive de source, outil ou ABI.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-003, US-004

**Acceptance Criteria:**

- [ ] Une lane `aarch64-apple-darwin` reconstruit libghostty-vt depuis le SHA épinglé avec Zig épinglé et publie logs, inventaire de symboles, build-info et hash, sur le modèle des workflows Linux et Windows existants.
- [ ] Une lane consumer distincte utilise uniquement l'artefact du repository et exécute les quality gates Cargo macOS.
- [ ] Les caches sont clés par SHA Ghostty, version Zig, target triple et flags ; aucun cache d'une autre architecture ou configuration n'est accepté.
- [ ] Header, bindings, archive, licences et notices sont vérifiés avant le build Paneflow, et `scripts/verify-libghostty-package.sh` couvre la cible Darwin.
- [ ] Les artefacts de release permettent de retracer chaque binaire au manifest sans inclure de contenu terminal ou secret CI.
- [ ] **Unhappy path:** dérive de hash, binding, licence, symbole, architecture ou toolchain fait échouer la lane avant packaging et interdit toute substitution automatique.

#### US-013: Enforcer corpus, performance et budgets de ressources sur macOS

**Description:** En tant que mainteneur, je veux comparer Ghostty à la référence Alacritty sur le même runner, afin que la promotion repose sur des seuils et non sur une impression visuelle.

**Priority:** P0  
**Size:** M  
**Dependencies:** Blocked by US-007, US-008, US-009

**Acceptance Criteria:**

- [ ] Le corpus différentiel, les benchmarks de parsing/snapshot, le temps de création host, la taille binaire et le stress ressources sont exécutés en release sur un runner contrôlé.
- [ ] Les baselines Alacritty et Ghostty utilisent fixtures, taille, runner et nombre d'itérations identiques avec médiane et P95 publiés.
- [ ] Les seuils QG-006, QG-007 et QG-008 sont évalués automatiquement et historisés par commit.
- [ ] Toute variance runner supérieure à la tolérance documentée déclenche un rerun borné unique, puis un échec si elle persiste.
- [ ] **Unhappy path:** régression au-delà d'un budget bloque la promotion ; elle ne peut être masquée par une mise à jour de baseline sans justification versionnée et review.

#### US-014: Intégrer le linkage statique au bundle, à la signature et au cask

**Description:** En tant qu'utilisateur, je veux installer Paneflow Ghostty avec le `.app` ou le cask normal, afin de ne gérer ni runtime Ghostty séparé, ni dylib, ni chemin système.

**Priority:** P0  
**Size:** L  
**Dependencies:** Blocked by US-012, US-013

**Acceptance Criteria:**

- [ ] Le build release macOS embarque un binaire `arm64` lié statiquement à libghostty-vt et n'ajoute aucune `libghostty-vt.dylib` au bundle.
- [ ] L'inspection des bibliothèques liées et des fichiers installés confirme uniquement les dépendances système et runtime explicitement approuvées.
- [ ] La signature, le hardened runtime, la notarisation et le stapling passent sans avertissement nouveau et sans modification du modèle de trust de l'updater.
- [ ] Les licences, notices et éléments SBOM de Ghostty et de ses dépendances natives sont présents dans les artefacts de release.
- [ ] Installation propre, upgrade depuis la dernière release macOS, lancement Ghostty, lancement Alacritty, mise à jour via le cask Homebrew et désinstallation sont automatisés.
- [ ] **Unhappy path:** dépendance dylib Ghostty, architecture incorrecte, échec de notarisation, fichier résiduel après désinstallation ou licence manquante fait échouer le package smoke et bloque la publication.

#### US-015: Exécuter la matrice macOS et publier le runbook de diagnostic

**Description:** En tant que mainteneur, je veux une qualification reproductible sur les macOS supportés et un runbook de rollback, afin de diagnostiquer un échec sans dépendre de la machine d'origine.

**Priority:** P0  
**Size:** M  
**Dependencies:** Blocked by US-011, US-014

**Acceptance Criteria:**

- [ ] Le runbook couvre macOS 13 et la version majeure courante, écran Retina et externe, veille/reprise, session distante si disponible et chemins non ASCII.
- [ ] Chaque shell obligatoire et chaque interaction de QG-009 à QG-012 possède une étape, un attendu et une preuve enregistrable sans contenu utilisateur.
- [ ] Le diagnostic expose backend demandé, backend effectif, phase d'échec, version Ghostty, target et code OS, sans commande, sortie, clipboard ou chemin sensible complet.
- [ ] Le rollback vers Alacritty est documenté, testable en une modification de configuration et ne requiert ni réinstallation ni suppression de données.
- [ ] Les limites connues et skips conditionnels sont distingués des échecs.
- [ ] **Unhappy path:** machine incompatible, PTY indisponible, GPU/driver défaillant ou politique de sécurité bloquante produit un diagnostic actionnable et conserve Alacritty comme chemin de récupération.

### EP-006: Qualifier, promouvoir et rendre le rollback explicite

**Goal:** Exposer Ghostty aux utilisateurs avancés macOS, accumuler les preuves de qualification puis basculer le choix `auto` sans supprimer Alacritty.

**Definition of Done:** Le backend est configurable, la qualification est complète, `auto` sélectionne Ghostty sur macOS et le rollback Alacritty reste vérifié.

#### US-016: Ajouter la sélection macOS et le mode de qualification

**Description:** En tant que contributeur macOS, je veux activer explicitement Ghostty et voir le backend effectif, afin de qualifier la feature avant sa promotion générale.

**Priority:** P0  
**Size:** S  
**Dependencies:** Blocked by US-005, US-007

**Acceptance Criteria:**

- [ ] La configuration macOS accepte `auto`, `ghostty` et `alacritty` avec validation, sérialisation et migration backward-compatible.
- [ ] Pendant la qualification, `auto` conserve Alacritty tandis que `ghostty` demande le backend Ghostty.
- [ ] Le backend demandé, le backend effectif et un échec pré-spawn sont visibles dans les diagnostics sans contenu terminal.
- [ ] Un fallback Ghostty vers Alacritty ne peut arriver qu'avant child spawn et au plus une fois.
- [ ] Linux et Windows conservent leur sélection actuelle.
- [ ] **Unhappy path:** valeur inconnue, feature native absente, artefact refusé ou échec PTY pré-spawn retourne vers Alacritty avec raison structurée ; un échec post-spawn ferme la session sans lancer Alacritty.

#### US-017: Promouvoir Ghostty comme backend auto sous macOS

**Description:** En tant qu'utilisateur macOS, je veux que Paneflow choisisse Ghostty automatiquement après qualification, afin de bénéficier du moteur moderne sans configuration manuelle tout en gardant un rollback sûr.

**Priority:** P0  
**Size:** S  
**Dependencies:** Blocked by US-011, US-012, US-013, US-014, US-015, US-016

**Acceptance Criteria:**

- [ ] Tous les quality gates QG-001 à QG-013 possèdent une preuve verte sur le commit candidat.
- [ ] Sur macOS Apple Silicon supporté, `auto` sélectionne Ghostty pour toute nouvelle session ; `ghostty` reste explicite et `alacritty` force le backend historique.
- [ ] La promotion ne change ni la sélection Linux, ni la sélection Windows, ni le format des workspaces et sessions persistés.
- [ ] La documentation utilisateur, `CLAUDE.md`, `ARCHITECTURE.md`, les release notes et le runbook décrivent le support macOS, la sélection explicite, les limites et le rollback.
- [ ] La release candidate passe une dernière installation, upgrade, smoke Ghostty et smoke Alacritty avant publication.
- [ ] **Unhappy path:** si un gate régresse, si l'artefact natif n'est pas vérifiable ou si le smoke de packaging échoue, `auto` reste ou revient à Alacritty avant publication ; aucune exception manuelle non versionnée ne permet la promotion.

## Functional Requirements

| ID | Requirement | Stories |
|---|---|---|
| FR-001 | La disponibilité du backend Ghostty natif doit être exprimée par un prédicat de compilation unique et vérifié. | US-001 |
| FR-002 | Paneflow doit consommer une libghostty-vt statique `aarch64-apple-darwin` issue du SHA épinglé et vérifiée par manifest. | US-002, US-003, US-012 |
| FR-003 | Le crate `-sys` et le wrapper doivent exposer la même API sûre sur les trois plateformes sans allocation traversant le mauvais allocateur. | US-004 |
| FR-004 | Un build macOS standard doit fonctionner sans réseau, Zig ou checkout Ghostty. | US-003, US-012 |
| FR-005 | `GhosttySession` doit utiliser `portable-pty` sur Darwin avec un seul child par session. | US-005 |
| FR-006 | Spawn, exit, kill, wait, drain et fermeture doivent être ordonnés et idempotents sur Darwin. | US-006, US-007 |
| FR-007 | Fermer un pane ou Paneflow doit terminer les descendants concernés sans affecter les autres panes. | US-006, US-007 |
| FR-008 | Output fragmenté, Unicode, resize et final drain doivent produire un snapshot complet et cohérent. | US-007, US-008 |
| FR-009 | Le renderer GPUI doit rester backend-neutre et consommer `Content` sans branche macOS spécifique. | US-008 |
| FR-010 | Clavier, Option-as-Meta, dead keys, IME Core Text et Kitty keyboard doivent suivre une voie backend-aware et respecter les keybindings Paneflow. | US-009 |
| FR-011 | Paste, mouse, focus, clipboard OSC 52, selection et hyperlinks doivent conserver les policies Paneflow. | US-010 |
| FR-012 | zsh, bash, fish et nushell doivent couvrir la matrice de workflow. | US-011 |
| FR-013 | La CI doit reconstruire la source épinglée, vérifier supply-chain, ABI, corpus, ressources et performance. | US-012, US-013 |
| FR-014 | Le bundle macOS doit distribuer Paneflow sans dylib Ghostty, en conservant signature et notarisation, et inclure licences, notices et SBOM. | US-014 |
| FR-015 | Les diagnostics doivent identifier backend et phase d'échec sans journaliser le contenu terminal. | US-007, US-015, US-016 |
| FR-016 | La configuration doit supporter `auto`, `ghostty` et `alacritty` ; le fallback automatique doit rester pré-spawn. | US-016, US-017 |
| FR-017 | La promotion de `auto` vers Ghostty doit être conditionnée à tous les gates et conserver un rollback Alacritty. | US-017 |

## Non-Functional Requirements

| ID | Category | Requirement | Measurement |
|---|---|---|---|
| NFR-001 | Compatibility | Supporter macOS 13+ sur Apple Silicon avec target `aarch64-apple-darwin`. | Build CI plus runbook macOS 13 et version majeure courante. |
| NFR-002 | Performance | Le débit médian Ghostty ne doit pas être inférieur de plus de 10 % à Alacritty sur le corpus release identique. | Benchmark comparatif, minimum 20 itérations après warmup. |
| NFR-003 | Startup | Le P95 de création du host Ghostty avant initialisation du shell doit rester inférieur à 500 ms sur le runner de référence. | 100 créations séquentielles en release. |
| NFR-004 | Binary size | Le binaire Paneflow release Ghostty ne doit pas dépasser de plus de 15 MiB le build Alacritty-only équivalent. | Taille du même commit et même target après packaging comparable. |
| NFR-005 | Memory bounds | Chaque queue runtime est bornée ; output en attente maximal 8 MiB, input en attente maximal 1 MiB, OSC 52 décodé maximal 100 KiB. | Tests de saturation et assertions de configuration. |
| NFR-006 | Lifecycle reliability | 200 cycles et 32 panes doivent produire zéro deadlock, double-spawn ou processus orphelin. | Suite US-007 avec timeout global et inventaire de processes. |
| NFR-007 | Resource recovery | Après warmup et cleanup, descripteurs de fichiers et RSS résiduels doivent revenir à moins de 5 % de la baseline. | Compteurs avant/après campagne QG-007. |
| NFR-008 | UI responsiveness | Aucun read, wait ou kill bloquant ne s'exécute sur le thread GPUI ; une rafale output/resize ne doit pas bloquer une frame plus de 16,7 ms au P95 sur le runner de référence. | Instrumentation test et trace de frame pendant stress. |
| NFR-009 | Supply-chain | 100 % des artefacts natifs, headers, bindings et build-info doivent être hashés et reliés au SHA source et à la version Zig. | Vérification manifest en CI. |
| NFR-010 | Privacy | Zéro byte de commande, output, clipboard ou séquence OSC utilisateur dans les logs et événements de télémétrie de production. | Tests de redaction avec canaries et audit des champs émis. |
| NFR-011 | Security | Aucun chargement de dylib Ghostty ; OSC 52 focus-gated et policy-gated ; aucune URI terminal exécutée implicitement ; hardened runtime conservé. | Inspection des bibliothèques liées, tests protocoles et security review ciblée. |
| NFR-012 | Regression safety | Les suites Linux et Windows existantes doivent rester inchangées et vertes, y compris sur le commit du refactor de gates. | Matrice CI multi-plateforme du commit candidat. |
| NFR-013 | Maintainability | Aucun nouveau site ne doit recopier la disjonction plateforme/feature du backend Ghostty ; ajouter une plateforme doit rester un changement localisé. | Revue de diff et contrôle automatisé sur le motif interdit. |

## Edge Cases & Error States

| Case | Expected behavior | Coverage |
|---|---|---|
| Artefact absent, hash invalide ou mauvaise architecture | Échec de build immédiat et actionnable, aucune substitution. | US-003, US-012 |
| Archive Darwin non reproductible entre deux builds propres | Le spike ne se ferme pas ; la source de non-déterminisme est identifiée avant toute consommation. | US-002 |
| Incompatibilité ABI ou runtime C++ | Smoke natif et contrat FFI échouent avant intégration runtime. | US-002, US-004 |
| Feature native activée sur une cible sans artefact reviewé | Erreur de build explicite nommant cible et feature, jamais un backend silencieusement absent. | US-001, US-003 |
| Spawn PTY refusé avant création du child | Fallback Alacritty unique, sinon session en erreur. | US-005, US-016 |
| Shell quitte avant que la vue soit prête | Exit et final drain sont publiés une fois, sans hang ni fallback post-spawn. | US-006, US-007 |
| Shell lance des descendants long-lived | Fermeture du pane termine l'arbre concerné selon la politique Darwin. | US-006, US-007 |
| Sémantique `waitid`/`WNOWAIT` divergente de Linux | Traitée explicitement dans le code et couverte par test, jamais supposée identique. | US-006 |
| UTF-8 ou VT découpé entre reads | Les bytes sont conservés dans l'ordre et parsés sans remplacement prématuré. | US-007, US-008 |
| Resize zéro, storm ou resize pendant shutdown | Coalescing, clamp et arrêt idempotent, sans appel invalide ni deadlock. | US-007 |
| Output dépasse la capacité du consumer | Backpressure bornée, thread UI réactif, erreur explicite si la policy de saturation s'active. | US-007 |
| Option pressée avec un layout composant un caractère | Le réglage actif décide ; aucun byte parasite n'est envoyé dans l'autre mode. | US-009 |
| IME Core Text annulé, dead key ou changement de layout | Aucun byte partiel ou raccourci parasite n'est envoyé. | US-009 |
| Scroll inertiel prolongé | Événements bornés et coalescés, thread GPUI réactif. | US-010 |
| OSC 52 invalide, hors focus ou trop grand | Rejet sans changement du presse-papiers et sans log du payload. | US-010 |
| URI malformée ou protocole non approuvé | Affichage possible comme texte, jamais d'exécution implicite. | US-010 |
| Chemin utilisateur en Unicode décomposé | cwd, config, shell et assets fonctionnent sans normalisation destructive. | US-011 |
| Paneflow crash ou fermeture forcée | Les descendants sont nettoyés selon la politique existante ; le prochain lancement reste sain. | US-006, US-007 |
| Notarisation refusée après ajout du composant natif | Le packaging échoue avant publication et le motif Apple est consigné. | US-014 |
| Upgrade depuis une release sans Ghostty macOS | Config migrée sans perte, `auto` suit la règle de la nouvelle version, `alacritty` reste sélectionnable. | US-014, US-016, US-017 |
| Gate régresse après qualification | Promotion bloquée ou `auto` rebasculé avant publication ; aucune release partielle. | US-013, US-017 |

## Risks & Mitigations

| Risk | Probability | Impact | Mitigation | Trigger / owner |
|---|---|---|---|---|
| L'archive Mach-O n'est pas reproductible sans équivalent d'`eu-strip` et `ar -D`. | High | Critical | Spike US-002 avant toute dépendance runtime ; recette de normalisation documentée source par source ; comparaison de deux builds depuis caches vierges ; go/no-go explicite du projet sur ce résultat. | Divergence de hash entre deux builds propres, owner EP-002. |
| Le refactor de gates casse silencieusement une branche Linux ou Windows non compilable localement. | Medium | Critical | Alias calculé en un seul point ; assertion liant alias et disponibilité effective ; QG-002 exige une preuve CI multi-plateforme avant merge ; la story reste isolée et mergeable seule. | Diff de symboles ou de modules sur une lane non-macOS, owner EP-001. |
| Une allocation Zig est libérée par la libc système. | Low | Critical | Wrapper RAII, API `ghostty_free` exclusive, tests allocate/free répétés et audit ABI. | Crash heap ou sanitizer, owner US-004. |
| La sémantique `waitid`/`WNOWAIT` de Darwin diffère de Linux et fait fuiter des zombies. | Medium | High | Test dédié dès US-006 plutôt qu'hypothèse ; fallback borné ; inventaire de processus dans la suite de stress. | Zombie ou reap manquant en stress, owner US-006. |
| Option-as-Meta, dead keys ou IME sont interprétés comme raccourcis. | Medium | High | Matrice layout réelle incluant AZERTY, séparation preedit/commit, encodeur backend-aware et tests bytes exacts. | Divergence US-009, owner EP-004. |
| Le composant natif perturbe signature, hardened runtime ou notarisation. | Low | High | Cible statique uniquement ; smoke de packaging complet avant publication ; toute dylib exige une décision séparée. | Refus Apple ou avertissement nouveau, owner US-014. |
| Les snapshots Ghostty divergent et poussent à modifier le renderer. | Medium | High | Corpus avant UI, adaptateur `Content`, fixtures distinctes seulement pour différences justifiées. | Branche backend détectée dans `paint`, owner US-008. |
| Le binaire ou le runtime régresse fortement. | Medium | Medium | Budgets automatiques taille, parsing, startup, frames, descripteurs et RSS. | Seuil QG-007/QG-008 dépassé, owner US-013. |
| Une dylib apparaît comme contournement rapide. | Low | High | Gate statique et inspection du bundle ; toute dylib exige une décision de scope séparée et une security review. | Dépendance Ghostty dynamique dans le binaire ou le package, owner US-014. |
| Le SHA Ghostty ou le C API change avant release. | Medium | Medium | Pin strict, drift checks, pas d'upgrade dans ce PRD. | Manifest modifié, owner US-012. |
| Le scope Intel retarde Apple Silicon. | Medium | Medium | `x86_64-apple-darwin` hors v1, chemins et manifest target-aware, décision séparée après promotion. | Demande d'artefact Intel, owner produit. |
| La promotion masque des problèmes rares de machine réelle. | Medium | High | Phase de qualification explicite, Alacritty maintenu, diagnostic privacy-safe, rollback en une config. | Régression release candidate, owner US-017. |

## Non-Goals

1. Porter l'application Ghostty officielle, son interface ou sa configuration sur Paneflow.
2. Remplacer GPUI, le renderer Paneflow, le système de panes ou la persistence de workspace.
3. Supprimer Alacritty du binaire ou du codebase macOS.
4. Changer le backend d'une session après le spawn de son child.
5. Livrer `x86_64-apple-darwin` ou un binaire universel dans cette version.
6. Étendre le support macOS sous la version minimale déjà exigée par le bundle Paneflow.
7. Mettre à jour le SHA Ghostty, Zig, GPUI ou Alacritty sans nécessité directe pour ce PRD.
8. Introduire une dylib Ghostty de production tant que le spike statique n'a pas démontré un blocage irréductible.
9. Modifier le modèle de signature, le trust de l'updater ou le format de cask au-delà des métadonnées et notices nécessaires.
10. Ajouter de la télémétrie de contenu terminal, commandes, clipboard ou OSC.
11. Refactoriser les gates `cfg` au-delà du motif de disponibilité du backend Ghostty.
12. Garantir le support d'un shell tiers non présent dans la matrice, tout en conservant un comportement générique via le PTY Darwin.

## Files NOT to Modify

| Path | Protection |
|---|---|
| `src-app/src/terminal/element/paint/**` | Ne pas ajouter de branche Ghostty ou macOS au renderer, aux fonts, aux couleurs ou à la géométrie. |
| `src-app/src/terminal/element/golden/**` | Ne pas réécrire en masse les goldens pour masquer une divergence. Ajouter une fixture ciblée et justifiée si nécessaire. |
| `src-app/src/app/session.rs` | Ne pas changer le format persistant pour représenter un handle ou un type Ghostty concret. |
| `src-app/src/update/signature.rs`, `src-app/src/update/verified_download.rs` | Ne pas modifier le trust, les clés ou le protocole updater pour distribuer le backend. |
| `native/libghostty/prebuilt/*-unknown-linux-gnu/**` | Ne pas remplacer ou régénérer les artefacts Linux pendant l'ajout macOS. |
| `native/libghostty/prebuilt/x86_64-pc-windows-msvc/**` | Ne pas remplacer ou régénérer l'artefact Windows pendant l'ajout macOS. |
| `native/libghostty/bindings.rs`, `native/libghostty/manifest.toml` clés existantes | Ne modifier que par ajout de clés Darwin ; ne pas régénérer les bindings ni toucher aux hashes des autres cibles. |
| Entrées GPUI de `Cargo.toml` et `src-app/Cargo.toml` | Ne pas remplacer les dépendances GPUI ou leur révision dans ce projet. |

## Technical Considerations

1. **Alias `cfg`:** recommander un `cfg` nommé émis par `src-app/build.rs` depuis `CARGO_CFG_TARGET_*` et `CARGO_FEATURE_*`, déclaré via `cargo::rustc-check-cfg` pour ne pas déclencher `unexpected_cfgs`. Le nom retenu doit être préfixé projet pour éviter toute collision avec un `cfg` amont.
2. **Gates mono-plateforme:** distinguer dès US-001 les gates qui expriment la disponibilité du backend de celles qui expriment une primitive OS. Les premières deviennent l'alias, les secondes restent explicites et sont réévaluées en US-005.
3. **Cible de déploiement:** épingler la version macOS dans le triple (`aarch64-macos.13.0.0`), jamais la laisser implicite. `-Dtarget=aarch64-macos` la résout depuis l'hôte de build, ce qui ferait fuiter la version d'OS du builder dans l'artefact et casse le link quand le SDK est plus ancien que l'hôte. Le manifest doit porter une clé `macos_deployment_target` et le `build-info` doit l'enregistrer pour que `build.rs` la vérifie, comme Windows le fait pour `source_date_epoch`. Mesuré en US-002.
4. **Normalisation Mach-O:** privilégier `zig ar` (llvm-ar, accepte `D`) et `zig objcopy`, qui sortent Xcode des entrées de normalisation et gardent la recette aussi hermétique que celle de Linux. À défaut, évaluer `llvm-strip -S`, `llvm-ar -D` et `libtool -static -D` comme équivalents de la recette Linux. Les archives d'objets ne portent pas de `LC_UUID`, qui est propre aux images liées, ce qui laisse essentiellement les en-têtes `ar` et les chemins de cache Zig comme sources de non-déterminisme. Cette hypothèse doit être vérifiée, pas supposée.
5. **Linkage système:** vérifier si les objets support compilés depuis les en-têtes libc++ imposent de lier `c++` sur Darwin, là où la cible Linux ne déclare aucune bibliothèque système. La liste doit venir du manifest, pas d'une détection implicite.
6. **Features Cargo:** conserver `libghostty-macos` comme feature explicite et target-specific, en cohérence avec `libghostty-linux` et `libghostty-windows`. Une généralisation en `libghostty-native` ne se justifie que si elle réduit réellement la surface sans masquer les artefacts par cible.
7. **Frontière host:** privilégier l'élargissement des gates POSIX existantes plutôt qu'une abstraction nouvelle. Une abstraction ne se justifie que si une divergence Darwin réelle apparaît en US-006.
8. **Input:** évaluer une voie unique transformant un événement GPUI en key event Ghostty, en gardant le commit IME comme bytes UTF-8 et les keybindings Paneflow en priorité. La distinction Command/Control/Option est le point spécifique macOS à modéliser explicitement.
9. **Snapshots:** comparer les modèles `Content` plutôt que les pixels pour le corpus automatisé. Les smoke visuels vérifient seulement l'intégration GPUI et le packaging.
10. **Artefact canonique:** stocker target triple, SHA source, Zig, flags, dépendances système et hashes dans une seule entrée manifest validée par `build.rs` et CI, en réutilisant la structure de clés déjà en place.
11. **Observabilité:** erreurs structurées avec phase, backend, code OS et version native. Les chemins utilisateur sont réduits ou hashés et tout contenu PTY reste absent.
12. **Intel:** conserver les sélections et chemins indexés par target triple dès maintenant, mais ne créer aucun faux artefact ou gate vert `x86_64-apple-darwin` sans runner et bibliothèque correspondants.

## Success Metrics

| Metric | Baseline | Target | Timeframe | Measurement |
|---|---|---|---|---|
| Sites recopiant la disjonction plateforme/feature | Environ 344 | 0 | Release contenant US-001 | Contrôle automatisé sur le motif interdit. |
| Sessions macOS `auto` utilisant Ghostty | 0 % | 100 % des nouvelles sessions sur macOS Apple Silicon supporté après promotion | Release contenant US-017 | Test d'intégration de sélection backend et diagnostic backend effectif. |
| Quality gates macOS Ghostty | 0 sur 13 | 13 sur 13 verts sur le commit candidat | Avant release candidate | Statuts CI et runbook signés par commit. |
| Divergences corpus inexpliquées | Non mesuré sur macOS Ghostty | 0 | Avant US-017 | Rapport de corpus différentiel par chunk. |
| Fiabilité lifecycle | Aucun scénario Ghostty macOS | 200 cycles et 32 panes, zéro deadlock, double-spawn ou orphan | Avant release candidate | Suite US-007 avec inventaire process/descripteurs. |
| Overhead parsing | Non mesuré | Régression médiane inférieure ou égale à 10 % face à Alacritty | Avant US-017 et à chaque upgrade Ghostty | Benchmark release sur runner contrôlé. |
| Overhead binaire | 0 MiB pour Ghostty macOS | Inférieur ou égal à 15 MiB | Avant packaging final | Comparaison des binaires du même commit. |
| Dépendance Ghostty dynamique | Aucune, backend absent | 0 dylib Ghostty liée ou installée | Chaque build release | Inspection des bibliothèques liées et inventaire du bundle. |
| Plateformes sur un moteur VT unique | 2 sur 3 | 3 sur 3 | Release contenant US-017 | Matrice de backend effectif par plateforme. |
| Confidentialité diagnostic | Logs backend macOS inexistants | 0 canary terminal/clipboard détectée | Avant promotion et en CI continue | Tests de redaction avec canaries. |
| Rollback utilisateur | Alacritty seul | Retour à Alacritty en une modification de configuration, sans réinstallation | Release candidate | Étape automatisée du smoke de packaging. |
| Régressions critiques confirmées | Baseline à établir pendant qualification | 0 issue P0 non corrigée et au plus 2 issues P1 confirmées | 30 jours après release | Issues GitHub labelisées macOS/Ghostty, crash telemetry opt-in sans contenu terminal. |

## Open Questions

| Question | Current decision | Owner | Deadline / dependency |
|---|---|---|---|
| Une archive statique Mach-O produite par Zig est-elle reproductible bit à bit après normalisation Darwin ? | Toujours ouverte : aucune archive n'a encore été produite. Voir `tasks/us-002-macos-zig-spike-findings.md`. La cible reste statique et reproductible ; aucune alternative n'est promue par défaut. | EP-002 | Résolution obligatoire dans US-002 avant US-003. C'est le go/no-go du projet. |
| Sur quel environnement l'artefact macOS peut-il être construit ? | Zig 0.15.2 ne compile pas nativement quand le SDK installé est plus ancien que le macOS hôte : `libSystem.tbd` ne déclare aucun symbole pour la cible détectée et tout libc devient indéfini. Un SDK au moins égal à la version de l'OS est requis, sinon la production se fait uniquement sur runner CI. | US-002 / US-012 | Bloque la première production d'artefact. |
| Faut-il lier `c++` sur Darwin pour fermer les objets support ? | À déterminer par inspection de symboles pendant le spike ; la liste finale vient du manifest. | US-002 | Avant US-003. |
| La sémantique `waitid` avec `WNOWAIT` de Darwin est-elle identique à celle de Linux pour la politique de reap actuelle ? | Ne pas supposer l'équivalence ; couvrir par test dédié et traiter toute divergence dans le code. | US-006 | Avant validation du lifecycle. |
| Quel réglage Option-as-Meta doit être le défaut macOS de Paneflow ? | Aligner sur le comportement actuel du backend Alacritty macOS pour ne pas changer l'expérience pendant la migration de moteur. | US-009 | Avant US-011. |
| Une différence de snapshot Ghostty doit-elle devenir une fixture spécifique ? | Seulement si elle est conforme au protocole, intentionnelle et documentée ; le renderer ne change pas pour égaliser artificiellement. | US-008 | Pendant chaque chunk corpus. |
| Quand ouvrir la cible `x86_64-apple-darwin` ou un binaire universel ? | Après promotion Apple Silicon et seulement si la demande utilisateur le justifie. | Product / release | Décision séparée, ne bloque pas US-017. |

[/PRD]
