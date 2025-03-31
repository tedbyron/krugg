import { sveltekit } from '@sveltejs/kit/vite'
import autoprefixer from 'autoprefixer'
import cssnano from 'cssnano'
import advancedPreset from 'cssnano-preset-advanced'
import tailwindcss from 'tailwindcss'
import icons from 'unplugin-icons/vite'
import { defineConfig, searchForWorkspaceRoot } from 'vite'
import { ViteImageOptimizer } from 'vite-plugin-image-optimizer'

export default defineConfig(({ mode }) => {
  const dev = mode === 'development'
  const host = process.env.TAURI_DEV_HOST

  return {
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host ?? false,
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ['**/crate/**'],
      },
      fs: {
        allow: [searchForWorkspaceRoot(import.meta.dirname), 'tauri-plugin-lcu/dist-js'],
      },
    },
    css: {
      postcss: {
        plugins: [
          tailwindcss(),
          autoprefixer(),
          ...(dev
            ? []
            : [
                cssnano({
                  preset: advancedPreset({
                    convertValues: { length: true },
                    discardComments: { removeAll: true },
                  }),
                }),
              ]),
        ],
      },
    },
    esbuild: { drop: dev ? undefined : ['console', 'debugger'] },
    plugins: [
      sveltekit(),
      icons({
        autoInstall: true,
        compiler: 'svelte',
        // Font size is 0.875rem (14pt), but we want icons to be 1rem.
        scale: parseFloat((1 / 0.875).toFixed(2)),
      }),
      ViteImageOptimizer({
        logStats: false,
        svg: {
          plugins: [
            {
              name: 'preset-default',
              params: {
                overrides: {
                  removeViewBox: false,
                },
              },
            },
            {
              name: 'addAttributesToSVGElement',
              params: {
                attributes: [{ xmlns: 'http://www.w3.org/2000/svg' }],
              },
            },
          ],
        },
      }),
    ],
  }
})
