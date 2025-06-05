/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#ecf5ff',
          100: '#d9ecff',
          200: '#c6e2ff',
          300: '#a0cfff',
          400: '#79bbff',
          500: '#409eff',
          600: '#337ecc',
          700: '#2b6cb0',
          800: '#1e40af',
          900: '#1e3a8a',
        }
      }
    },
  },
  plugins: [],
}
