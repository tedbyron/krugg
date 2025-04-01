import { invoke } from '@tauri-apps/api/core'

import { newChannel } from '$lib'

export const showMainWindow = async () => {
  await invoke<null>('show_main_window')
}

export const getChampions = async () => {
  await invoke<null>('get_champions', { channel: newChannel() })
}

export const getChampion = async (id: string) => {
  await invoke<null>('get_champion', { channel: newChannel(), id })
}
