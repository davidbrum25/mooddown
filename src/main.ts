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

let currentConfig: MooddownConfig | null = null;

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

async function loadData() {
  try {
    const config = await invoke<MooddownConfig>("get_config");
    currentConfig = config;

    // Apply values to UI
    selectProviderCard(config.general.provider);

    inputWidth.value = String(config.window.width_percent);
    labelWidth.textContent = `${config.window.width_percent}%`;

    inputHeight.value = String(config.window.height_percent);
    labelHeight.textContent = `${config.window.height_percent}%`;

    inputAnim.value = String(config.window.animation_duration_ms);
    labelAnim.textContent = `${config.window.animation_duration_ms} ms`;

    inputHotkey.value = config.general.hotkey;
    toggleAutohide.checked = config.general.autohide_on_blur;
    togglePinned.checked = config.general.pinned;
    toggleAutostart.checked = config.general.start_at_boot;

    // Check detected browsers
    const browsers = await invoke<DetectedBrowser[]>("get_detected_browsers");
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

inputHotkey.addEventListener("change", () => {
  if (currentConfig) currentConfig.general.hotkey = inputHotkey.value.trim();
});

btnOpenProfile.addEventListener("click", async () => {
  try {
    await invoke("open_profile_folder");
  } catch (err) {
    console.error("Failed to open profile folder:", err);
  }
});

btnTestToggle.addEventListener("click", async () => {
  try {
    await invoke("toggle_dropdown");
  } catch (err) {
    console.error("Failed to toggle dropdown:", err);
  }
});

btnSave.addEventListener("click", async () => {
  if (!currentConfig) return;
  try {
    btnSave.disabled = true;
    currentConfig.general.hotkey = inputHotkey.value.trim();
    currentConfig.window.width_percent = parseInt(inputWidth.value, 10);
    currentConfig.window.height_percent = parseInt(inputHeight.value, 10);
    currentConfig.window.animation_duration_ms = parseInt(inputAnim.value, 10);
    currentConfig.general.autohide_on_blur = toggleAutohide.checked;
    currentConfig.general.pinned = togglePinned.checked;
    currentConfig.general.start_at_boot = toggleAutostart.checked;

    await invoke("save_config", { config: currentConfig });

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
    await invoke("hide_settings_window");
  } catch (err) {
    console.error("Failed to hide window:", err);
  }
});

// Initialize on page load
loadData();
