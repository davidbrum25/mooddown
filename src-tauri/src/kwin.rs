use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::config::MooddownConfig;

pub struct KWinController;

impl KWinController {
    pub const SCRIPT_NAME: &'static str = "mooddown";

    pub fn script_dir() -> PathBuf {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join("kwin").join("scripts").join("mooddown");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    pub fn normalize_hotkey(hotkey: &str) -> String {
        let trimmed = hotkey.trim();
        if trimmed.is_empty() {
            return "F12".to_string();
        }
        trimmed
            .replace("Super+", "Meta+")
            .replace("super+", "Meta+")
            .replace("Win+", "Meta+")
            .replace("win+", "Meta+")
    }

    pub fn shortcut_id_for_hotkey(hotkey: &str) -> String {
        let normalized = Self::normalize_hotkey(hotkey);
        let sanitized: String = normalized
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        format!("MooddownToggle_{}", sanitized)
    }

    pub fn cleanup_old_shortcuts() {
        if let Ok(output) = Command::new("qdbus6")
            .args([
                "org.kde.kglobalaccel",
                "/component/kwin",
                "org.kde.kglobalaccel.Component.shortcutNames",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let name = line.trim();
                    if name.starts_with("MooddownToggle") {
                        let _ = Command::new("qdbus6")
                            .args([
                                "org.kde.kglobalaccel",
                                "/kglobalaccel",
                                "org.kde.KGlobalAccel.unregister",
                                "kwin",
                                name,
                            ])
                            .output();
                    }
                }
            }
        }
    }

    pub fn generate_script_content(config: &MooddownConfig) -> String {
        let width_pct = config.window.width_percent as f64 / 100.0;
        let height_pct = config.window.height_percent as f64 / 100.0;
        let anim_duration = config.window.animation_duration_ms;
        let autohide = config.general.autohide_on_blur;
        let pinned = config.general.pinned;
        let hotkey = Self::normalize_hotkey(&config.general.hotkey);
        let shortcut_id = Self::shortcut_id_for_hotkey(&config.general.hotkey);

        format!(r#"
// Mooddown KWin 6 Dropdown Controller Script
(function() {{
    var widthPct = {width_pct};
    var heightPct = {height_pct};
    var animDuration = {anim_duration};
    var autoHideOnBlur = {autohide};
    var isPinned = {pinned};
    var isAnimating = false;
    var targetWindow = null;
    var mainChatWindow = null;
    var currentStep = 0;
    var totalSteps = Math.max(6, Math.round(animDuration / 16));
    var slideTimer = new QTimer();
    slideTimer.interval = 16;
    slideTimer.singleShot = false;

    var startY = 0;
    var targetY = 0;
    var currentX = 0;
    var currentW = 0;
    var currentH = 0;
    var isSlidingDown = true;
    var lastShowTime = 0;
    var lastAutoHideTime = 0;

    function isMooddownChatWindow(w) {{
        if (!w || !w.normalWindow) return false;
        var resClass = (w.resourceClass || "").toLowerCase();
        var resName = (w.resourceName || "").toLowerCase();
        var dFile = (w.desktopFileName || "").toLowerCase();
        var caption = (w.caption || "").toLowerCase();

        // Never match the Settings window itself
        if (resClass === "mooddown" ||
            resName === "mooddown" ||
            resClass.indexOf("mooddown-settings") >= 0 ||
            dFile.indexOf("mooddown-settings") >= 0 ||
            resClass.indexOf("127.0.0.1") >= 0 ||
            dFile.indexOf("127.0.0.1") >= 0 ||
            caption.indexOf("mooddown settings") >= 0) {{
            return false;
        }}

        if (resClass.indexOf("mooddown-app") >= 0 ||
            dFile.indexOf("mooddown-app") >= 0 ||
            resClass.indexOf("chrome-gemini") >= 0 ||
            resClass.indexOf("chrome-claude") >= 0 ||
            resClass.indexOf("chrome-grok") >= 0 ||
            resClass.indexOf("chrome-chatgpt") >= 0 ||
            resClass.indexOf("chrome-aistudio") >= 0 ||
            dFile.indexOf("chrome-gemini") >= 0 ||
            dFile.indexOf("chrome-claude") >= 0 ||
            dFile.indexOf("chrome-grok") >= 0 ||
            dFile.indexOf("chrome-chatgpt") >= 0 ||
            dFile.indexOf("chrome-aistudio") >= 0) {{
            return true;
        }}
        return false;
    }}

    function findMooddownWindow() {{
        if (mainChatWindow) {{
            var stillExists = false;
            var all = workspace.windowList();
            for (var j = 0; j < all.length; j++) {{
                if (all[j] === mainChatWindow) {{
                    stillExists = true;
                    break;
                }}
            }}
            if (stillExists) return mainChatWindow;
            mainChatWindow = null;
        }}

        var windows = workspace.windowList();
        for (var i = 0; i < windows.length; i++) {{
            var w = windows[i];
            if (isMooddownChatWindow(w)) {{
                mainChatWindow = w;
                return w;
            }}
        }}
        return null;
    }}

    function applyWindowProperties(w) {{
        w.keepAbove = true;
        w.skipTaskbar = true;
        w.skipPager = true;
        w.skipSwitcher = true;
        w.onAllDesktops = true;
        w.noBorder = true;
    }}

    function getTargetScreenArea() {{
        var pos = workspace.cursorPos;
        var screen = workspace.screenAt ? workspace.screenAt(pos) : workspace.activeScreen;
        var clientArea = workspace.clientArea ? workspace.clientArea(0, screen, workspace.currentDesktop) : screen.geometry;
        return clientArea;
    }}

    slideTimer.timeout.connect(function() {{
        try {{
            if (!targetWindow) {{
                slideTimer.stop();
                isAnimating = false;
                return;
            }}

            currentStep++;
            var progress = Math.min(1.0, currentStep / totalSteps);
            var eased = 1 - Math.pow(1 - progress, 3);
            var newY = Math.round(startY + (targetY - startY) * eased);

            targetWindow.frameGeometry = {{
                x: currentX,
                y: newY,
                width: currentW,
                height: currentH
            }};

            if (progress >= 1.0) {{
                slideTimer.stop();
                isAnimating = false;
                if (!isSlidingDown) {{
                    targetWindow.minimized = true;
                }} else {{
                    workspace.activeWindow = targetWindow;
                }}
            }}
        }} catch (err) {{
            console.log("[Mooddown] Animation error: " + err);
            slideTimer.stop();
            isAnimating = false;
        }}
    }});

    function slideDown(w) {{
        if (!w) return;
        lastShowTime = Date.now();
        var area = getTargetScreenArea();
        currentW = Math.round(area.width * widthPct);
        currentH = Math.round(area.height * heightPct);
        currentX = Math.round(area.x + (area.width - currentW) / 2);
        targetY = Math.round(area.y);
        startY = Math.round(area.y - currentH);

        applyWindowProperties(w);
        w.minimized = false;
        w.frameGeometry = {{
            x: currentX,
            y: startY,
            width: currentW,
            height: currentH
        }};
        workspace.activeWindow = w;

        targetWindow = w;
        isSlidingDown = true;
        currentStep = 0;
        isAnimating = true;
        slideTimer.start();
    }}

    function slideUp(w) {{
        if (!w) return;
        var area = getTargetScreenArea();
        currentX = Math.round(w.frameGeometry.x);
        currentW = Math.round(w.frameGeometry.width);
        currentH = Math.round(w.frameGeometry.height);
        startY = Math.round(w.frameGeometry.y);
        targetY = Math.round(area.y - currentH);

        targetWindow = w;
        isSlidingDown = false;
        currentStep = 0;
        isAnimating = true;
        slideTimer.start();
    }}

    function toggleMooddown() {{
        if (isAnimating) return;

        // Avoid bounce if auto-hide just closed the window from a tray click
        if (Date.now() - lastAutoHideTime < 350) return;

        var w = findMooddownWindow();
        if (!w) {{
            console.log("[Mooddown] Chat window not found. Requesting daemon to launch it via D-Bus...");
            callDBus("com.mooddown.Daemon", "/Mooddown", "com.mooddown.Daemon", "LaunchBrowser");
            return;
        }}

        if (w.minimized) {{
            slideDown(w);
        }} else {{
            slideUp(w);
        }}
    }}

    var saveTimer = new QTimer();
    saveTimer.interval = 800;
    saveTimer.singleShot = true;
    saveTimer.timeout.connect(function() {{
        var wPct = Math.round(widthPct * 100);
        var hPct = Math.round(heightPct * 100);
        if (wPct >= 20 && hPct >= 20) {{
            console.log("[Mooddown] Persisting user resized window geometry: " + wPct + "% x " + hPct + "%");
            callDBus("com.mooddown.Daemon", "/Mooddown", "com.mooddown.Daemon", "SaveWindowSize", wPct, hPct);
        }}
    }});

    function setupWindowListeners(w) {{
        if (!w || w.__mooddownListenersInstalled) return;
        w.__mooddownListenersInstalled = true;

        w.frameGeometryChanged.connect(function() {{
            if (isAnimating || w.minimized || !isSlidingDown) return;
            var fg = w.frameGeometry;
            if (!fg || fg.width <= 250 || fg.height <= 200) return;

            var area = getTargetScreenArea();
            // Only update geometry when the window is resting at the top of the screen (not offscreen or transitioning)
            if (Math.abs(fg.y - area.y) > 12) return;

            var newW = Math.round(fg.width);
            var newH = Math.round(fg.height);

            currentW = newW;
            currentH = newH;
            widthPct = Math.min(1.0, Math.max(0.2, newW / area.width));
            heightPct = Math.min(1.0, Math.max(0.2, newH / area.height));

            // Keep horizontally centered if resized from bottom
            currentX = Math.round(area.x + (area.width - currentW) / 2);

            saveTimer.start();
        }});
    }}

    // Style any existing window if script was reloaded while window was open
    var existing = findMooddownWindow();
    if (existing) {{
        applyWindowProperties(existing);
        setupWindowListeners(existing);
    }}

    // Automatically style and slide down the chat window as soon as Chrome creates it,
    // and block any secondary Chrome windows (e.g. Ctrl+N, Ctrl+Shift+N, Downloads popup) from the same PID
    workspace.windowAdded.connect(function(w) {{
        if (!w) return;
        var mainWin = findMooddownWindow();

        // If a main chat window already exists and the same browser process tries to spawn another window, close it immediately
        if (mainWin && w !== mainWin && w.pid && mainWin.pid && w.pid === mainWin.pid) {{
            console.log("[Mooddown] Blocking secondary browser window from Mooddown PID: " + w.caption);
            lastShowTime = Date.now();
            w.closeWindow();
            workspace.activeWindow = mainWin;
            return;
        }}

        if (isMooddownChatWindow(w)) {{
            mainChatWindow = w;
            console.log("[Mooddown] Detected new AI chat window: " + w.resourceClass);
            setupWindowListeners(w);
            slideDown(w);
        }}
    }});

    // Auto-hide when focus is lost
    workspace.windowActivated.connect(function(client) {{
        if (!client) return;
        var w = findMooddownWindow();

        // If focus shifted to a stray secondary window from the same browser PID, close it and keep main window active
        if (w && client !== w && client.pid && w.pid && client.pid === w.pid) {{
            lastShowTime = Date.now();
            client.closeWindow();
            workspace.activeWindow = w;
            return;
        }}

        if (!autoHideOnBlur || isPinned || isAnimating) return;
        if (Date.now() - lastShowTime < 400) return;
        if (w && !w.minimized && client !== w) {{
            lastAutoHideTime = Date.now();
            slideUp(w);
        }}
    }});

    // Register global shortcut with KWin / kglobalaccel
    registerShortcut("{shortcut_id}", "Toggle Mooddown AI Dropdown ({hotkey})", "{hotkey}", function() {{
        toggleMooddown();
    }});

    console.log("[Mooddown] KWin 6 Dropdown script initialized successfully!");
}})();
"#,
            width_pct = width_pct,
            height_pct = height_pct,
            anim_duration = anim_duration,
            autohide = autohide,
            pinned = pinned,
            hotkey = hotkey,
            shortcut_id = shortcut_id
        )
    }

    pub fn install_and_reload(config: &MooddownConfig) -> Result<(), String> {
        let script_file = Self::script_dir().join("code.js");
        let content = Self::generate_script_content(config);
        fs::write(&script_file, content).map_err(|e| format!("Failed to write KWin script: {}", e))?;

        let script_path = script_file.to_string_lossy().to_string();

        // Unload existing script and purge stale kglobalaccel shortcut bindings
        let _ = Self::unload();
        Self::cleanup_old_shortcuts();

        // Load script
        let load_output = Command::new("qdbus6")
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.loadScript", &script_path, Self::SCRIPT_NAME])
            .output()
            .map_err(|e| format!("Failed to execute qdbus6: {}", e))?;

        if !load_output.status.success() {
            return Err(String::from_utf8_lossy(&load_output.stderr).to_string());
        }

        // Start scripting
        let _ = Command::new("qdbus6")
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.start"])
            .output();

        Ok(())
    }

    pub fn unload() -> Result<(), String> {
        let _ = Command::new("qdbus6")
            .args(["org.kde.KWin", "/Scripting", "org.kde.kwin.Scripting.unloadScript", Self::SCRIPT_NAME])
            .output();
        Ok(())
    }

    pub fn invoke_toggle() -> Result<(), String> {
        let config = MooddownConfig::load();
        let shortcut_id = Self::shortcut_id_for_hotkey(&config.general.hotkey);
        let output = Command::new("qdbus6")
            .args([
                "org.kde.kglobalaccel",
                "/component/kwin",
                "org.kde.kglobalaccel.Component.invokeShortcut",
                &shortcut_id,
            ])
            .output()
            .map_err(|e| format!("Failed to invoke shortcut: {}", e))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }
}
