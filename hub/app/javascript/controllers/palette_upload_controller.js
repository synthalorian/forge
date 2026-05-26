import { Controller } from "@hotwired/stimulus"
import { Turbo } from "@hotwired/turbo-rails"

export default class extends Controller {
  static targets = ["dropzone", "preview", "fileInput", "format", "submitBtn", "error"]

  connect() {
    this.selectedFile = null
    // Bind the suppress handler so we can remove it in disconnect
    this._boundSuppress = this._suppress.bind(this)
    ;["dragenter", "dragover", "dragleave", "drop"].forEach((name) => {
      document.addEventListener(name, this._boundSuppress)
    })
  }

  disconnect() {
    if (this._boundSuppress) {
      ;["dragenter", "dragover", "dragleave", "drop"].forEach((name) => {
        document.removeEventListener(name, this._boundSuppress)
      })
    }
  }

  _suppress(e) {
    e.preventDefault()
    e.stopPropagation()
  }

  // Click the dropzone => open file picker
  click() {
    this.fileInputTarget.click()
  }

  dragOver() {
    this.dropzoneTarget.classList.add("border-neon-red", "bg-neon-red/5")
  }

  dragLeave() {
    this.dropzoneTarget.classList.remove("border-neon-red", "bg-neon-red/5")
  }

  drop(event) {
    this.dragLeave()
    const files = event.dataTransfer.files
    if (files.length > 0) {
      this._loadFile(files[0])
    }
  }

  fileSelected(event) {
    const file = event.target.files[0]
    if (file) {
      this._loadFile(file)
    }
  }

  _loadFile(file) {
    // Clear previous error
    if (this.hasErrorTarget) {
      this.errorTarget.textContent = ""
    }

    // Validate type
    if (!file.type.match(/^image\/(png|jpe?g)$/)) {
      if (this.hasErrorTarget) {
        this.errorTarget.textContent = "Only PNG and JPG images are supported"
      }
      return
    }

    // Validate size (max 10MB)
    if (file.size > 10 * 1024 * 1024) {
      if (this.hasErrorTarget) {
        this.errorTarget.textContent = "Image must be smaller than 10MB"
      }
      return
    }

    this.selectedFile = file

    // Show preview — replaces the placeholder content
    const reader = new FileReader()
    reader.onload = (e) => {
      this.previewTarget.innerHTML =
        `<img src="${e.target.result}" ` +
        `class="max-h-[300px] w-auto rounded-lg border border-border-faint shadow-sm" />`
    }
    reader.readAsDataURL(file)

    // Enable submit
    if (this.hasSubmitBtnTarget) {
      this.submitBtnTarget.disabled = false
    }
  }

  upload(event) {
    event.preventDefault()
    if (!this.selectedFile) return

    this.submitBtnTarget.disabled = true
    this.submitBtnTarget.textContent = "Extracting..."
    if (this.hasErrorTarget) {
      this.errorTarget.textContent = ""
    }

    const formData = new FormData()
    formData.append("image", this.selectedFile)
    formData.append("format", this.formatTarget.value)

    fetch("/crucible/palette/upload", {
      method: "POST",
      body: formData,
      headers: { Accept: "text/vnd.turbo-stream.html" }
    })
      .then((r) => r.text())
      .then((html) => {
        Turbo.renderStreamMessage(html)
        this.submitBtnTarget.disabled = false
        this.submitBtnTarget.textContent = "Extract Palette"
      })
      .catch((err) => {
        console.error("Palette upload failed:", err)
        if (this.hasErrorTarget) {
          this.errorTarget.textContent = "Upload failed — check console for details"
        }
        this.submitBtnTarget.disabled = false
        this.submitBtnTarget.textContent = "Extract Palette"
      })
  }
}