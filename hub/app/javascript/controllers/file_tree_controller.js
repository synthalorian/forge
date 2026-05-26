import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["toggle", "children"]

  toggle(e) {
    const target = e.currentTarget
    const dirPath = target.dataset.dirPath
    const children = this.element.querySelector(`[data-dir-children="${dirPath}"]`)
    const icon = target.querySelector(".file-tree-icon")

    if (!children) return

    const isHidden = children.classList.contains("hidden")
    children.classList.toggle("hidden")

    if (icon) {
      icon.textContent = isHidden ? "▾" : "▸"
    }
  }

  // Expand all directories (can be connected to a button)
  expandAll() {
    this.childrenTargets.forEach(el => el.classList.remove("hidden"))
    this.toggleTargets.forEach(el => {
      const icon = el.querySelector(".file-tree-icon")
      if (icon) icon.textContent = "▾"
    })
  }

  // Collapse all directories
  collapseAll() {
    this.childrenTargets.forEach(el => el.classList.add("hidden"))
    this.toggleTargets.forEach(el => {
      const icon = el.querySelector(".file-tree-icon")
      if (icon) icon.textContent = "▸"
    })
  }
}