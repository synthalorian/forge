// Stimulus controller for the dotfile track form
// Bridges the text input value to the hidden form param on submit
import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["input", "hidden"]

  connect() {
    this.element.addEventListener("submit", (e) => {
      const hidden = this.element.querySelector("input[name='path']")
      const input = this.element.querySelector("[data-dotfile-track-target='input']")
      if (hidden && input) {
        hidden.value = input.value
      }
    })
  }
}
