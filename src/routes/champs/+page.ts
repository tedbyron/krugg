import { api, error, getChampions } from '$lib'
import type { PageLoad } from './$types'

export const load: PageLoad = async () => {
  if (api.champs === undefined) {
    try {
      await getChampions()
    } catch (err) {
      console.error(err)
      error.getChampions()
    }
  }

  return {
    title: 'Champions',
  }
}
