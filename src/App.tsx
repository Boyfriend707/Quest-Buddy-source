import { useEffect, useState, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import TitleBar from './TitleBar'
import Settings from './Settings'
import SaveEditor from './SaveEditor'
import { useSettings } from './SettingsContext'

interface Progress {
  game: string
  running: boolean
  percentage: number
  geo: number
  items: string[]
  next_steps: string[]
  play_time_formatted: string
  max_health: number
  soul_vessels: number
  dream_essence: number
  grubs: number
  nail_damage: number
  nail_name: string
  bosses: string[]
  charms_count: number
  has_save: boolean
}

interface GameStatus {
  running: boolean
}

function App() {
  const [progress, setProgress] = useState<Progress | null>(null)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [tab, setTab] = useState<'progress' | 'next-steps' | 'editor'>('progress')
  const { settings } = useSettings()
  const notified = useRef(false)

  useEffect(() => {
    invoke<Progress>('get_progress').then(setProgress).catch(console.error)

    const unlistenProgress = listen<Progress>('progress-updated', (e) => {
      setProgress(e.payload)
    })

    const unlistenStatus = listen<GameStatus>('game-status-changed', (e) => {
      setProgress(prev => prev ? { ...prev, running: e.payload.running } : null)
      if (e.payload.running && !notified.current) {
        notified.current = true
        if ('Notification' in window && Notification.permission === 'granted') {
          new Notification('QuestBuddy', { body: 'Game detected!' })
        }
      }
      if (!e.payload.running) {
        notified.current = false
      }
    })

    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission()
    }

    return () => {
      unlistenProgress.then(fn => fn())
      unlistenStatus.then(fn => fn())
    }
  }, [])

  const showProgress = tab === 'progress'
  const showNext = tab === 'next-steps'
  const showEditor = tab === 'editor'

  return (
    <div className="app">
      <TitleBar />
      <div className="container">
        <header>
          <div className="header-left">
            <div className="logo">QB</div>
            <h1>QuestBuddy</h1>
          </div>
          <div className="header-right">
            <span className={`badge ${progress?.running ? 'running' : progress?.has_save ? 'saved' : 'not_running'}`}>
              {progress?.running ? 'Playing' : progress?.has_save ? 'Saved' : 'No save'}
            </span>
            <button className="gear-btn" onClick={() => setSettingsOpen(!settingsOpen)} title="Settings">
              &#9881;
            </button>
            <Settings open={settingsOpen} onClose={() => setSettingsOpen(false)} />
          </div>
        </header>

        {progress && (
          <div className="game-card">
            <div className="card-header">
              <h2>{progress.game}</h2>
              <span className="play-time">{progress.play_time_formatted}</span>
            </div>

            <div className="tabs">
              <button className={`tab ${showProgress ? 'active' : ''}`} onClick={() => setTab('progress')}>Progress</button>
              <button className={`tab ${showNext ? 'active' : ''}`} onClick={() => setTab('next-steps')}>Next Steps</button>
              <button className={`tab ${showEditor ? 'active' : ''}`} onClick={() => setTab('editor')}>Save Editor</button>
            </div>

            <>
            {showProgress && (
              <>
                <div className="progress-bar">
                  <div className="progress-fill" style={{ width: `${progress.percentage}%` }} />
                </div>
                <p className="percent">{progress.percentage.toFixed(1)}% complete</p>

                <div className="stats-grid">
                  <div className="stat" style={{ '--stat-accent': '#f59e0b' } as React.CSSProperties}>
                    <span className="stat-icon">G</span>
                    <span className="stat-label">Geo</span>
                    <span className="stat-value">{progress.geo.toLocaleString()}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#a78bfa' } as React.CSSProperties}>
                    <span className="stat-icon">S</span>
                    <span className="stat-label">Soul</span>
                    <span className="stat-value">{progress.soul_vessels}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#f472b6' } as React.CSSProperties}>
                    <span className="stat-icon">E</span>
                    <span className="stat-label">Dream Essence</span>
                    <span className="stat-value">{progress.dream_essence.toLocaleString()}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#34d399' } as React.CSSProperties}>
                    <span className="stat-icon">G</span>
                    <span className="stat-label">Grubs</span>
                    <span className="stat-value">{progress.grubs}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#60a5fa' } as React.CSSProperties}>
                    <span className="stat-icon">N</span>
                    <span className="stat-label">Nail</span>
                    <span className="stat-value">{progress.nail_name}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#fb7185' } as React.CSSProperties}>
                    <span className="stat-icon">C</span>
                    <span className="stat-label">Charms</span>
                    <span className="stat-value">{progress.charms_count}</span>
                  </div>
                  <div className="stat" style={{ '--stat-accent': '#f97316' } as React.CSSProperties}>
                    <span className="stat-icon">B</span>
                    <span className="stat-label">Bosses</span>
                    <span className="stat-value">{progress.bosses.length}</span>
                  </div>
                </div>

                {settings.showItems && progress.items.length > 0 && (
                  <div className="section">
                    <h3>Items Collected</h3>
                    <div className="tag-list">
                      {progress.items.map((item, i) => (
                        <span key={i} className="tag">{item}</span>
                      ))}
                    </div>
                  </div>
                )}

                {settings.showBosses && progress.bosses.length > 0 && (
                  <div className="section">
                    <h3>Bosses Defeated</h3>
                    <div className="tag-list">
                      {progress.bosses.map((boss, i) => (
                        <span key={i} className="tag boss-tag">{boss}</span>
                      ))}
                    </div>
                  </div>
                )}

                {settings.showNextSteps && (
                  <div className="section">
                    <h3>Next Steps</h3>
                    <ul className="steps">
                      {progress.next_steps.map((step, i) => (
                        <li key={i}>{step}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </>
            )}

            {showNext && (
              <div className="section">
                <ul className="steps">
                  {progress.next_steps.map((step, i) => (
                    <li key={i}>{step}</li>
                  ))}
                </ul>
              </div>
            )}

            {showEditor && <SaveEditor />}
            </>
          </div>
        )}
      </div>
    </div>
  )
}

export default App
