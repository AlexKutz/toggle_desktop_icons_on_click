import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface AppConfig {
  features: {
    desktop_toggler: boolean;
    cursor_hider: boolean;
  };
  cursor_hider: {
    timeout_seconds: number;
  };
}

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, []);

  async function loadConfig() {
    try {
      const loadedConfig = await invoke<AppConfig>("load_config");
      setConfig(loadedConfig);
      setLoading(false);
    } catch (error) {
      console.error("Failed to load config:", error);
      setMessage("Error loading settings: " + error);
      setLoading(false);
    }
  }

  async function saveConfig() {
    if (!config) return;
    
    setSaving(true);
    setMessage("");
    
    try {
      const result = await invoke<string>("save_config", { config });
      setMessage(result);
      setTimeout(() => setMessage(""), 3000);
    } catch (error) {
      console.error("Failed to save config:", error);
      setMessage("Error saving settings: " + error);
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return (
      <div className="container">
        <div className="loading">Loading settings...</div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="container">
        <div className="error">Failed to load configuration</div>
      </div>
    );
  }

  return (
    <main className="container">
      <h1>Desktop Icon Toggler</h1>
      <h2>Settings</h2>

      <div className="settings-form">
        {/* Desktop Toggler Toggle */}
        <div className="setting-item">
          <div className="setting-label">
            <h3>Desktop Icon Toggler</h3>
            <p>Toggle desktop icons on double-click</p>
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={config.features.desktop_toggler}
              onChange={(e) =>
                setConfig({
                  ...config,
                  features: {
                    ...config.features,
                    desktop_toggler: e.target.checked,
                  },
                })
              }
            />
            <span className="toggle-slider"></span>
          </label>
        </div>

        {/* Cursor Hider Toggle */}
        <div className="setting-item">
          <div className="setting-label">
            <h3>Cursor Hider</h3>
            <p>Automatically hide cursor after inactivity</p>
          </div>
          <label className="toggle">
            <input
              type="checkbox"
              checked={config.features.cursor_hider}
              onChange={(e) =>
                setConfig({
                  ...config,
                  features: {
                    ...config.features,
                    cursor_hider: e.target.checked,
                  },
                })
              }
            />
            <span className="toggle-slider"></span>
          </label>
        </div>

        {/* Cursor Hider Timeout */}
        {config.features.cursor_hider && (
          <div className="setting-item setting-item-indent">
            <div className="setting-label">
              <h3>Cursor Hide Timeout</h3>
              <p>Seconds of inactivity before hiding cursor</p>
            </div>
            <input
              type="number"
              min="1"
              max="60"
              value={config.cursor_hider.timeout_seconds}
              onChange={(e) =>
                setConfig({
                  ...config,
                  cursor_hider: {
                    timeout_seconds: parseInt(e.target.value) || 5,
                  },
                })
              }
              className="number-input"
            />
          </div>
        )}

        {/* Message Display */}
        {message && (
          <div className={`message ${message.includes("Error") ? "error" : "success"}`}>
            {message}
          </div>
        )}

        {/* Save Button */}
        <button className="save-button" onClick={saveConfig} disabled={saving}>
          {saving ? "Saving..." : "Save Settings"}
        </button>

        <p className="info-text">
          Changes will be applied immediately to the running application.
        </p>
      </div>
    </main>
  );
}

export default App;
