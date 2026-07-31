import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { useSettings, type Theme, type FontSize } from './SettingsContext'
import { getCurrentWindow } from '@tauri-apps/api/window'

const themes: { value: Theme; label: string }[] = [
  { value: 'dark', label: 'Dark' },
  { value: 'purple', label: 'Purple' },
  { value: 'light', label: 'Light' },
  { value: 'forest', label: 'Forest' },
  { value: 'ocean', label: 'Ocean' },
  { value: 'amber', label: 'Amber' },
  { value: 'rose', label: 'Rose' },
  { value: 'slate', label: 'Slate' },
]

const sizes: { value: FontSize; label: string }[] = [
  { value: 'small', label: 'Small' },
  { value: 'default', label: 'Default' },
  { value: 'large', label: 'Large' },
]

interface Props {
  open: boolean
  onClose: () => void
}

export default function Settings({ open, onClose }: Props) {
  const { settings, update, reset } = useSettings()
  const [autoStartState, setAutoStartState] = useState(false)
  const [appVersion, setAppVersion] = useState('')

  useEffect(() => {
    invoke<boolean>('get_auto_start').then(setAutoStartState).catch(() => {})
  }, [])

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {})
  }, [])

  return (
    <>
      <div className={`settings-overlay ${open ? 'open' : ''}`} onClick={onClose} />
      <div className={`settings-panel ${open ? 'open' : ''}`}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={onClose}>&#x2715;</button>
        </div>

        <div className="settings-body">
          <div className="settings-group">
            <h3>Appearance</h3>
            <div className="settings-inner">
              <label className="settings-label">Theme</label>
              <div className="settings-row wrap">
                {themes.map(t => (
                  <button
                    key={t.value}
                    className={`settings-btn ${settings.theme === t.value ? 'active' : ''}`}
                    onClick={() => update({ theme: t.value })}
                  >
                    {t.label}
                  </button>
                ))}
              </div>

              <label className="settings-label">Font Size</label>
              <div className="settings-row">
                {sizes.map(s => (
                  <button
                    key={s.value}
                    className={`settings-btn ${settings.fontSize === s.value ? 'active' : ''}`}
                    onClick={() => update({ fontSize: s.value })}
                  >
                    {s.label}
                  </button>
                ))}
              </div>

              <label className="settings-label">Layout</label>
              <div className="settings-row">
                <button
                  className={`settings-btn ${!settings.compact ? 'active' : ''}`}
                  onClick={() => update({ compact: false })}
                >
                  Normal
                </button>
                <button
                  className={`settings-btn ${settings.compact ? 'active' : ''}`}
                  onClick={() => update({ compact: true })}
                >
                  Compact
                </button>
              </div>
            </div>
          </div>

          <div className="settings-group">
            <h3>Window</h3>
            <div className="settings-inner">
              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={settings.alwaysOnTop}
                  onChange={() => {
                    update({ alwaysOnTop: !settings.alwaysOnTop })
                    getCurrentWindow().setAlwaysOnTop(!settings.alwaysOnTop)
                  }}
                />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Always on top
              </label>

              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={settings.startMinimized}
                  onChange={() => {
                    update({ startMinimized: !settings.startMinimized })
                    if (!settings.startMinimized) getCurrentWindow().hide()
                  }}
                />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Start minimized
              </label>

              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={settings.startupAnimation}
                  onChange={() => update({ startupAnimation: !settings.startupAnimation })}
                />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Startup animation
              </label>

              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={settings.rememberWindow}
                  onChange={() => update({ rememberWindow: !settings.rememberWindow })}
                />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Remember position
              </label>

              <label className="settings-toggle">
                <input
                  type="checkbox"
                  checked={autoStartState}
                  onChange={async () => {
                    const next = !autoStartState
                    setAutoStartState(next)
                    await invoke('set_auto_start', { enabled: next })
                    update({ autoStart: next })
                  }}
                />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Auto-start with Windows
              </label>
            </div>
          </div>

          <div className="settings-group">
            <h3>Sections</h3>
            <div className="settings-inner">
              <label className="settings-toggle">
                <input type="checkbox" checked={settings.showItems} onChange={() => update({ showItems: !settings.showItems })} />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Items Collected
              </label>
              <label className="settings-toggle">
                <input type="checkbox" checked={settings.showBosses} onChange={() => update({ showBosses: !settings.showBosses })} />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Bosses Defeated
              </label>
              <label className="settings-toggle">
                <input type="checkbox" checked={settings.showNextSteps} onChange={() => update({ showNextSteps: !settings.showNextSteps })} />
                <span className="toggle-track"><span className="toggle-knob" /></span>
                Next Steps
              </label>
            </div>
          </div>

          <div className="settings-group">
            <h3>Save Paths</h3>
            <div className="settings-inner">
              <label className="settings-label">Hollow Knight</label>
              <input
                className="settings-input"
                type="text"
                placeholder="%USERPROFILE%\AppData\LocalLow\Team Cherry\Hollow Knight"
                value={settings.customSaveHk}
                onChange={e => update({ customSaveHk: e.target.value })}
              />
              <label className="settings-label">Silksong</label>
              <input
                className="settings-input"
                type="text"
                placeholder="%USERPROFILE%\AppData\LocalLow\Team Cherry\Silksong"
                value={settings.customSaveSs}
                onChange={e => update({ customSaveSs: e.target.value })}
              />
            </div>
          </div>

          <button className="settings-reset" onClick={() => reset()}>
            Reset to defaults
          </button>
          <div className="settings-version">v{appVersion}</div>
        </div>
      </div>
    </>
  )
}
