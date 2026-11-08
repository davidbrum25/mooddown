import { invoke } from "@tauri-apps/api/core";

interface MooddownConfig {
  general: {
    provider: string;
    custom_url: string;
    browser: string;
    hotkey: string;
    autohide_on_blur: boolean;
    pinned: boolean;
    start_at_boot: boolean;
  };
  providers: {
    gemini: string;
    claude: string;
    grok: string;
    chatgpt: string;
  };
  window: {
    width_percent: number;
    height_percent: number;
    animation_duration_ms: number;
    target_screen: string;
  };
}

interface DetectedBrowser {
  name: string;
  path: string;
  is_available: boolean;
}

const isTauriEnv = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function apiGetConfig(): Promise<MooddownConfig> {
  if (isTauriEnv) {
    return await invoke<MooddownConfig>("get_config");
  }
  const res = await fetch("/api/config");
  return await res.json();
}

async function apiGetBrowsers(): Promise<DetectedBrowser[]> {
  if (isTauriEnv) {
    return await invoke<DetectedBrowser[]>("get_detected_browsers");
  }
  const res = await fetch("/api/browsers");
  return await res.json();
}

async function apiSaveConfig(config: MooddownConfig): Promise<void> {
  if (isTauriEnv) {
    await invoke("save_config", { config });
    return;
  }
  const res = await fetch("/api/save", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(config),
  });
  if (!res.ok) {
    throw new Error(await res.text());
  }
}

async function apiToggleDropdown(): Promise<void> {
  if (isTauriEnv) {
    await invoke("toggle_dropdown");
    return;
  }
  await fetch("/api/toggle", { method: "POST" });
}

async function apiOpenProfile(): Promise<void> {
  if (isTauriEnv) {
    await invoke("open_profile_folder");
    return;
  }
  await fetch("/api/open-profile", { method: "POST" });
}

async function apiCloseSettings(): Promise<void> {
  if (isTauriEnv) {
    await invoke("hide_settings_window");
    return;
  }
  await fetch("/api/close", { method: "POST" });
  window.close();
}

let currentConfig: MooddownConfig | null = null;
let isRecordingHotkey = false;
let previousHotkey = "F12";

// Elements
const activeUrlDisplay = document.getElementById("active-url-display") as HTMLElement;
const providerCards = document.querySelectorAll(".provider-card") as NodeListOf<HTMLButtonElement>;
const inputWidth = document.getElementById("input-width") as HTMLInputElement;
const labelWidth = document.getElementById("label-width") as HTMLElement;
const inputHeight = document.getElementById("input-height") as HTMLInputElement;
const labelHeight = document.getElementById("label-height") as HTMLElement;
const inputAnim = document.getElementById("input-anim") as HTMLInputElement;
const labelAnim = document.getElementById("label-anim") as HTMLElement;
const inputHotkey = document.getElementById("input-hotkey") as HTMLInputElement;
const btnRecordHotkey = document.getElementById("btn-record-hotkey") as HTMLButtonElement;
const toggleAutohide = document.getElementById("toggle-autohide") as HTMLInputElement;
const togglePinned = document.getElementById("toggle-pinned") as HTMLInputElement;
const toggleAutostart = document.getElementById("toggle-autostart") as HTMLInputElement;
const detectedBrowserName = document.getElementById("detected-browser-name") as HTMLElement;
const btnOpenProfile = document.getElementById("btn-open-profile") as HTMLButtonElement;
const btnTestToggle = document.getElementById("btn-test-toggle") as HTMLButtonElement;
const btnSave = document.getElementById("btn-save") as HTMLButtonElement;
const btnClose = document.getElementById("btn-close") as HTMLButtonElement;
const saveStatus = document.getElementById("save-status") as HTMLElement;

function selectProviderCard(providerName: string) {
  providerCards.forEach((card) => {
    const p = card.dataset.provider;
    if (p === providerName) {
      card.classList.add("border-sky-500/50", "bg-sky-500/10", "shadow-lg", "shadow-sky-500/10");
      card.classList.remove("border-white/5", "bg-[#161a29]/60");
    } else {
      card.classList.remove("border-sky-500/50", "bg-sky-500/10", "shadow-lg", "shadow-sky-500/10");
      card.classList.add("border-white/5", "bg-[#161a29]/60");
    }
  });

  if (currentConfig) {
    currentConfig.general.provider = providerName;
    const url = (currentConfig.providers as Record<string, string>)[providerName] || "";
    activeUrlDisplay.textContent = url.replace("https://", "");
  }
}

function mapKeyToQtKey(e: KeyboardEvent): string | null {
  const ignoredModifiers = ["Control", "Shift", "Alt", "AltGraph", "Meta", "OS", "Super", "Hyper"];
  if (ignoredModifiers.includes(e.key)) {
    return null;
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(e.key)) {
    return e.key.toUpperCase();
  }

  const code = e.code || "";
  if (code.startsWith("Key") && code.length === 4) {
    return code.slice(3).toUpperCase();
  }
  if (code.startsWith("Digit") && code.length === 6) {
    return code.slice(5);
  }

  const codeMap: Record<string, string> = {
    Space: "Space",
    Enter: "Return",
    NumpadEnter: "Return",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Del",
    Insert: "Ins",
    Home: "Home",
    End: "End",
    PageUp: "PgUp",
    PageDown: "PgDown",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Backquote: "`",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
  };

  if (codeMap[code]) {
    return codeMap[code];
  }

  if (e.key && e.key.length === 1) {
    return e.key.toUpperCase();
  }

  return null;
}

function startRecordingHotkey() {
  if (isRecordingHotkey) return;
  isRecordingHotkey = true;
  previousHotkey = inputHotkey.value || "F12";
  inputHotkey.value = "Presiona teclas...";
  inputHotkey.classList.add("border-sky-400", "bg-sky-500/15", "text-white");
  if (btnRecordHotkey) {
    btnRecordHotkey.textContent = "Cancelar";
  }
  inputHotkey.focus();
}

function stopRecordingHotkey(newHotkey?: string) {
  isRecordingHotkey = false;
  const finalHotkey = newHotkey || previousHotkey;
  inputHotkey.value = finalHotkey;
  if (currentConfig) {
    currentConfig.general.hotkey = finalHotkey;
  }
  inputHotkey.classList.remove("border-sky-400", "bg-sky-500/15", "text-white");
  if (btnRecordHotkey) {
    btnRecordHotkey.textContent = "Grabar";
  }
}

window.addEventListener(
  "keydown",
  (e: KeyboardEvent) => {
    if (!isRecordingHotkey) return;

    e.preventDefault();
    e.stopPropagation();
    e.stopImmediatePropagation();

    if (e.key === "Escape" && !e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
      stopRecordingHotkey(previousHotkey);
      return;
    }

    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    if (e.metaKey) parts.push("Meta");

    const mainKey = mapKeyToQtKey(e);
    if (!mainKey) {
      if (parts.length > 0) {
        inputHotkey.value = `${parts.join("+")}+...`;
      }
      return;
    }

    parts.push(mainKey);
    stopRecordingHotkey(parts.join("+"));
  },
  true
);

async function loadData() {
  try {
    const config = await apiGetConfig();
    currentConfig = config;

    selectProviderCard(config.general.provider);

    inputWidth.value = String(config.window.width_percent);
    labelWidth.textContent = `${config.window.width_percent}%`;

    inputHeight.value = String(config.window.height_percent);
    labelHeight.textContent = `${config.window.height_percent}%`;

    inputAnim.value = String(config.window.animation_duration_ms);
    labelAnim.textContent = `${config.window.animation_duration_ms} ms`;

    inputHotkey.value = config.general.hotkey;
    previousHotkey = config.general.hotkey;
    toggleAutohide.checked = config.general.autohide_on_blur;
    togglePinned.checked = config.general.pinned;
    toggleAutostart.checked = config.general.start_at_boot;

    const browsers = await apiGetBrowsers();
    const available = browsers.filter((b) => b.is_available);
    if (available.length > 0) {
      detectedBrowserName.textContent = `${available[0].name} (${available[0].path})`;
    } else {
      detectedBrowserName.textContent = "No se detectó ningún Chromium (instala Chrome o Brave)";
      detectedBrowserName.classList.add("text-amber-400");
    }
  } catch (err) {
    console.error("Error loading config:", err);
  }
}

// Event Listeners
providerCards.forEach((card) => {
  card.addEventListener("click", () => {
    const p = card.dataset.provider;
    if (p) selectProviderCard(p);
  });
});

inputHotkey.addEventListener("click", () => {
  startRecordingHotkey();
});

if (btnRecordHotkey) {
  btnRecordHotkey.addEventListener("click", () => {
    if (isRecordingHotkey) {
      stopRecordingHotkey(previousHotkey);
    } else {
      startRecordingHotkey();
    }
  });
}

inputWidth.addEventListener("input", () => {
  labelWidth.textContent = `${inputWidth.value}%`;
  if (currentConfig) currentConfig.window.width_percent = parseInt(inputWidth.value, 10);
});

inputHeight.addEventListener("input", () => {
  labelHeight.textContent = `${inputHeight.value}%`;
  if (currentConfig) currentConfig.window.height_percent = parseInt(inputHeight.value, 10);
});

inputAnim.addEventListener("input", () => {
  labelAnim.textContent = `${inputAnim.value} ms`;
  if (currentConfig) currentConfig.window.animation_duration_ms = parseInt(inputAnim.value, 10);
});

toggleAutohide.addEventListener("change", () => {
  if (currentConfig) currentConfig.general.autohide_on_blur = toggleAutohide.checked;
});

togglePinned.addEventListener("change", () => {
  if (currentConfig) currentConfig.general.pinned = togglePinned.checked;
});

toggleAutostart.addEventListener("change", () => {
  if (currentConfig) currentConfig.general.start_at_boot = toggleAutostart.checked;
});

btnOpenProfile.addEventListener("click", async () => {
  try {
    await apiOpenProfile();
  } catch (err) {
    console.error("Failed to open profile folder:", err);
  }
});

btnTestToggle.addEventListener("click", async () => {
  try {
    await apiToggleDropdown();
  } catch (err) {
    console.error("Failed to toggle dropdown:", err);
  }
});

btnSave.addEventListener("click", async () => {
  if (!currentConfig) return;
  try {
    if (isRecordingHotkey) {
      stopRecordingHotkey(previousHotkey);
    }
    btnSave.disabled = true;
    currentConfig.general.hotkey = inputHotkey.value.trim();
    currentConfig.window.width_percent = parseInt(inputWidth.value, 10);
    currentConfig.window.height_percent = parseInt(inputHeight.value, 10);
    currentConfig.window.animation_duration_ms = parseInt(inputAnim.value, 10);
    currentConfig.general.autohide_on_blur = toggleAutohide.checked;
    currentConfig.general.pinned = togglePinned.checked;
    currentConfig.general.start_at_boot = toggleAutostart.checked;

    await apiSaveConfig(currentConfig);

    saveStatus.style.opacity = "1";
    setTimeout(() => {
      saveStatus.style.opacity = "0";
    }, 2000);
  } catch (err) {
    console.error("Error saving config:", err);
    alert(`Error guardando configuración: ${err}`);
  } finally {
    btnSave.disabled = false;
  }
});

btnClose.addEventListener("click", async () => {
  try {
    await apiCloseSettings();
  } catch (err) {
    console.error("Failed to hide window:", err);
  }
});

// Initialize on page load
loadData();
