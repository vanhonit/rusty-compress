/** @type {import('tailwindcss').Config} */

// macOS-style semantic palette. Values live as RGB channels in CSS variables
// (src/index.css) so they switch with the system Light/Dark appearance and
// still support Tailwind opacity modifiers, e.g. `bg-accent/10`.
const token = (name) => `rgb(var(--${name}) / <alpha-value>)`;

export default {
  content: [
    './src/**/*.{js,jsx,ts,tsx}',
    './index.html',
  ],
  darkMode: 'media',
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: token('surface'),     // content background (file list, modals)
          secondary: token('surface-2'), // toolbar, tab bar, headers, footers
          tertiary: token('surface-3'),  // hover fills, inputs, chips
        },
        separator: token('separator'),   // borders and dividers
        fg: {
          DEFAULT: token('fg'),          // primary text
          secondary: token('fg-2'),      // secondary text, icons
          tertiary: token('fg-3'),       // placeholders, disabled
        },
        accent: {
          DEFAULT: token('accent'),      // the one blue accent
          hover: token('accent-hover'),
          fg: token('accent-fg'),        // text on accent
        },
        danger: token('danger'),
        success: token('success'),
        warning: token('warning'),
      },
      borderRadius: {
        mac: '6px',
      },
      boxShadow: {
        window: '0 0 0 0.5px rgb(0 0 0 / 0.12), 0 12px 32px rgb(0 0 0 / 0.18)',
      },
    },
  },
  plugins: [],
}
