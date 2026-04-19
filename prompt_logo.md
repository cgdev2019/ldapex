# Nano Banana prompt — Ldapex app icon

Use the prompt below with **Gemini 2.5 Flash Image (Nano Banana)** to
generate the Ldapex application icon. Generate **one master square
artwork** and derive the other formats from it.

---

## Master icon — prompt

```text
Create a modern, minimalist app icon for "Ldapex", a desktop application
that browses and edits LDAP directories. 1024×1024 pixels, perfectly
square, transparent-safe subject centered in the canvas with clear
safe-zone padding (roughly 12% on every side so the art reads well at
32px).

Style: flat vector, soft 3D depth (subtle inner shadow + single light
from top-left), crisp edges, no gradients noise, no photographic
textures, no text. Inspired by macOS Big Sur / iOS 17 app icon style
but less skeuomorphic — clean and geometric.

Composition: a stylised directory tree represented as three stacked
horizontal "nodes" (rounded rectangles) connected by thin branch lines
on the left, suggesting an LDAP DIT (Directory Information Tree). The
top node is smallest, the middle and bottom are progressively wider.
In the bottom-right corner, overlap a small, confident magnifying
glass that hints at the "search/edit" aspect. The magnifier handle
extends just past the icon's inner safe zone. Keep the whole artwork
readable as a single silhouette at 16px.

Rounded-square background tile: 22% corner radius (typical iOS squircle
proportions), filled with a diagonal gradient from royal blue
#2563EB (top-left) to deep indigo #1E40AF (bottom-right). Add a very
subtle 1-pixel inner light stroke at the top for glass depth.

Foreground color: off-white #F8FAFC for the nodes and branches, with
a single warm amber accent #FBBF24 on the magnifier glass rim to add
focal contrast. The tree branches are 2–3 px strokes, rounded line
caps.

No letters, no numbers, no wordmark. The icon must work equally well
on light and dark OS backgrounds, so keep the central subject
high-contrast against the blue tile.

Export: PNG with transparent background outside the rounded-square
tile.
```

---

## Required output formats

After the master PNG is accepted, request these variants from Nano
Banana (same subject + palette, adjust canvas as noted):

| Format                      | Target platform      | Notes                                                             |
| --------------------------- | -------------------- | ----------------------------------------------------------------- |
| `icon-1024.png` (1024×1024) | source               | Master artwork with transparent corners                           |
| `icon-512.png`   (512×512)  | Linux AppImage, web  | Same art, downscaled                                              |
| `icon-256.png`   (256×256)  | Windows .ico base    | Keep subtle highlights visible                                    |
| `icon-128.png`   (128×128)  | macOS .icns, web     | Simplify inner 1-px strokes                                       |
| `icon-64.png`    (64×64)    | tray / small widgets | Thicken branch strokes to stay legible                            |
| `icon-32.png`    (32×32)    | Windows small icon   | Drop the magnifier handle detail beyond the frame                 |
| `icon-mac.png`   (1024×1024)| macOS icns build     | Add 8% transparent padding so the squircle leaves breathing room  |
| `icon-win.png`   (256×256)  | Windows .ico build   | Hard-cut the background at the squircle — no outer transparency   |
| `favicon.png`    (32×32)    | webview fallback     | Same as icon-32 with a slightly darker background (#1E3A8A)       |

Ask for `transparent background outside the rounded-square tile` in
every variant so the existing Tauri icon pipeline (`cargo tauri icon`)
can pick them up without retouching.

---

## Iteration prompts

If the first draft is off, reuse these targeted nudges:

- "Flatten the magnifier — remove the thick black outline, keep only
  the warm amber rim."
- "The tree nodes should feel like 'cards' stacked in a 3D tilt. Add a
  very subtle downward offset shadow (no more than 2 px) between each
  node, not on the whole icon."
- "Reduce visual noise: no sparkles, no gradient mesh, no particle
  effects. Stay geometric and quiet."
- "Keep the wordmark out — the icon must be purely symbolic."
- "Re-centre the magnifier lower-right, but make sure its inner circle
  does not touch the bottom node."

---

## Drop-in location

When the final `icon-1024.png` is ready, run from the repo root:

```bash
cp /path/to/icon-1024.png /tmp/ldapex-icon/source.png
(cd crates/ldapex-app && cargo tauri icon /tmp/ldapex-icon/source.png)
rm -rf crates/ldapex-app/icons/android crates/ldapex-app/icons/ios
git add crates/ldapex-app/icons && git commit -m "chore(icon): replace placeholder with Nano Banana artwork"
```

That regenerates every size Tauri needs and keeps only the desktop
variants, matching the existing convention in `tauri.conf.json`.
