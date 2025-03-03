import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  compilerOptions: { runes: true },
  kit: {
    adapter: adapter(),
    alias: { $components: 'src/lib/components' },
    typescript: {
      config: (config) => ({
        ...config,
        include: [
          // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
          ...config.include,
          '../eslint.config.js',
          '../svelte.config.js',
          '../tailwind.config.ts',
        ],
      }),
    },
  },
  preprocess: vitePreprocess(),
}
