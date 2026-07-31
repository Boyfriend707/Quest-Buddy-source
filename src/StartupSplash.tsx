import { useEffect, useState } from 'react'

export default function StartupSplash() {
  const [done, setDone] = useState(false)

  useEffect(() => {
    const t = setTimeout(() => setDone(true), 2000)
    return () => clearTimeout(t)
  }, [])

  if (done) return null

  return (
    <div className="startup-splash">
      <div className="startup-logo">QB</div>
      <div className="startup-name">QuestBuddy</div>
    </div>
  )
}
