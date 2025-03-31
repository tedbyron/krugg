import type { Config } from 'tailwindcss'

export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'selector',
  theme: {
    extend: {
      colors: {
        'gruvbox-dark': {
          bg: '#282828',
          red: '#cc241d',
          green: '#98971a',
          yellow: '#d79921',
          blue: '#458588',
          purple: '#b16286',
          aqua: '#689d6a',
          gray: '#a89984',

          gray2: '#928374',
          red2: '#fb4934',
          green2: '#b8bb26',
          yellow2: '#fabd2f',
          blue2: '#83a598',
          purple2: '#d3869b',
          aqua2: '#8ec07c',
          fg: '#ebdbb2',

          'bg-h': '#1d2021',
          bg1: '#3c3836',
          bg2: '#504945',
          bg3: '#665c54',
          bg4: '#7c6f64',
          orange: '#d65d0e',

          'bg-s': '#32302f',
          fg4: '#a89984',
          fg3: '#bdae93',
          fg2: '#d5c4a1',
          fg1: '#ebdbb2',
          fg0: '#fbf1c7',
          orange2: '#fe8019',
        },
        gruvbox: {
          bg: '#fbf1c7',
          red: '#cc241d',
          green: '#98971a',
          yellow: '#d79921',
          blue: '#458588',
          purple: '#b16286',
          aqua: '#689d6a',
          gray: '#7c6f64',

          gray2: '#928374',
          red2: '#9d0006',
          green2: '#79740e',
          yellow2: '#b57614',
          blue2: '#076678',
          purple2: '#8f3f71',
          aqua2: '#427b58',
          fg: '#3c3836',

          'bg-h': '#f9f5d7',
          bg1: '#ebdbb2',
          bg2: '#d5c4a1',
          bg3: '#bdae93',
          bg4: '#a89984',
          orange: '#d65d0e',

          'bg-s': '#f2e5bc',
          fg4: '#7c6f64',
          fg3: '#665c54',
          fg2: '#504945',
          fg1: '#3c3836',
          fg0: '#282828',
          orange2: '#af3a03',
        },
      },
      container: {
        center: true,
      },
    },
  },
} satisfies Config
