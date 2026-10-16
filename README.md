# CariYuk 🔍

Aplikasi pencarian file super cepat untuk Windows dengan panel quick-access ala Spotlight-nya Mac. Dibuat untuk menyelesaikan masalah: **susah cari file dengan cepat karena folder berantakan**.

Tekan hotkey global → panel muncul → ketik → enter. Selesai.

## Fitur

- ⚡ **Pencarian instan** — fuzzy search nama file (exact > prefix > substring > subsequence) dengan hasil real-time
- 🎯 **Hotkey global** — default `Alt+Space`, bisa diubah sendiri di pengaturan (otomatis fallback jika sudah dipakai app lain)
- 📁 **Index folder penting** — Documents, Downloads, Desktop, Pictures, Videos; bisa ditambah/dikurangi
- 🔄 **Index otomatis** — scan awal saat startup + watcher perubahan file real-time (buat/ubah/hapus/_rename)
- 🏷️ **Filter pencarian** — `ext:pdf`, `folder:skripsi`
- 🖥️ **Jalan di background** — app standby di system tray, panel muncul/hilang via hotkey atau klik tray
- 🌑 **Dark mode** ala Spotlight, auto-hide saat klik di luar panel

## Teknologi

| Layer | Teknologi |
|---|---|
| Frontend | React 18 + TypeScript + Vite |
| Backend | Rust (Tauri 2) |
| Database index | SQLite (rusqlite) |
| File watcher | notify + debouncer |
| Installer | NSIS (Windows) |

## Shortcut

| Tombol | Aksi |
|---|---|
| `Hotkey` | Buka/tutup panel pencarian |
| `Enter` | Buka file dengan app default |
| `Ctrl+Enter` | Buka lokasi folder di Explorer |
| `↑` `↓` | Navigasi hasil |
| `Esc` | Tutup panel |

## Development

```bash
# Prasyarat: Node.js, Rust (rustup), Tauri CLI
npm install

# Jalankan mode development (hot reload)
npm run tauri dev

# Build installer release (output: src-tauri/target/release/bundle/nsis/)
npm run tauri build
```

Data app tersimpan di `%APPDATA%\cariyuk\app\data\` (settings.json + index.db).

## Struktur Project

```
├── src/                    # Frontend React
│   ├── windows/            # Search (panel) & Settings (pengaturan)
│   ├── components/         # SearchBar, ResultList, FileTypeIcon
│   ├── hooks/              # useSearch (debounce + invoke)
│   └── lib/                # API wrapper Tauri commands
├── src-tauri/              # Backend Rust
│   └── src/
│       ├── commands.rs     # Tauri commands (search, settings, dll)
│       ├── indexer.rs      # Scanner + file watcher
│       ├── search.rs       # Fuzzy engine + ranking
│       ├── db.rs           # SQLite layer
│       ├── hotkey.rs       # Global shortcut + fallback
│       └── tray.rs         # System tray
└── tools/                  # Script utilitas (generate icon, dll)
```

## License

MIT
