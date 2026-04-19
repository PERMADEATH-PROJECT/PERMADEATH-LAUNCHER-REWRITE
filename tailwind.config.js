/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,ts}'],
  theme: {
    extend: {
      colors: {
        'brand-red':      '#f12b43',
        'brand-red-soft': '#ff758f',
        'brand-cyan':     '#21e6c1',
        'brand-purple':   '#a259f7',
        'brand-yellow':   '#ffe151',
        'brand-blue':     '#5bc0eb',
        'brand-gray':     '#b9b7c3',
        'sidebar':        '#0a0a0d',
        'sidebar-btn':    '#151419',
        'sidebar-btn-hover': '#28222b',
      },
      fontFamily: {
        sans: ['Inter', 'Segoe UI', 'Arial', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
