/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Echoes the Aegis shield logo (emerald/teal gradient).
        brand: {
          DEFAULT: "#10b981",
          deep: "#0d9488",
          glow: "#2dd4bf",
        },
      },
    },
  },
  plugins: [],
};
