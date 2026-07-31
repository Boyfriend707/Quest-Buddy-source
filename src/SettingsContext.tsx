import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { getCurrentWindow, PhysicalSize, PhysicalPosition } from '@tauri-apps/api/window'

export type Theme = 'dark' | 'purple' | 'light' | 'forest' | 'ocean' | 'amber' | 'rose' | 'slate'
export type FontSize = 'small' | 'default' | 'large'

export interface Settings {
  theme: Theme
  fontSize: FontSize
  compact: boolean
  alwaysOnTop: boolean
  startMinimized: boolean
  startupAnimation: boolean
  showItems: boolean
  showBosses: boolean
  showNextSteps: boolean
  autoStart: boolean
  rememberWindow: boolean
  customSaveHk: string
  customSaveSs: string
}

const STORAGE_KEY = 'quest-buddy-settings'

export const defaultSettings: Settings = {
  theme: 'dark',
  fontSize: 'default',
  compact: false,
  alwaysOnTop: false,
  startMinimized: false,
  startupAnimation: true,
  showItems: true,
  showBosses: true,
  showNextSteps: true,
  autoStart: false,
  rememberWindow: false,
  customSaveHk: '',
  customSaveSs: '',
}

function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...defaultSettings, ...JSON.parse(raw) }
  } catch { /* ignore */ }
  return { ...defaultSettings }
}

function saveSettings(s: Settings) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s))
}

function applySettings(s: Settings) {
  document.documentElement.setAttribute('data-theme', s.theme)
  document.documentElement.setAttribute('data-compact', String(s.compact))
  const sizes: Record<FontSize, string> = { small: '14px', default: '16px', large: '18px' }
  document.documentElement.style.fontSize = sizes[s.fontSize]
}

interface SettingsCtx {
  settings: Settings
  update: (patch: Partial<Settings>) => void
  reset: () => void
}

const Ctx = createContext<SettingsCtx>({
  settings: defaultSettings,
  update: () => {},
  reset: () => {},
})

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(loadSettings)

  useEffect(() => {
    applySettings(settings)
  }, [settings])

  useEffect(() => {
    const s = loadSettings()
    if (s.startMinimized) {
      getCurrentWindow().hide()
    }
    if (s.alwaysOnTop) {
      getCurrentWindow().setAlwaysOnTop(true)
    }
    if (s.rememberWindow) {
      const savedPos = localStorage.getItem('quest-buddy-window')
      if (savedPos) {
        try {
          const { x, y, w, h } = JSON.parse(savedPos)
          const win = getCurrentWindow()
          if (w && h) win.setSize(new PhysicalSize(w, h))
          if (x !== undefined && y !== undefined) win.setPosition(new PhysicalPosition(x, y))
        } catch { /* ignore */ }
      }
    }
  }, [])

  useEffect(() => {
    if (settings.rememberWindow) {
      const win = getCurrentWindow()
      let timer: ReturnType<typeof setTimeout>
      const save = async () => {
        try {
          const pos = await win.outerPosition()
          const size = await win.outerSize()
          localStorage.setItem('quest-buddy-window', JSON.stringify({
            x: pos.x, y: pos.y, w: size.width, h: size.height,
          }))
        } catch { /* ignore */ }
      }
      const handler = () => { clearTimeout(timer); timer = setTimeout(save, 300) }
      window.addEventListener('resize', handler)
      window.addEventListener('beforeunload', save)
      return () => {
        window.removeEventListener('resize', handler)
        window.removeEventListener('beforeunload', save)
      }
    }
  }, [settings.rememberWindow])

  const update = (patch: Partial<Settings>) => {
    setSettings(prev => {
      const next = { ...prev, ...patch }
      saveSettings(next)
      return next
    })
  }

  const reset = () => {
    setSettings({ ...defaultSettings })
    localStorage.removeItem(STORAGE_KEY)
  }

  return <Ctx.Provider value={{ settings, update, reset }}>{children}</Ctx.Provider>
}

export function useSettings() {
  return useContext(Ctx)
}
