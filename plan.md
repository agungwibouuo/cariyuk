# CariYuk — Rencana Project

File search app untuk Windows dengan quick-access panel ala Spotlight-nya Mac.
Dibuat untuk menyelesaikan problem: **susah cari file dengan cepat karena folder berantakan**.

---

## 1. Ringkasan Keputusan

| Aspek | Keputusan |
|---|---|
| Hotkey global | Configurable di settings, default **`Alt+Space`** |
| Folder default index | Documents, Downloads, Desktop, Pictures, Videos (bisa diatur di settings) |
| Lingkup search | **Nama file saja** (+ path & ekstensi), tanpa isi file |
| Tech stack | **Tauri 2 + React + TypeScript**, backend logic di Rust |
| Database | SQLite (`rusqlite`) |
| Target platform | Windows (utama) |

## 2. Konsep UX

```
[Alt+Space] → panel floating muncul di tengah layar
    │
    ├─ ketik kosong  → tampilkan file terbaru (recent)
    ├─ ketik keyword → hasil fuzzy search real-time (<100ms)
    ├─ Enter         → buka file dengan app default
    ├─ Ctrl+Enter    → buka lokasi folder di Explorer
    ├─ ↑ / ↓         → navigasi hasil
    └─ Esc           → tutup panel
```

- App jalan di **system tray** (background), panel di-show/hide via hotkey
- Window panel: frameless, transparent, always-on-top, auto-focus ke input
- Window terpisah untuk **Settings**

## 3. Arsitektur

### Frontend (React + TS)

| Bagian | Tanggung Jawab |
|---|---|
| `Search.tsx` | Panel quick-access: input, hasil, recent files, keyboard nav |
| `Settings.tsx` | Kelola folder ter-index, ubah hotkey, toggle auto-start |
| `ResultList.tsx` | Render hasil ter-rank + highlight match |
| `api.ts` | Wrapper `invoke()` ke command Rust |

### Backend (Rust)

| Modul | Tanggung Jawab |
|---|---|
| `db.rs` | SQLite schema & queries (`files` table: path, name, ext, parent, size, modified) |
| `indexer.rs` | Initial scan folder + file watcher (`notify`) untuk incremental update |
| `search.rs` | Fuzzy matching + ranking: exact prefix > substring > sub-sequence |
| `hotkey.rs` | Global shortcut show/hide panel (`tauri-plugin-global-shortcut`) |
| `settings.rs` | Persistensi config (folder list, hotkey, auto-start) |
| `tray` | System tray icon + menu (via Tauri tray API) |

### Plugin Tauri yang dipakai

- `tauri-plugin-global-shortcut` — hotkey global
- `tauri-plugin-opener` — buka file / reveal di Explorer
- `tauri-plugin-autostart` — launch saat startup (opsional)
- `tauri-plugin-sql` atau `rusqlite` langsung di Rust — pilih `rusqlite` (lebih terkontrol)

## 4. Fitur Detail

### MVP (fase awal)
- [x] Panel search via hotkey global
- [x] Index folder default saat first run
- [x] Fuzzy search nama file, real-time
- [x] Ranking hasil (relevansi + yang sering dibuka)
- [x] Recent files saat input kosong
- [x] Buka file (Enter) & reveal di Explorer (Ctrl+Enter)
- [x] File watcher incremental update
- [x] Settings: kelola folder, ubah hotkey

### Filter pencarian (fase 2)
- [x] `ext:pdf` — filter ekstensi
- [x] `folder:skripsi` — filter nama folder
- [ ] `size:>10mb` — filter ukuran *(belum diimplementasi)*
- [ ] `modified:today|week|month` — filter waktu modifikasi *(belum diimplementasi)*

### Nice-to-have (fase 3)
- Pinned/favorite files
- Statistik file yang paling sering dibuka → boost ranking
- Multiple keyword ("skripsi draft" → kedua kata harus match)
- Theme (dark/light)
- Auto-start saat boot Windows
- Pencarian isi file teks (opsional, berat)

## 5. Struktur Folder

```
cariyuk/
├── plan.md                  # dokumen ini
├── package.json
├── src/                     # React frontend
│   ├── main.tsx
│   ├── windows/
│   │   ├── Search.tsx       # panel quick-access
│   │   └── Settings.tsx     # halaman pengaturan
│   ├── components/
│   │   ├── SearchBar.tsx
│   │   └── ResultList.tsx
│   ├── hooks/
│   │   └── useSearch.ts     # debounce + invoke search
│   └── lib/
│       └── api.ts           # wrapper invoke() ke Rust
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── icons/
    └── src/
        ├── main.rs          # entry, setup app
        ├── db.rs            # SQLite schema & queries
        ├── indexer.rs       # scan + watcher
        ├── search.rs        # fuzzy ranking
        ├── hotkey.rs        # global shortcut
        └── settings.rs      # config persistence
```

## 6. Schema Database

```sql
CREATE TABLE files (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    path      TEXT UNIQUE NOT NULL,   -- full path
    name      TEXT NOT NULL,          -- nama file + ekstensi
    ext       TEXT,                   -- ekstensi lowercase tanpa titik
    parent    TEXT NOT NULL,          -- path folder induk
    size      INTEGER NOT NULL,
    modified  INTEGER NOT NULL,       -- unix timestamp
    open_count INTEGER DEFAULT 0      -- untuk ranking
);
CREATE INDEX idx_files_name ON files(name);
CREATE INDEX idx_files_ext  ON files(ext);
CREATE INDEX idx_files_modified ON files(modified DESC);
```

## 7. Fase Pengerjaan

| Fase | Deliverable |
|---|---|
| 1. Scaffold | Init Tauri 2 + React/TS, 2 window (search + settings), tray icon |
| 2. DB & Indexer | Schema SQLite, initial scan folder default, file watcher |
| 3. Search | Fuzzy ranking + integrasi ke UI panel |
| 4. UI Panel | Layout floating, keyboard nav, recent files, highlight match |
| 5. Hotkey & Tray | Global shortcut, show/hide logic, tray menu |
| 6. Settings | Kelola folder, ubah hotkey, auto-start |
| 7. Polish | Icon app, build installer (.msi/.exe), testing |

## 8. Catatan Teknis

- Debounce input search ~50-100ms agar tidak spam query
- Pencarian query di Rust (bukan JS) supaya cepat — frontend hanya render
- Watcher perlu handle: file dibuat, dihapus, direname, dimodifikasi
- Path disimpan lowercase di kolom terpisah jika perlu (case-insensitive search di Windows)
- App harus tetap jalan saat panel ditutup (hide, bukan exit)
