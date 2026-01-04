/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./templates/**/*.html",
    "./src/**/*.rs",
    "./assets/ts/**/*.ts"
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
          50: '#f9fafb',
          100: '#f3f4f6',
          200: '#e5e7eb',
          300: '#d1d5db',
          400: '#9ca3af',
          500: '#6b7280',
          600: '#4b5563',
          700: '#374151',
          800: '#1f2937',
          850: '#1a222e',
          900: '#111827',
          925: '#0a0f1a',
          950: '#030712',
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
        darkBg: '#111827',          // dark.900
        darkBorder: '#374151',      // dark.700
        darkAccent: '#4b5563',      // dark.600
        darkButtonBg: '#1f2937',    // dark.800
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
    require('@tailwindcss/container-queries'),
  ],
} 