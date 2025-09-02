import React from "react"
import { Link } from "react-router-dom"

export default function Layout({ children }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", minHeight: "100vh", background: "var(--bg)" }}>
      <header className="navbar">
        <div>Meela Intake</div>
        <nav>
          <Link to="/">Home</Link>
        </nav>
      </header>

      <main style={{ flex: 1 }}>{children}</main>

      <footer className="footer">
        <p>
          © {new Date().getFullYear()} Meela Health •{" "}
          <a href="https://meelahealth.com" target="_blank" rel="noopener noreferrer">
            Visit Website
          </a>
        </p>
      </footer>
    </div>
  )
}