# Mooddown 🔻
> Dropdown AI Chat Companion para KDE Plasma 6 (Wayland) y Windows.

Mooddown es una aplicación *dropdown* (estilo Quake / Yakuake) siempre disponible mediante un atajo global (por defecto `F12`), diseñada para invocar instantáneamente a tu asistente de IA favorito (**Google Gemini**, **Anthropic Claude**, **xAI Grok** o **ChatGPT**) deslizándose suavemente desde el borde superior de tu pantalla.

---

## 📸 Capturas de Pantalla

### Dropdown AI Chat en Escritorio (KDE Plasma 6 Wayland)
![Mooddown AI Dropdown en acción](docs/screenshots/dropdown.png)

### Ventana de Ajustes (Diseño Antigravity)
<p align="center">
  <img src="docs/screenshots/settings.png" width="520" alt="Mooddown Settings UI" />
</p>

---

## ✨ Características

- 🎯 **Integración Nativa con KDE Plasma 6 (Wayland):** Utiliza un script de KWin 6 dedicado que responde en milisegundos a nivel de compositor.
- 🖱️ **Cursor-Follow Inteligente:** El panel desciende en la pantalla donde se encuentra tu cursor en configuraciones multimonitor.
- ⚡ **Modo Navegador Aislado (`--app`):** Carga las aplicaciones web oficiales completas mediante el navegador Chromium de tu sistema (Chrome, Brave, Chromium, Edge), garantizando soporte total para inicio de sesión en cuentas Google sin bloqueos y sin gastar cientos de MB de RAM innecesarios.
- 🛡️ **Escudo Anti-Atajos de Navegador (CDP + KWin):** Bloquea atajos propios del navegador (`Ctrl+N`, `Ctrl+J`, `Ctrl+H`, `Ctrl+T`, `Ctrl+P`, `Ctrl+S`, `Ctrl+U`, `F12`, etc.) y evita la apertura de ventanas secundarias para ofrecer una experiencia 100% nativa.
- 🔒 **Perfil de Sesión Persistente:** Mantiene tus sesiones y cuentas guardadas en un directorio local dedicado (`~/.local/share/mooddown/profile`), separado de tu navegación diaria.
- 🎨 **Interfaz de Ajustes Estilo Antigravity:** Panel de configuración moderno en modo oscuro profundo, bordes sutiles de 1px y controles deslizantes reactivos.
- 📌 **Auto-Hide, Pin y Autostart:** Oculta automáticamente el chat al hacer clic fuera de él, fíjalo en pantalla (*Pin*) o configúralo para iniciar automáticamente con el sistema.

---

## 🚀 Requisitos

- **Linux:** KDE Plasma 6 (Wayland o X11) con herramientas D-Bus (`qdbus6` preinstalado en KDE).
- **Navegador:** Cualquier navegador basado en Chromium instalado (Google Chrome, Brave, Chromium o Edge).
- **Windows:** Windows 10 u 11 (compatible con API Win32).

---

## 🛠️ Compilación y Uso

```bash
# Instalar dependencias de desarrollo
npm install

# Compilar frontend
npm run build

# Compilar binario de producción (optimizado ~6.8 MB)
cd src-tauri
cargo build --release

# Ejecutar Mooddown
cd ..
./run.sh
```

---

## ⚙️ Configuración (`config.toml`)

Mooddown guarda su configuración en `~/.config/mooddown/config.toml`:

```toml
[general]
provider = "gemini"               # "gemini" | "claude" | "grok" | "chatgpt" | "custom"
custom_url = "https://aistudio.google.com"
browser = "auto"                  # "auto" o comando específico
hotkey = "F12"                    # Atajo global
autohide_on_blur = true           # Ocultar al perder el foco
pinned = false                    # Fijar en pantalla
start_at_boot = true              # Iniciar con el sistema

[window]
width_percent = 65                # Ancho respecto al monitor (%)
height_percent = 55               # Alto respecto al monitor (%)
animation_duration_ms = 180       # Velocidad de deslizamiento
target_screen = "cursor"          # Monitor del cursor
```
