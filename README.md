# QuestBuddy

A system-tray game progression companion for Hollow Knight and Silksong. Tracks save data, displays stats, suggests next steps, and lets you edit save fields — all from a compact desktop window.

## Features

- **Save Auto-Detection** — reads HK/Silksong saves (AES-256-ECB decryption built in)
- **Progress Overview** — geo, soul, grubs, nail, charms, bosses, dream essence
- **Next Steps** — smart suggestions based on your current save
- **Save Editor** — browse and edit raw `playerData` fields
- **8 Themes** — dark, purple, light, forest, ocean, amber, rose, slate
- **Custom Title Bar** — draggable, with min/max/close to tray
- **Settings** — always-on-top, auto-start, font size, compact mode, section visibility, custom save paths, window memory
- **Tray Integration** — minimizes to system tray, quits from tray menu

## Building

```bash
npm install
npm run build
cargo tauri build
```

Requires Rust and the Tauri CLI.

## License

MIT
