/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        trade: {
          gold: "#FFD700",
          dark: "#0a0e17",
          panel: "#111827",
          accent: "#10B981",
          danger: "#EF4444",
          blue: "#3B82F6",
        },
      },
    },
  },
  plugins: [],
};
