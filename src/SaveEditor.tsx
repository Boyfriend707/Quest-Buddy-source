import { useEffect, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface EditableField {
  key: string
  value: string
  type: 'number' | 'bool' | 'string'
}

export default function SaveEditor() {
  const [fields, setFields] = useState<EditableField[]>([])
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState<string | null>(null)
  const [msg, setMsg] = useState('')
  const [search, setSearch] = useState('')

  const load = useCallback(async () => {
    setLoading(true)
    setMsg('')
    try {
      const pd = await invoke<Record<string, unknown>>('get_save_fields')
      const entries: EditableField[] = []
      for (const [key, val] of Object.entries(pd)) {
        if (typeof val === 'number') {
          entries.push({ key, value: String(val), type: 'number' })
        } else if (typeof val === 'boolean') {
          entries.push({ key, value: val ? 'true' : 'false', type: 'bool' })
        } else if (typeof val === 'string') {
          entries.push({ key, value: val, type: 'string' })
        }
      }
      entries.sort((a, b) => a.key.localeCompare(b.key))
      setFields(entries)
    } catch (e) {
      setMsg(String(e))
    }
    setLoading(false)
  }, [])

  useEffect(() => { load() }, [load])

  const updateField = (key: string, newValue: string) => {
    setFields(prev => prev.map(f => f.key === key ? { ...f, value: newValue } : f))
  }

  const saveField = async (field: EditableField) => {
    setSaving(field.key)
    setMsg('')
    try {
      let val: unknown
      if (field.type === 'number') {
        val = field.value.includes('.') ? parseFloat(field.value) : parseInt(field.value, 10)
      } else if (field.type === 'bool') {
        val = field.value === 'true'
      } else {
        val = field.value
      }
      await invoke('write_save_field', { field: field.key, value: val })
      setMsg(`Saved "${field.key}"`)
    } catch (e) {
      setMsg(`Error: ${e}`)
    }
    setSaving(null)
  }

  const filtered = search
    ? fields.filter(f => f.key.toLowerCase().includes(search.toLowerCase()))
    : fields

  return (
    <div className="save-editor">
      <div className="save-editor-header">
        <h3>Save Editor</h3>
        <button className="tab" onClick={load} disabled={loading}>
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      </div>

      {msg && <p className="save-editor-msg">{msg}</p>}

      {!loading && fields.length > 0 && (
        <div className="save-editor-search">
          <input
            className="settings-input"
            type="text"
            placeholder="Search fields..."
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
        </div>
      )}

      {loading ? (
        <p className="save-editor-empty">Loading save data...</p>
      ) : fields.length === 0 ? (
        <p className="save-editor-empty">No save file found.</p>
      ) : filtered.length === 0 ? (
        <p className="save-editor-empty">No fields match "{search}"</p>
      ) : (
        <div className="save-editor-list">
          {filtered.map(f => (
            <div key={f.key} className="save-editor-row">
              <span className="save-editor-key">{f.key}</span>
              <input
                className="save-editor-input"
                type="text"
                value={f.value}
                onChange={e => updateField(f.key, e.target.value)}
                onKeyDown={e => { if (e.key === 'Enter') saveField(f) }}
              />
              <span className="save-editor-type">{f.type}</span>
              <button
                className="save-editor-save"
                onClick={() => saveField(f)}
                disabled={saving === f.key}
              >
                {saving === f.key ? '...' : 'Save'}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
