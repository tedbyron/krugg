import { Store } from '@tauri-apps/plugin-store'

export interface AppData {
  lockfile_path: string

  theme: 'system' | 'light' | 'dark'
}

const stores = $state<{
  appData: Store | null
}>({
  appData: null,
})

export const appData = async () => {
  stores.appData ??= await Store.load('app_data.json')
  return stores.appData
}
