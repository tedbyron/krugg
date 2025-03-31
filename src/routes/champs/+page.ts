import { invoke } from '@tauri-apps/api/core'

import { api, error, getOrInitChannel } from '$lib'
import type { PageLoad } from './$types'

export const load: PageLoad = async () => {
  if (api.champs === undefined) {
    try {
      await invoke<null>('get_champions', { channel: getOrInitChannel() })
    } catch (err) {
      console.error(err)
      error.getChampions()
    }
  }
}
