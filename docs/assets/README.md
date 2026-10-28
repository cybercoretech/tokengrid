# TokenGrid Asset Pack

Every graphic on [cybercoretech.github.io/tokengrid](https://cybercoretech.github.io/tokengrid/)
was drawn for TokenGrid and ships in this folder. MIT licensed, same as the plugin.

```text
assets/
├── pack.json        # manifest — every file, type, size, purpose
├── icons/           # 16 SVG glyphs (64px source, themed, currentColor-safe)
├── png/             # 512px PNG renders of the icons, logo and wordmark
├── svg/             # logo-mark.svg · logo-wordmark.svg
├── cards/           # hero card (1200×630) + 4 feature cards (800×500), SVG + PNG
└── img/             # favicon.svg + social-card.svg/.png (Open Graph 1200×630)
```

## Icon set

| File | Drawn for |
| --- | --- |
| `icon-grid` | the 2×2 token grid / plugin identity |
| `icon-token` | a single counted token |
| `icon-budget` | per-agent budget caps |
| `icon-agent` | any of the 19 CLI agents |
| `icon-bar` | the Omarchy bar widget |
| `icon-terminal` | the Rust TUI |
| `icon-ledger` | the SQLite ledger |
| `icon-window` | the 1100×650 floating window rule |
| `icon-pipeline` | alias → wrapper → daemon → ledger |
| `icon-clock` | wrapper duration measurement |
| `icon-bolt` | live telemetry / `⚡` summary |
| `icon-ghostty` | the Ghostty isolation profile |
| `icon-rule` | the Hyprland Lua window rule |
| `icon-shield` | local-only, nothing leaves the host |
| `icon-download` | this pack |
| `icon-palette` | the 312-theme chooser |

## Rendering

PNGs are rendered from the SVG sources at 2× their display size:

```zsh
rsvg-convert -w 512 icons/icon-grid.svg -o png/icon-grid.png
rsvg-convert -w 1200 -h 630 cards/tokengrid-hero-card.svg -o cards/tokengrid-hero-card.png
```

Regenerate `pack.json` after adding files — it is the manifest the page's
asset section counts from.
