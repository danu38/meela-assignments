import React, { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { createDraft } from '../api'

const box = { maxWidth: 720, margin: '40px auto', padding: 20, border: '1px solid #ddd', borderRadius: 12, fontFamily: 'system-ui, sans-serif' }

export default function Home() {
  const nav = useNavigate()
  const [creatingDrafts, setcreatingDrafts] = useState(false)

  const start = async () => {
    setcreatingDrafts(true)
    try {
      const { id } = await createDraft()
      nav(`/form/${id}`)
    } finally {
      setcreatingDrafts(false)
    }
  }

  return (
    <div style={box}>
      <h2>Client Onboarding (POC)</h2>
      <p>This demo saves your progress server-side so you can resume later.</p>
      <button onClick={start} disabled={creatingDrafts}>
        {creatingDrafts ? 'Starting…' : 'Start new intake'}
      </button>
    </div>
  )
}