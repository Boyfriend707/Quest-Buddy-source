import { getCurrentWindow } from '@tauri-apps/api/window'
import type { MouseEvent } from 'react'

const win = getCurrentWindow()

function stop(e: MouseEvent) {
  e.stopPropagation()
}

export default function TitleBar() {
  return (
    <div className="titlebar" onMouseDown={() => win.startDragging()}>
      <span className="titlebar-title">QuestBuddy</span>
      <div className="titlebar-controls">
        <button className="titlebar-btn" onMouseDown={stop} onClick={() => win.minimize()} title="Minimize">&#x2500;</button>
        <button className="titlebar-btn" onMouseDown={stop} onClick={() => win.toggleMaximize()} title="Maximize">&#x25A1;</button>
        <button className="titlebar-btn titlebar-close" onMouseDown={stop} onClick={() => win.hide()} title="Hide to tray">&#x2715;</button>
      </div>
    </div>
  )
}
