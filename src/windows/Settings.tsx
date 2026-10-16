import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  AppSettings,
  DEFAULT_SETTINGS,
  loadSettings,
  saveSettings,
  pickFolder,
  reindexNow,
  getIndexStats,
  IndexStats,
} from "../lib/api";

/** Keyboard combo → tauri global-shortcut syntax ("Ctrl+Alt+K"). */
function keyComboToString(e: KeyboardEvent): string | null {
  // Ignore modifier-only presses — they are not valid hotkeys.
  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return null;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");

  let key = e.key;
  if (key === " ") key = "Space";
  else if (key.length === 1) key = key.toUpperCase();
  else if (key === "ArrowUp") key = "Up";
  else if (key === "ArrowDown") key = "Down";
  else if (key === "ArrowLeft") key = "Left";
  else if (key === "ArrowRight") key = "Right";

  if (parts.length === 0) return null; // require at least one modifier

  parts.push(key);
  return parts.join("+");
}

function HotkeyRecorder({
  hotkey,
  onChange,
}: {
  hotkey: string;
  onChange: (v: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!recording) return;
    const el = ref.current;
    if (!el) return;
    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.key === "Escape") {
        setRecording(false);
        return;
      }
      const combo = keyComboToString(e);
      if (combo) onChange(combo);
      if (combo || e.key === "Escape") setRecording(false);
    };
    window.addEventListener("keydown", handler, true);
    return () => window.removeEventListener("keydown", handler, true);
  }, [recording, onChange]);

  return (
    <div
      ref={ref}
      className={`hotkey-box ${recording ? "recording" : ""}`}
      onClick={() => setRecording((r) => !r)}
      title="Klik lalu tekan kombinasi keyboard"
    >
      {recording ? (
        <span className="recording-label">Tekan kombinasi… (Esc untuk batal)</span>
      ) : (
        <span>{hotkey || "Tidak diatur"}</span>
      )}
    </div>
  );
}

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [status, setStatus] = useState<{ text: string; error?: boolean } | null>(null);
  const [stats, setStats] = useState<IndexStats>({ count: 0, indexing: false });
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    loadSettings().then((s) => {
      if (!disposed) setSettings(s);
    });

    const win = getCurrentWindow();
    win
      .onFocusChanged(({ payload: focused }) => {
        if (focused && !disposed) getIndexStats().then(setStats);
      })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const id = setInterval(() => getIndexStats().then(setStats), 3000);
    return () => clearInterval(id);
  }, []);

  async function handleSave() {
    setBusy(true);
    setStatus(null);
    try {
      await saveSettings(settings);
      setStatus({ text: "Tersimpan!" });
      setTimeout(() => setStatus(null), 2000);
    } catch (err) {
      setStatus({ text: `Gagal: ${err instanceof Error ? err.message : String(err)}`, error: true });
    } finally {
      setBusy(false);
    }
  }

  async function handleAddFolder() {
    const folder = await pickFolder();
    if (folder && !settings.indexedFolders.includes(folder)) {
      setSettings((s) => ({ ...s, indexedFolders: [...s.indexedFolders, folder] }));
    }
  }

  function handleRemoveFolder(path: string) {
    setSettings((s) => ({
      ...s,
      indexedFolders: s.indexedFolders.filter((f) => f !== path),
    }));
  }

  return (
    <main className="settings-panel">
      <h1>Pengaturan CariYuk</h1>

      <section>
        <h2>Hotkey Global</h2>
        <p className="hint">Tekan tombol ini di mana saja untuk membuka panel pencarian.</p>
        <div className="hotkey-row">
          <HotkeyRecorder
            hotkey={settings.hotkey}
            onChange={(v) => setSettings((s) => ({ ...s, hotkey: v }))}
          />
          <button className="btn-secondary" onClick={() => setSettings((s) => ({ ...s, hotkey: "Alt+Space" }))}>
            Reset
          </button>
        </div>
      </section>

      <section>
        <h2>Folder yang Di-index</h2>
        {settings.indexedFolders.length === 0 ? (
          <p className="hint">Belum ada folder. Tambahkan di bawah.</p>
        ) : (
          <ul className="folder-list">
            {settings.indexedFolders.map((f) => (
              <li key={f}>
                <span className="folder-path" title={f}>{f}</span>
                <button
                  className="btn-icon"
                  title="Hapus folder"
                  onClick={() => handleRemoveFolder(f)}
                >
                  ✕
                </button>
              </li>
            ))}
          </ul>
        )}
        <button className="btn-secondary" onClick={handleAddFolder}>
          + Tambah Folder
        </button>
      </section>

      <section>
        <h2>Statistik Index</h2>
        <div className="stats-row">
          <div className="stat">
            <span className="stat-value">{stats.indexing ? "…" : stats.count.toLocaleString("id-ID")}</span>
            <span className="stat-label">file ter-index</span>
          </div>
          <div className="stat">
            <span className="stat-value">{settings.indexedFolders.length}</span>
            <span className="stat-label">folder</span>
          </div>
        </div>
        <button
          className="btn-secondary"
          onClick={() => {
            reindexNow();
            setStatus({ text: "Re-index berjalan di background…" });
            setTimeout(() => setStatus(null), 2500);
          }}
        >
          ↻ Re-index Sekarang
        </button>
      </section>

      <section>
        <h2>Startup</h2>
        <label className="toggle-row">
          <input
            type="checkbox"
            checked={settings.autostart}
            onChange={(e) => setSettings((s) => ({ ...s, autostart: e.target.checked }))}
          />
          <span>Jalankan CariYuk saat Windows mulai</span>
        </label>
      </section>

      <div className="actions">
        <button
          disabled={busy}
          onClick={handleSave}
        >
          {busy ? "Menyimpan…" : "Simpan"}
        </button>
        {status && <span className={`status ${status.error ? "error" : ""}`}>{status.text}</span>}
      </div>
    </main>
  );
}
