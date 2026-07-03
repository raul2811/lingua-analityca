module.exports = {
  content: ["./templates/**/*.html"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        brand: {
          blue: "#3b6ff5",
          hoverBlue: "#2557d6",
          deepBlue: "#0f1c3f",
          bg: "#0b1326",
          card: "#161f30",
          deep: "#060e20",
          border: "#2a3547",
          text: "#f8fafc",
          muted: "#94a3b8",
        },
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "-apple-system", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "monospace"],
      },
      animation: {
        "fade-in": "fadeIn 0.25s ease-out both",
        "slide-up": "slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1) both",
        "spin-slow": "spin 1s linear infinite",
      },
      keyframes: {
        fadeIn: { "0%": { opacity: "0" }, "100%": { opacity: "1" } },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(12px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
    },
  },
};
