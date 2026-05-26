import { Controller } from "@hotwired/stimulus"

// ── Preset pipelines ──────────────────────────────────────────
const PRESETS = {
  code_review: {
    label: "Code Review",
    steps: [
      { name: "review_diff",  task: "Review this PR diff for bugs, style issues, and security concerns",          agent: "auto",      input_keys: "pr_diff" },
      { name: "summarize",    task: "Summarize the changes in this pull request for the team",                     agent: "auto",      input_keys: "" },
    ],
  },
  research_write: {
    label: "Research & Write",
    steps: [
      { name: "search",      task: "Search the web for latest information on the given topic",                     agent: "opencode",  input_keys: "topic" },
      { name: "summarize",   task: "Summarize the search results into key findings",                               agent: "llama-swap", input_keys: "" },
      { name: "draft",       task: "Write a comprehensive article based on the summarized findings",               agent: "hermes",     input_keys: "" },
    ],
  },
  code_gen_test: {
    label: "Code Gen + Test",
    steps: [
      { name: "generate",    task: "Write production-ready code for the specified feature",                        agent: "codex",     input_keys: "spec" },
      { name: "review",      task: "Review the generated code for correctness and best practices",                 agent: "auto",      input_keys: "" },
      { name: "test",        task: "Write unit tests for the generated code",                                      agent: "opencode",  input_keys: "" },
    ],
  },
  data_pipeline: {
    label: "Data Pipeline",
    steps: [
      { name: "extract",     task: "Extract data from the specified source",                                       agent: "opencode",  input_keys: "source" },
      { name: "transform",   task: "Clean and transform the extracted data into the desired format",               agent: "llama-swap", input_keys: "" },
      { name: "analyze",     task: "Analyze the transformed data and produce insights",                            agent: "hermes",    input_keys: "" },
    ],
  },
}

const AGENTS = ["auto", "opencode", "llama-swap", "hermes", "codex"]

export default class extends Controller {
  static targets = ["stepsContainer", "hiddenToml", "collapsedInfo", "expandedBody"]

  connect() {
    this.steps = []
    this.renderCollapsed()
    this.renderExpanded()
    this.applyToggleState()
  }

  // ── Expand / Collapse ────────────────────────────────────────
  toggleExpand() {
    const body = this.expandedBodyTarget
    const isHidden = body.style.display === "none"
    body.style.display = isHidden ? "" : "none"
    this.applyToggleState()
  }

  applyToggleState() {
    const body = this.expandedBodyTarget
    const isHidden = body.style.display === "none"
    // Update button text (first child span inside the button)
    const btn = this.element.querySelector("[data-action='pipeline-builder#toggleExpand']")
    if (btn) {
      btn.firstElementChild.textContent = isHidden ? "▸ Expand" : "▾ Collapse"
    }
  }

  // ── Presets ──────────────────────────────────────────────────
  loadPreset(e) {
    const key = e.target.value
    if (!key || !PRESETS[key]) return
    this.steps = PRESETS[key].steps.map(s => ({ ...s }))
    this.renderCollapsed()
    this.renderExpanded()
    // Auto-expand when loading a preset
    this.expandedBodyTarget.style.display = ""
    this.applyToggleState()
    e.target.value = ""
  }

  // ── Step CRUD ────────────────────────────────────────────────
  addStep() {
    this.steps.push({ name: "", task: "", agent: "auto", input_keys: "" })
    this.renderCollapsed()
    this.renderExpanded()
    // Auto-expand when adding a step
    this.expandedBodyTarget.style.display = ""
    this.applyToggleState()
    // Scroll to the new step
    requestAnimationFrame(() => {
      const cards = this.stepsContainerTarget.querySelectorAll("[data-step-card]")
      if (cards.length > 0) {
        cards[cards.length - 1].scrollIntoView({ behavior: "smooth", block: "nearest" })
      }
    })
  }

  removeStep(e) {
    const idx = parseInt(e.params.index, 10)
    this.steps.splice(idx, 1)
    this.renderCollapsed()
    this.renderExpanded()
  }

  moveUp(e) {
    const idx = parseInt(e.params.index, 10)
    if (idx <= 0) return
    const tmp = this.steps[idx]
    this.steps[idx] = this.steps[idx - 1]
    this.steps[idx - 1] = tmp
    this.renderCollapsed()
    this.renderExpanded()
  }

  moveDown(e) {
    const idx = parseInt(e.params.index, 10)
    if (idx >= this.steps.length - 1) return
    const tmp = this.steps[idx]
    this.steps[idx] = this.steps[idx + 1]
    this.steps[idx + 1] = tmp
    this.renderCollapsed()
    this.renderExpanded()
  }

  // ── Field change handler ─────────────────────────────────────
  updateField(e) {
    const card = e.target.closest("[data-step-index]")
    if (!card) return
    const idx = parseInt(card.dataset.stepIndex, 10)
    const field = e.target.dataset.stepField
    if (field && idx >= 0 && idx < this.steps.length) {
      this.steps[idx][field] = e.target.value
      this.renderCollapsed()
    }
  }

  // ── Render: collapsed summary ────────────────────────────────
  renderCollapsed() {
    if (!this.hasCollapsedInfoTarget) return

    if (this.steps.length === 0) {
      this.collapsedInfoTarget.innerHTML = `<span class="text-text-dim text-xs italic">No pipeline steps yet</span>`
    } else {
      const names = this.steps.map((s, i) =>
        `<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-mono bg-bg-surface/50 border border-border-faint text-text-muted">
          ${i + 1}. ${s.name || "unnamed"}
        </span>`
      ).join('<span class="text-text-dim mx-1">→</span>')
      this.collapsedInfoTarget.innerHTML = names
    }
  }

  // ── Render: expanded step cards ──────────────────────────────
  renderExpanded() {
    if (!this.hasStepsContainerTarget) return

    if (this.steps.length === 0) {
      this.stepsContainerTarget.innerHTML = `
        <div class="text-center py-6 text-text-dim border-2 border-dashed border-border-faint rounded-xl">
          <p class="text-sm font-mono">No steps defined</p>
          <p class="text-xs mt-1">Click <span class="text-neon-purple">"Add Step"</span> to begin building your pipeline</p>
        </div>
      `
      return
    }

    this.stepsContainerTarget.innerHTML = this.steps.map((step, i) => `
      <div data-step-card data-step-index="${i}"
           class="step-card rounded-xl bg-bg-dark/60 border border-border-faint p-4 transition-all duration-200 hover:border-neon-purple/40">

        <!-- Step header with controls -->
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-neon-purple/20 border border-neon-purple/30 flex items-center justify-center text-xs font-mono text-neon-purple font-bold">${i + 1}</span>
            <span class="text-xs text-text-muted font-mono uppercase tracking-wider">Step</span>
          </div>
          <div class="flex items-center gap-1">
            <button type="button" data-action="pipeline-builder#moveUp" data-pipeline-builder-index-param="${i}"
                    ${i === 0 ? 'disabled' : ''}
                    class="p-1.5 rounded-lg text-text-muted hover:text-neon-cyan hover:bg-neon-cyan/10 transition-colors ${i === 0 ? 'opacity-30 cursor-not-allowed' : 'cursor-pointer'}"
                    title="Move up">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>
            </button>
            <button type="button" data-action="pipeline-builder#moveDown" data-pipeline-builder-index-param="${i}"
                    ${i === this.steps.length - 1 ? 'disabled' : ''}
                    class="p-1.5 rounded-lg text-text-muted hover:text-neon-cyan hover:bg-neon-cyan/10 transition-colors ${i === this.steps.length - 1 ? 'opacity-30 cursor-not-allowed' : 'cursor-pointer'}"
                    title="Move down">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
            </button>
            <button type="button" data-action="pipeline-builder#removeStep" data-pipeline-builder-index-param="${i}"
                    class="p-1.5 rounded-lg text-text-muted hover:text-neon-red hover:bg-neon-red/10 transition-colors cursor-pointer"
                    title="Remove step">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
            </button>
          </div>
        </div>

        <!-- Fields grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <!-- Name -->
          <div>
            <label class="block text-xs font-mono text-text-muted mb-1 uppercase tracking-wider">Name</label>
            <input type="text" value="${this.#escapeHtml(step.name)}"
                   data-step-field="name"
                   data-action="input->pipeline-builder#updateField"
                   placeholder="step name"
                   class="w-full rounded-lg bg-bg-dark border border-border-faint px-3 py-1.5 text-sm font-mono text-text-primary placeholder-text-dim focus:border-neon-purple focus:outline-none focus:ring-1 focus:ring-neon-purple/50 transition-colors" />
          </div>
          <!-- Agent -->
          <div>
            <label class="block text-xs font-mono text-text-muted mb-1 uppercase tracking-wider">Agent</label>
            <select data-step-field="agent"
                    data-action="change->pipeline-builder#updateField"
                    class="w-full rounded-lg bg-bg-dark border border-border-faint px-3 py-1.5 text-sm font-mono text-text-primary focus:border-neon-purple focus:outline-none focus:ring-1 focus:ring-neon-purple/50 transition-colors">
              ${AGENTS.map(a => `<option value="${a}" ${a === step.agent ? 'selected' : ''}>${a}</option>`).join("")}
            </select>
          </div>
          <!-- Task (spans full width) -->
          <div class="sm:col-span-2">
            <label class="block text-xs font-mono text-text-muted mb-1 uppercase tracking-wider">Task</label>
            <textarea rows="2"
                      data-step-field="task"
                      data-action="input->pipeline-builder#updateField"
                      placeholder="Describe what this step should do..."
                      class="w-full rounded-lg bg-bg-dark border border-border-faint px-3 py-1.5 text-sm font-mono text-text-primary placeholder-text-dim focus:border-neon-purple focus:outline-none focus:ring-1 focus:ring-neon-purple/50 resize-none transition-colors">${this.#escapeHtml(step.task)}</textarea>
          </div>
          <!-- Input Keys -->
          <div class="sm:col-span-2">
            <label class="block text-xs font-mono text-text-muted mb-1 uppercase tracking-wider">Input Keys <span class="text-text-dim normal-case">(comma-separated)</span></label>
            <input type="text" value="${this.#escapeHtml(step.input_keys)}"
                   data-step-field="input_keys"
                   data-action="input->pipeline-builder#updateField"
                   placeholder="pr_diff, spec, topic"
                   class="w-full rounded-lg bg-bg-dark border border-border-faint px-3 py-1.5 text-sm font-mono text-text-primary placeholder-text-dim focus:border-neon-purple focus:outline-none focus:ring-1 focus:ring-neon-purple/50 transition-colors" />
          </div>
        </div>
      </div>
    `).join("")
  }

  // ── Serialize to TOML & submit ──────────────────────────────
  submitPipeline(e) {
    e.preventDefault()

    const validSteps = this.steps.filter(s => s.task.trim().length > 0)
    if (validSteps.length === 0) {
      alert("Please add at least one step with a task description before running the pipeline.")
      return
    }

    const toml = this.#serializeToml(validSteps)
    this.hiddenTomlTarget.value = toml
    this.element.querySelector("#pipeline-form")?.requestSubmit()
  }

  #serializeToml(steps) {
    const lines = []
    for (const step of steps) {
      lines.push("[[steps]]")
      lines.push(`name = "${this.#escapeToml(step.name || "")}"`)
      lines.push(`task = """${step.task}"""`)
      lines.push(`agent = "${step.agent || "auto"}"`)
      lines.push(`input_keys = "${this.#escapeToml(step.input_keys || "")}"`)
      lines.push("")
    }
    return lines.join("\n")
  }

  #escapeToml(str) {
    return str.replace(/\\/g, "\\\\").replace(/"/g, '\\"').replace(/\n/g, "\\n")
  }

  #escapeHtml(str) {
    const div = document.createElement("div")
    div.appendChild(document.createTextNode(str))
    return div.innerHTML
  }
}
