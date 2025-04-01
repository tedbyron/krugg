import { error as svelteError } from '@sveltejs/kit'

export * from './channel.svelte'
export * from './commands'
export { lcu } from './events.svelte'
export * from './stores.svelte'

export const error = {
  /** `500`, Failed to load champion data. */
  getChampions: () => svelteError(500, 'Failed to load champion data'),
} as const

export const themes = ['system', 'light', 'dark'] as const
export type Theme = (typeof themes)[number]

export const placeholderImgSrc =
  'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVQI12NgYAAAAAMAASDVlMcAAAAASUVORK5CYII'
