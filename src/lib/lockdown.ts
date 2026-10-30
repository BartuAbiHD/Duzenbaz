// Arayüzün tarayıcı gibi davranmasını engeller. Windows'ta aynı şey WebView2 ayarlarıyla da
// yapılır (src-tauri/src/lockdown.rs); bu katman diğer platformlar içindir.
const BLOCKED_KEYS = new Set(["F3", "F5", "F7", "F11", "F12"]);
const CTRL_BLOCKED = new Set(["p", "s", "u", "f", "g", "r", "o", "+", "-", "=", "0"]);
const CTRL_SHIFT_BLOCKED = new Set(["i", "j", "c", "r"]);

export function lockdown() {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("dragstart", (e) => e.preventDefault());
  window.addEventListener("wheel", (e) => e.ctrlKey && e.preventDefault(), { passive: false });
  window.addEventListener(
    "keydown",
    (e) => {
      const k = e.key.toLowerCase();
      const mod = e.ctrlKey || e.metaKey;
      const blocked =
        BLOCKED_KEYS.has(e.key) ||
        (mod && !e.shiftKey && !e.altKey && CTRL_BLOCKED.has(k)) ||
        (mod && e.shiftKey && CTRL_SHIFT_BLOCKED.has(k)) ||
        (e.metaKey && e.altKey && k === "i") ||
        (e.altKey && (e.key === "ArrowLeft" || e.key === "ArrowRight"));
      if (blocked) {
        e.preventDefault();
        // Tauri'nin geliştirme (debug) derlemelerinde eklediği betik Ctrl+Shift+I'yi `document`
        // üzerinde dinleyip geliştirici araçlarını açar. Olayı burada durdurarak ona ulaşmasını engelleriz.
        e.stopImmediatePropagation();
      }
    },
    true,
  );
}
