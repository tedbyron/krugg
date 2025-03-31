import { api } from '$lib'
import type { PageLoad } from './$types'

export const load: PageLoad = ({ params }) => {
  const { champName } = params

  return {
    champ: api.champs?.find(({ name }) => name === champName),
  }
}
