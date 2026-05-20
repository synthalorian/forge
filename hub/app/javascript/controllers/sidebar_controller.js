import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["sidebar", "toggleIcon", "overlay"]
  static classes = ["collapsed"]

  connect() {
    const collapsed = localStorage.getItem("forge:sidebar:collapsed") === "true"
    if (collapsed) {
      this.sidebarTarget.classList.add(this.collapsedClass)
      this.toggleIconTarget.textContent = "▶"
    }
  }

  toggle() {
    const wasCollapsed = this.sidebarTarget.classList.contains(this.collapsedClass)
    this.sidebarTarget.classList.toggle(this.collapsedClass)
    this.toggleIconTarget.textContent = wasCollapsed ? "◀" : "▶"
    localStorage.setItem("forge:sidebar:collapsed", !wasCollapsed)
    this.dispatch("sidebar-toggled", { detail: { collapsed: !wasCollapsed } })
  }
}
