import { useState } from 'react'
import { relaunch } from '@tauri-apps/plugin-process'
import type { Update } from '@tauri-apps/plugin-updater'

interface Props {
  update: Update
  onDismiss: () => void
}

function UpdateBanner({ update, onDismiss }: Props) {
  const [state, setState] = useState<'idle' | 'downloading' | 'error'>('idle')

  async function doUpdate() {
    try {
      setState('downloading')
      await update.downloadAndInstall()
      await relaunch()
    } catch (err) {
      console.error('update install failed', err)
      setState('error')
    }
  }

  return (
    <div className="update-banner">
      <div className="update-info">
        <div className="update-title">Update available: v{update.version}</div>
        <div className="update-sub">A new version of QuestBuddy is ready to install.</div>
      </div>
      {state === 'error' && <div className="update-error">Update failed. Please try again.</div>}
      <div className="update-actions">
        <button className="update-btn primary" onClick={doUpdate} disabled={state === 'downloading'}>
          {state === 'downloading' ? 'Downloading…' : 'Update'}
        </button>
        <button className="update-btn" onClick={onDismiss} disabled={state === 'downloading'}>
          Later
        </button>
      </div>
    </div>
  )
}

export default UpdateBanner
