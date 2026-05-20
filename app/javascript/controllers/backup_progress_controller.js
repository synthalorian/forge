import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { jobId: String }

  connect() {
    this._scrollContainer = this.element.querySelector("#backup-output")
    this._observer = new MutationObserver(() => {
      this.autoScroll()
    })

    if (this._scrollContainer) {
      this._observer.observe(this._scrollContainer, {
        childList: true,
        subtree: true
      })
    }
  }

  disconnect() {
    if (this._observer) {
      this._observer.disconnect()
    }
  }

  autoScroll() {
    if (this._scrollContainer) {
      this._scrollContainer.scrollTop = this._scrollContainer.scrollHeight
    }
  }
}
