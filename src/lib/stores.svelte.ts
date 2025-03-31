import { Store } from '@tauri-apps/plugin-store'

const stores = $state<{
  appData: Store | null
}>({
  appData: null,
})

export const loadAppData = async () => {
  stores.appData ??= await Store.load('app_data.json')
  return stores.appData
}
