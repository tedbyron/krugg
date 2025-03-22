import { error } from '@sveltejs/kit'
import { invoke } from '@tauri-apps/api/core'

import { champions, newChannel } from '$lib'
import type { PageLoad } from './$types'

export const load: PageLoad = async () => {
  const channel = newChannel(({ type, data }) => {
    if (type === 'champions') {
      champions.set(data)
    }
  })

  try {
    await invoke<null>('get_champions', { channel })
  } catch (err) {
    console.error(err)
    error(500, 'Failed to load champion data')
  }
}
