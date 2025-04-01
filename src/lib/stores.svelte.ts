import { Store } from '@tauri-apps/plugin-store'
import type { Theme } from '$lib'

export const appState = $state<{
  theme: Theme
}>({
  theme: 'system',
})

export type AppData = Partial<{
  lockfile_path: string
  theme: Theme
}>

const stores = $state<{
  appData: Store | null
}>({
  appData: null,
})

export const loadAppData = async () => {
  stores.appData ??= await Store.load('app_data.json')
  return stores.appData
}
