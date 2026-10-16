const EXT_CATEGORIES: Record<string, { label: string; className: string }> = {
  // Documents
  pdf: { label: "PDF", className: "ft-doc-pdf" },
  doc: { label: "DOC", className: "ft-doc" },
  docx: { label: "DOC", className: "ft-doc" },
  odt: { label: "DOC", className: "ft-doc" },
  txt: { label: "TXT", className: "ft-doc" },
  md: { label: "MD", className: "ft-doc" },
  rtf: { label: "RTF", className: "ft-doc" },
  // Spreadsheets
  xls: { label: "XLS", className: "ft-sheet" },
  xlsx: { label: "XLS", className: "ft-sheet" },
  csv: { label: "CSV", className: "ft-sheet" },
  ods: { label: "ODS", className: "ft-sheet" },
  // Slides
  ppt: { label: "PPT", className: "ft-slide" },
  pptx: { label: "PPT", className: "ft-slide" },
  odp: { label: "PPT", className: "ft-slide" },
  // Images
  png: { label: "IMG", className: "ft-image" },
  jpg: { label: "IMG", className: "ft-image" },
  jpeg: { label: "IMG", className: "ft-image" },
  gif: { label: "IMG", className: "ft-image" },
  webp: { label: "IMG", className: "ft-image" },
  svg: { label: "IMG", className: "ft-image" },
  bmp: { label: "IMG", className: "ft-image" },
  psd: { label: "PSD", className: "ft-image" },
  // Audio
  mp3: { label: "MP3", className: "ft-audio" },
  wav: { label: "WAV", className: "ft-audio" },
  flac: { label: "FLAC", className: "ft-audio" },
  ogg: { label: "OGG", className: "ft-audio" },
  m4a: { label: "M4A", className: "ft-audio" },
  // Video
  mp4: { label: "MP4", className: "ft-video" },
  mkv: { label: "MKV", className: "ft-video" },
  avi: { label: "AVI", className: "ft-video" },
  mov: { label: "MOV", className: "ft-video" },
  webm: { label: "WEBM", className: "ft-video" },
  // Archives
  zip: { label: "ZIP", className: "ft-archive" },
  rar: { label: "RAR", className: "ft-archive" },
  "7z": { label: "7Z", className: "ft-archive" },
  tar: { label: "TAR", className: "ft-archive" },
  gz: { label: "GZ", className: "ft-archive" },
  // Code
  js: { label: "JS", className: "ft-code" },
  ts: { label: "TS", className: "ft-code" },
  tsx: { label: "TSX", className: "ft-code" },
  jsx: { label: "JSX", className: "ft-code" },
  json: { label: "JSON", className: "ft-code" },
  html: { label: "HTML", className: "ft-code" },
  css: { label: "CSS", className: "ft-code" },
  py: { label: "PY", className: "ft-code" },
  rs: { label: "RS", className: "ft-code" },
  java: { label: "JAVA", className: "ft-code" },
  php: { label: "PHP", className: "ft-code" },
  // Executables
  exe: { label: "EXE", className: "ft-exe" },
  msi: { label: "MSI", className: "ft-exe" },
  bat: { label: "BAT", className: "ft-exe" },
  apk: { label: "APK", className: "ft-exe" },
};

export default function FileTypeIcon({ ext }: { ext: string | null }) {
  if (!ext) {
    return <span className="ft-badge ft-unknown">•</span>;
  }
  const category = EXT_CATEGORIES[ext.toLowerCase()];
  if (!category) {
    return <span className="ft-badge ft-unknown">{ext.slice(0, 4).toUpperCase()}</span>;
  }
  return <span className={`ft-badge ${category.className}`}>{category.label}</span>;
}
