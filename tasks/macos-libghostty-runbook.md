# Runbook - backend libghostty sous macOS

**PRD :** `tasks/prd-macos-libghostty-backend-2026-Q3.md` (EP-005 / US-015)
**Cible :** macOS 13+ sur Apple Silicon (`aarch64-apple-darwin`)

Ce runbook sert à trois choses : choisir le backend, diagnostiquer un échec
sans exposer le contenu du terminal, et revenir à Alacritty. Aucune commande
ici n'affiche de sortie utilisateur, de commande tapée ou de presse-papiers.

---

## 1. État actuel : phase de qualification

Le backend Ghostty est **compilé** dans les builds macOS standards, mais
`auto` sélectionne toujours Alacritty. C'est délibéré : la promotion (US-017)
exige que tous les quality gates soient verts sur le commit candidat.

| Valeur de `terminal.backend` | Effet sur macOS Apple Silicon |
|---|---|
| `auto` (défaut) | Alacritty |
| `ghostty` | Ghostty |
| `alacritty` | Alacritty |

Sur un Mac Intel, le backend Ghostty n'est pas compilé du tout : aucun artefact
reviewé n'existe pour `x86_64-apple-darwin`. `ghostty` y retombe sur Alacritty
avec un diagnostic.

---

## 2. Choisir le backend

`~/.config/paneflow/paneflow.json` :

```json
{
  "terminal": {
    "backend": "ghostty"
  }
}
```

Le changement s'applique aux **nouvelles** sessions. Une session qui a déjà
lancé son processus ne change jamais de backend : c'est une garantie du PRD,
pas une limite d'implémentation.

### Vérifier le backend réellement utilisé

```bash
RUST_LOG=paneflow::terminal::backend=info cargo run
```

Le log indique le backend demandé et le backend effectif. Si les deux diffèrent,
c'est qu'un fallback pré-spawn a eu lieu et la raison est journalisée.

---

## 3. Rollback

Une seule modification de configuration, sans réinstallation ni suppression de
données :

```json
{
  "terminal": {
    "backend": "alacritty"
  }
}
```

Un build de secours sans le backend natif reste possible :

```bash
cargo build --release -p paneflow-app --no-default-features
```

Cette configuration est vérifiée par la CI à chaque run, précisément pour que
le chemin de repli ne pourrisse pas.

---

## 4. Construire l'artefact natif

Cas normal : **rien à faire.** L'archive reviewée est dans le dépôt sous
`native/libghostty/prebuilt/aarch64-apple-darwin/` et Cargo ne télécharge rien.

Pour la reconstruire depuis la source épinglée :

```bash
PANEFLOW_GHOSTTY_SOURCE_DIR=/chemin/vers/ghostty \
  scripts/build-libghostty-macos.sh --verify-reproducible
```

Prérequis : Zig 0.15.2, et un checkout Ghostty **propre** au SHA
`ae52f97dcac558735cfa916ea3965f247e5c6e9e`. Le script refuse tout autre SHA ou
un arbre sale, et vérifie l'empreinte du header avant de construire.

Pour consommer une archive construite localement plutôt que celle du dépôt :

```bash
PANEFLOW_LIBGHOSTTY_DIR="$PWD/target/libghostty/aarch64-apple-darwin" cargo build
```

Dans ce mode, la vérification se fait contre le `build-info.txt` de l'archive
locale et non contre le manifest — c'est réservé au développement.

---

## 5. Diagnostiquer un échec de build

| Symptôme | Cause | Action |
|---|---|---|
| `failed to spawn cmake` | `cmake` absent, exigé transitivement par `wasmtime-c-api-impl` | `brew install cmake` |
| `xcrun: unable to find utility "metal"` | `gpui_macos` compile ses shaders ; les Command Line Tools ne fournissent pas `metal` | Xcode complet, puis `sudo xcode-select -s /Applications/Xcode.app`. Sur Xcode 26+, `xcodebuild -downloadComponent MetalToolchain` |
| `xcrun --show-sdk-path` renvoie un chemin obsolète | Cache `xcrun` périmé après un `xcode-select` | `xcrun --kill-cache` |
| `libghostty input rejected for target …` | Archive absente, corrompue ou métadonnées incohérentes | Le message nomme le fichier attendu et l'action corrective. Ne jamais contourner en désactivant la vérification |
| `libghostty manifest has no archive_sha256_…` | Aucun artefact reviewé pour cette cible | Construire avec le script et pointer `PANEFLOW_LIBGHOSTTY_DIR`, ou restaurer l'artefact du dépôt |
| Tout libc indéfini au link Zig (`_abort`, `_bzero`, …) | SDK macOS 26+ : Apple y a retiré la tranche `arm64-macos` du `.tbd` que le linker de Zig 0.15.2 exige | Voir `tasks/us-002-macos-zig-spike-findings.md`, section « Local workaround ». Ne concerne que la reconstruction de l'archive, jamais un build normal |

---

## 6. Vérifier une installation

```bash
# Le moteur répond : vrai PTY, /bin/sh, feed, resize, snapshot, enfant reapé
cargo run -p paneflow-ghostty-smoke --features native
# attendu : libghostty package smoke passed

# Aucun dylib Ghostty : l'archive doit être liée statiquement
otool -L target/release/paneflow | tail -n +2 | grep -i ghostty
# attendu : aucune sortie
```

---

## 7. Campagnes de lifecycle

Marquées `#[ignore]` : ce sont des gates de promotion, pas des tests de boucle
courte.

```bash
cargo test -p paneflow-app ghostty_spawn_resize_close_stress_has_no_residual_growth -- --ignored
cargo test -p paneflow-app macos_ghostty_lifecycle_scenario_matrix_is_bounded -- --ignored
cargo test -p paneflow-app macos_ghostty_32_pane_resize_and_close_orders_are_bounded -- --ignored
```

En cas d'échec sur `phase=resources`, lire les deux chiffres avant de conclure
à une fuite :

- `handles_start` vs `handles_end` — une divergence est une **vraie** fuite de
  descripteurs, sans ambiguïté.
- `rss_start` vs `rss_end` — l'allocateur Darwin ne rend pas ses pages au
  noyau. Une croissance dont les incréments **décroissent** d'une campagne à
  l'autre est un plateau ; une croissance à incrément **constant** est une
  fuite. Les campagnes chauffent jusqu'à stabilisation avant de mesurer,
  précisément pour que ce signal reste lisible.

---

## 8. Limites connues

- Apple Silicon uniquement. Aucun artefact `x86_64-apple-darwin` n'est publié.
- macOS 13 est le plancher, épinglé dans le triple Zig
  (`macos_deployment_target` au manifest) et non hérité de la machine de build.
- Sur POSIX, un `shutdown()` explicite d'un enfant vivant ne publie pas
  `ChildExited` — voir OBS-004 dans `tasks/macos-libghostty-observations.md`.
  Comportement antérieur à ce chantier, partagé avec Linux.
- L'empreinte de l'archive au manifest est marquée `PROVISIONAL` tant qu'un run
  de `libghostty-macos.yml` ne l'a pas re-dérivée.
