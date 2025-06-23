/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.html",
    "./src/**/*.rs"
  ],
  theme: {
    extend: {
      fontFamily: {
        'sans': [
          'ui-sans-serif', 
          'system-ui', 
          '-apple-system', 
          'BlinkMacSystemFont', 
          '"Segoe UI"', 
          'Roboto', 
          '"Helvetica Neue"', 
          'Arial', 
          '"Noto Sans"', 
          'sans-serif', 
          '"Apple Color Emoji"', 
          '"Segoe UI Emoji"', 
          '"Segoe UI Symbol"', 
          '"Noto Color Emoji"'
        ],
      },
      fontSize: {
        '2xs': ['0.65rem', { lineHeight: '1rem' }],
      },
      colors: {
        'primary': {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0ea5e9',
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
        },
        'dark': {
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          300: '#cbd5e1',
          400: '#94a3b8',
          500: '#64748b',
          600: '#475569',
          700: '#334155',
          800: '#1e293b',
          850: '#172033',
          900: '#0f172a',
          950: '#020617',
        }
      },
      aspectRatio: {
        'book': '2 / 3',
      },
      gridTemplateColumns: {
        '53': 'repeat(53, minmax(0, 1fr))'
      },
      // Calendar specific variables
      calendar: {
        darkBg: '#0f172a',          // dark.900
        darkBorder: '#334155',      // dark.700
        darkAccent: '#475569',      // dark.600
        darkButtonBg: '#1e293b',    // dark.800
        buttonActive: '#0284c7',    // primary.600
        buttonActiveBorder: '#0369a1', // primary.700
        primaryLighter: 'rgba(14, 165, 233, 0.1)', // primary.500 with opacity
        primaryLight: '#38bdf8',    // primary.400
        primary: '#0284c7',         // primary.600
      }
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
    require('@tailwindcss/line-clamp'),
  ],
} 