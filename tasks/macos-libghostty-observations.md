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

## OBS-006 - Le digest de l'image openSUSE Tumbleweed a disparu du registre

**Sévérité :** moyenne (fait échouer `Package smoke (opensuse)` et, par
agrégation, `libghostty validation` sur toute PR)
**Statut :** OPEN
**Découvert :** 2026-08-16, run CI de la PR #1

`.github/workflows/libghostty-linux.yml:370` épingle :

```
registry.opensuse.org/opensuse/tumbleweed:latest@sha256:362c1e2f5a2313e3d1e8713d28e06d13ee7f5a68399f8f86ffa1aa8a2a320c43
```

Le job n'atteint jamais son code : il échoue au `docker pull`, trois fois de
suite après retry.

```
Error response from daemon: manifest unknown
```

Tumbleweed est une rolling release et son registre récupère les anciens
digests. Le digest épinglé n'existe simplement plus en amont. Les quatre
autres distributions du même matrix — arch, debian, fedora, ubuntu — passent,
ce qui confirme que le smoke lui-même va bien.

**Sans lien avec ce chantier :** `git diff main...HEAD` ne touche pas
`libghostty-linux.yml`. Cet échec se produirait sur n'importe quelle PR
ouverte aujourd'hui, et se produira à chaque purge du registre.

**Piste :** re-épingler sur un digest courant. Mais épingler par digest une
image rolling garantit que le problème revient : soit accepter un tag mobile
pour cette entrée précise du matrix (au prix de la reproductibilité), soit
basculer sur une image openSUSE Leap, versionnée et stable, dont les digests
ne sont pas purgés. La seconde option semble préférable — le but du job est de
vérifier l'installation d'un RPM, pas de suivre le dernier Tumbleweed.

---

## OBS-005 - `cargo-deny` échoue sur RUSTSEC-2026-0222 (Wasmtime)

**Sévérité :** haute (bloque le check « Security Audit » sur toute PR)
**Statut :** OPEN
**Découvert :** 2026-08-16, premier run CI de la PR #1

Le job `Security Audit (cargo-deny)` de `run_tests.yml` échoue :

```
error[vulnerability]: Stores can mix up type indices between engines
├ ID: RUSTSEC-2026-0222
├ Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0222
```

Wasmtime est une dépendance transitive de GPUI (host d'extensions). L'avis a
été publié dans la base RustSec après le dernier run vert ; rien dans le code
Paneflow ne l'a déclenché.

**Sans lien avec ce chantier**, et vérifiable : `git diff main...HEAD` ne
touche pas `Cargo.lock` et ne contient aucune occurrence de `wasmtime`. Le
check échouerait à l'identique sur n'importe quelle PR ouverte aujourd'hui.

**Hors périmètre :** corriger revient à relever Wasmtime, qui arrive par la
fork GPUI épinglée. Le PRD interdit explicitement de bumper GPUI (Non-Goal 7),
et le faire à l'intérieur d'un port de plateforme mélangerait deux sujets à
risque très différents.

**Correctif disponible, non appliqué ici.** `cargo update -p wasmtime
--dry-run` propose **36.0.10 → 36.0.13**, un bump de patch dans la même
version mineure, qui ne touche que `Cargo.lock` et laisse la révision GPUI
épinglée intacte. Le Non-Goal 7 du PRD interdit de bumper GPUI, pas de mettre
à jour une entrée transitive du lockfile : la correction est donc légitime,
mais elle relève d'un changement supply-chain à part entière et n'a rien à
faire dans un portage de plateforme.

Avant de l'appliquer, confirmer que 36.0.13 est bien dans la plage corrigée de
l'avis RUSTSEC-2026-0222, puis relancer `cargo deny check`. Deux
avertissements `yanked` sur `spin` traînent dans le même run et méritent le
même passage.

Tant que ce n'est pas fait, le check « Security Audit » reste rouge sur toute
PR, celle-ci comprise.

---

## OBS-004 - Sur POSIX, `shutdown()` d'un enfant vivant ne publie jamais `ChildExited`

**Sévérité :** moyenne
**Statut :** OPEN
**Découvert :** 2026-08-16, en portant la matrice de lifecycle sur macOS (US-007)
**Concerne :** Linux et macOS. Pas Windows.

Dans la boucle du worker Ghostty (`src-app/src/terminal/ghostty_session.rs`,
arm `#[cfg(unix)]` du test `shutdown_sent`) :

```rust
if inner.shutdown_sent.load(Ordering::Acquire) {
    #[cfg(unix)]
    {
        if exit.is_none() {
            terminate_child(child.child_mut(), termination_target);
            child.disarm();
            break;              // sort sans publier ChildExited
        }
    }
    #[cfg(target_os = "windows")]
    {
        shutdown_requested = true;   // -> séquence qui publie l'événement
    }
}
```

Quand un `shutdown()` explicite arrive alors que l'enfant tourne encore, le
chemin POSIX termine bien le processus mais quitte la boucle sans passer par
`publish_child_exit_once`. Le chemin Windows, lui, enclenche une séquence
d'arrêt qui publie l'événement. `ChildExited` est pourtant décrit dans le code
comme « the externally observable teardown barrier ».

Observé en écrivant la matrice de lifecycle macOS : le scénario « shell
bloqué » attend l'événement après `shutdown()` et expire, alors que le
processus est bien terminé. Le test macOS assère donc la terminaison du
processus au lieu de l'événement, avec un commentaire renvoyant ici.

**Hors périmètre :** le comportement est antérieur à ce chantier et identique
sur Linux, où il est livré depuis la promotion du backend. Le corriger
changerait la sémantique d'arrêt d'une plateforme en production, ce qui mérite
sa propre décision plutôt que d'être glissé dans un port.

**Piste :** décider si `ChildExited` doit être publié sur tout arrêt, y compris
demandé, et aligner les deux plateformes. Vérifier au passage les consommateurs
de l'événement (marks, service detector, nettoyage d'état workspace) pour
savoir si l'absence a un effet visible ou reste inerte parce que la vue est
détruite de toute façon.

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
