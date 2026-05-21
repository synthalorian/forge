import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["input", "badge", "form"]

  connect() {
    this.debounceTimer = null
  }

  search() {
    clearTimeout(this.debounceTimer)
    const query = this.inputTarget.value.trim()

    if (query.length < 2) {
      this.badgeTarget.classList.add("hidden")
      this.badgeTarget.textContent = ""
      return
    }

    this.badgeTarget.classList.remove("hidden")
    this.badgeTarget.textContent = "..."
    this.debounceTimer = setTimeout(() => {
      this.submitSearch()
    }, 400)
  }

  submit(event) {
    if (event.key === "Enter") {
      clearTimeout(this.debounceTimer)
    }
  }

  submitSearch() {
    const query = this.inputTarget.value.trim()
    if (query.length < 2) return
    this.formTarget.requestSubmit()
  }

  disconnect() {
    clearTimeout(this.debounceTimer)
  }
}
