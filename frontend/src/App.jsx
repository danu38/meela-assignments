import React from 'react'
import { Routes, Route, Link } from 'react-router-dom'
import Layout from "./components/Layout"
import Home from './pages/Home'
import IntakeForm from './pages/IntakeForm'

const box = { maxWidth: 720, margin: '40px auto', padding: 20, fontFamily: 'system-ui, sans-serif' }

export default function App() {
  return (
    <Layout>
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/form/:id" element={<IntakeForm />} />
      <Route path="*" element={<div style={box}><p>Not found. <Link to="/">Go home</Link></p></div>} />
    </Routes>
     </Layout>
  )
}