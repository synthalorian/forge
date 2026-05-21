import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["input", "results", "form"]

  connect() {
    this.debounceTimer = null
  }

  search() {
    clearTimeout(this.debounceTimer)
    this.debounceTimer = setTimeout(() => {
      this.submitSearch()
    }, 400)
  }

  submitSearch() {
    const query = this.inputTarget.value.trim()
    if (query.length < 2) {
      this.resultsTarget.innerHTML = ""
      return
    }

    this.formTarget.requestSubmit()
  }

  clear() {
    this.inputTarget.value = ""
    this.resultsTarget.innerHTML = ""
  }
}
