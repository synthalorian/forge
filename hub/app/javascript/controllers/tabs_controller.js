import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["tab", "panel"]

  connect() {
    // Restore last active tab from sessionStorage
    const saved = sessionStorage.getItem("forge_crucible_tab")
    if (saved) {
      this.selectTab(saved)
    }
  }

  select(e) {
    const tab = e.currentTarget.dataset.tab
    sessionStorage.setItem("forge_crucible_tab", tab)
    this.selectTab(tab)
  }

  selectTab(tab) {
    this.tabTargets.forEach(el => {
      const isActive = el.dataset.tab === tab
      el.classList.toggle("text-neon-cyan", isActive)
      el.classList.toggle("border-b-2", isActive)
      el.classList.toggle("border-neon-cyan", isActive)
      el.classList.toggle("text-text-muted", !isActive)
      el.classList.toggle("border-transparent", !isActive)
    })

    this.panelTargets.forEach(el => {
      el.style.display = el.dataset.tab === tab ? "" : "none"
    })
  }
}
