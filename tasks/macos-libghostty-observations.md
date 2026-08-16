# Observations hors périmètre - chantier macOS libghostty

Ce fichier recense ce qui est découvert **pendant** le chantier macOS libghostty
mais qui n'en fait pas partie. Rien ici n'est corrigé dans cette branche : le
but est de ne pas mélanger des correctifs opportunistes avec la feature, tout
en ne perdant pas l'information.

Chaque entrée porte une sévérité, ce qui a été observé, et pourquoi c'est jugé
hors périmètre.

Statuts : `OPEN` (à traiter plus tard), `WATCH` (pas un défaut aujourd'hui,
à surveiller), `RESOLVED` (traité ailleurs).

---

## OBS-001 - `block v0.1.6` sera rejeté par une future version de Rust

**Sévérité :** basse
**Statut :** OPEN
**Découvert :** 2026-08-15, pendant le premier `cargo check` macOS

Chaque build macOS émet :

```
warning: the following packages contain code that will be rejected by a future
version of Rust: block v0.1.6
```

`block` est une dépendance transitive de la pile Objective-C de GPUI. Le
warning est présent avant ce chantier et ne dépend d'aucun de ses changements.

**Hors périmètre :** dépendance amont de GPUI, remontée par la révision Zed
épinglée. Sa résolution passe par un bump de la fork GPUI, ce que le PRD
interdit explicitement (Non-Goal 7).

**Piste :** vérifier au prochain bump de la fork Zed si `block` a été remplacé
par `block2`. `cargo report future-incompatibilities --id 1` donne le détail.

---

## OBS-002 - Prérequis de build macOS non documentés

**Sévérité :** moyenne
**Statut :** OPEN
**Découvert :** 2026-08-15, en tentant le premier build sur une machine neuve

Construire `paneflow-app` sur macOS échoue sur une machine qui n'a que les
Command Line Tools, avec deux erreurs successives sans rapport apparent :

1. `failed to spawn cmake` - `wasmtime-c-api-impl` exige `cmake`, absent par
   défaut. `brew install cmake`.
2. `xcrun: error: unable to find utility "metal"` - `gpui_macos` compile
   `shaders.metal` dans son build script. Les Command Line Tools ne fournissent
   pas le compilateur `metal` ; Xcode complet est requis, et sur Xcode 26+ la
   toolchain Metal est un composant téléchargé séparément.

Aucun des deux n'est mentionné dans `CLAUDE.md` ni dans la documentation
contributeur. Un nouveau contributeur macOS perd du temps sur des messages qui
ne pointent pas vers la cause.

**Hors périmètre :** documentation d'onboarding générale, pas propre à
libghostty. Toucher `CLAUDE.md` pour ça mélangerait des sujets.

**Piste :** une section « Prérequis macOS » dans la doc contributeur, ou un
`scripts/check-macos-prereqs.sh` appelé en amont.

---

## OBS-003 - Zig 0.15.2 ne peut pas construire nativement sur un SDK macOS 26+

**Sévérité :** moyenne
**Statut :** WATCH
**Découvert :** 2026-08-15, pendant US-002

Apple a retiré la tranche `arm64-macos` de `usr/lib/libSystem.tbd` à partir du
SDK macOS 26 ; seule `arm64e-macos` subsiste. Le linker Mach-O de Zig 0.15.2
exige une correspondance exacte, ne trouve rien, et déclare tout libc
indéfini. `MacOSX15.0.sdk` porte encore la tranche.

Conséquence directe : `zig build` échoue sur une machine dont le SDK le plus
récent est ≥ 26, y compris pour `zig build --help`, parce que Zig compile son
propre build runner contre le SDK hôte avant toute étape projet.

Ce n'est pas un défaut Paneflow, mais ça contraint le chantier : le lane CI est
épinglé sur une image macOS 15 et vérifie la présence de la tranche avant de
construire.

**Hors périmètre :** amont (Zig / Apple). Documenté en détail avec le
contournement local dans `tasks/us-002-macos-zig-spike-findings.md`.

**Piste :** si le pin Zig est un jour relevé (hors périmètre de ce PRD),
revérifier si le linker accepte `arm64e`. Cela lèverait la contrainte d'image
CI et le besoin de shim local.
