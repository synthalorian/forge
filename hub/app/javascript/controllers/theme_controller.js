import { Controller } from "@hotwired/stimulus"

const THEMES = ["synthwave84", "midnight", "ocean", "light"]
const COOKIE_NAME = "forge_hub_theme"
const COOKIE_DAYS = 365

export default class extends Controller {
  static values = { current: { type: String, default: "synthwave84" } }

  connect() {
    const saved = this.readCookie()
    if (saved && THEMES.includes(saved)) {
      this.applyTheme(saved)
    } else {
      this.applyTheme(this.currentValue)
    }
  }

  select(e) {
    const theme = e.currentTarget.dataset.themeValue
    if (theme && THEMES.includes(theme)) {
      this.applyTheme(theme)
      this.writeCookie(theme)
      this.closeAllMenus()
    }
  }

  applyTheme(theme) {
    document.documentElement.setAttribute("data-theme", theme)
    this.currentValue = theme
    document.querySelectorAll("[data-theme-picker-active]").forEach(el => {
      el.textContent = el.dataset.themeLabel || theme
    })
    document.querySelectorAll(".theme-dropdown-item").forEach(el => {
      el.classList.toggle("active", el.dataset.themeValue === theme)
    })
  }

  toggleMenu(e) {
    e.stopPropagation()
    const menu = e.currentTarget.nextElementSibling
    const isOpen = menu.classList.contains("open")
    this.closeAllMenus()
    if (!isOpen) menu.classList.add("open")
  }

  closeAllMenus() {
    document.querySelectorAll(".theme-dropdown-menu").forEach(m => m.classList.remove("open"))
  }

  readCookie() {
    const match = document.cookie.match(new RegExp(`(?:^|; )${COOKIE_NAME}=([^;]*)`))
    return match ? decodeURIComponent(match[1]) : null
  }

  writeCookie(theme) {
    const expires = new Date(Date.now() + COOKIE_DAYS * 864e5).toUTCString()
    document.cookie = `${COOKIE_NAME}=${encodeURIComponent(theme)}; expires=${expires}; path=/; SameSite=Lax`
  }
}
