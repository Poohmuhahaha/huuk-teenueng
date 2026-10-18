// Centralized Tailwind CSS design tokens per Master Engineering SOP §2.1.
// In Tailwind v4 (@tailwindcss/vite), CSS variables in src/app/style.css (@theme)
// mirror this canonical token registry so utility classes never require raw
// arbitrary bracket values.
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f7f7f6',
          100: '#ededeb',
          500: '#555555',
          600: '#222222',
          700: '#111111',
        },
        surface: {
          DEFAULT: 'var(--bg)',
          muted: 'var(--bg-soft)',
          subtle: 'var(--surface-2)',
          dark: '#0f172a',
        },
        ink: {
          DEFAULT: 'var(--ink)',
          soft: 'var(--ink-soft)',
          muted: 'var(--muted)',
        },
        danger: 'var(--danger)',
        warning: 'var(--warn)',
        success: 'var(--ok)',
      },
      fontFamily: {
        sans: ['var(--font-sans)', 'system-ui', 'sans-serif'],
        mono: ['var(--font-mono)', 'monospace'],
      },
      fontSize: {
        '2xs': ['0.6875rem', { lineHeight: '1rem' }],
        'xs-plus': ['0.8125rem', { lineHeight: '1.25rem' }],
        'sm-plus': ['0.9375rem', { lineHeight: '1.375rem' }],
      },
      spacing: {
        touch: '2.75rem',
      },
      borderRadius: {
        card: 'var(--radius, 12px)',
      },
    },
  },
  plugins: [],
}
