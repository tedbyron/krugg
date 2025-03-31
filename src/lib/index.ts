import { error as svelteError } from '@sveltejs/kit'

export * from './channel.svelte'
export { lcu } from './events.svelte'
export * from './stores.svelte'

export const error = {
  getChampions: () => svelteError(500, 'Failed to load champion data'),
} as const
