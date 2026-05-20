import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { url: String }

  connect() {
    this.loadChart()
  }

  disconnect() {
    if (this.chart) {
      this.chart.destroy()
    }
  }

  async loadChart() {
    if (typeof Chart === "undefined") {
      await this.loadChartJs()
    }

    const resp = await fetch(this.urlValue, { headers: { Accept: "application/json" } })
    const data = await resp.json()
    this.renderChart(data)
  }

  loadChartJs() {
    return new Promise((resolve, reject) => {
      if (typeof Chart !== "undefined") { resolve(); return }
      const script = document.createElement("script")
      script.src = "https://cdn.jsdelivr.net/npm/chart.js@4.4.7/dist/chart.umd.min.js"
      script.onload = resolve
      script.onerror = reject
      document.head.appendChild(script)
    })
  }

  renderChart(data) {
    const canvas = this.element.querySelector("canvas")
    if (!canvas) return

    const labels = data.map(d => d.date)
    const sizes = data.map(d => d.size)
    const ids = data.map(d => d.id)
    const repoNames = data.map(d => d.repo_name)

    const ctx = canvas.getContext("2d")
    this.chart = new Chart(ctx, {
      type: "bar",
      data: {
        labels,
        datasets: [{
          label: "Backup Size",
          data: sizes,
          backgroundColor: "rgba(0, 240, 255, 0.8)",
          hoverBackgroundColor: "rgba(255, 0, 255, 0.9)",
          borderColor: "rgba(0, 240, 255, 0.6)",
          borderWidth: 1,
          borderRadius: 4,
          barPercentage: 0.7
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        onClick: (event, elements) => {
          if (elements.length > 0) {
            const idx = elements[0].index
            const date = labels[idx]
            window.location.href = `/anvil/backups?date=${date}`
          }
        },
        plugins: {
          legend: { display: false },
          tooltip: {
            backgroundColor: "rgba(26, 10, 46, 0.95)",
            borderColor: "rgba(0, 240, 255, 0.3)",
            borderWidth: 1,
            titleColor: "#00f0ff",
            bodyColor: "#e0e0e0",
            titleFont: { family: "monospace", size: 12 },
            bodyFont: { family: "monospace", size: 11 },
            padding: 10,
            callbacks: {
              title: (items) => items[0].label,
              label: (item) => {
                const idx = item.dataIndex
                return ` ${this.formatBytes(sizes[idx])} · ${repoNames[idx]}`
              }
            }
          }
        },
        scales: {
          x: {
            grid: { color: "rgba(0, 240, 255, 0.08)", drawBorder: false },
            ticks: {
              color: "#e0e0e0",
              font: { family: "monospace", size: 10 },
              maxRotation: 45,
              callback: function(value) {
                const label = this.getLabelForValue(value)
                return label ? label.slice(5) : label
              }
            }
          },
          y: {
            grid: { color: "rgba(0, 240, 255, 0.08)", drawBorder: false },
            ticks: {
              color: "#e0e0e0",
              font: { family: "monospace", size: 10 },
              callback: (value) => this.formatBytes(value)
            },
            beginAtZero: true
          }
        }
      }
    })
  }

  formatBytes(bytes) {
    if (bytes === 0) return "0 B"
    const units = ["B", "KB", "MB", "GB", "TB"]
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    const value = (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)
    return `${value} ${units[i]}`
  }
}
