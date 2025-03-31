import { error as svelteError } from '@sveltejs/kit'

export type * from './channel.svelte'
export { api, getOrInitChannel } from './channel.svelte'
export { lcu } from './events.svelte'
export { loadAppData } from './stores.svelte'

export const error = {
  getChampions: () => svelteError(500, 'Failed to load champion data'),
} as const
