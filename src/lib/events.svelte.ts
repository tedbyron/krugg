import { listen } from '@tauri-apps/api/event'
import type { LockFile } from 'tauri-plugin-lcu-api'

export const lcu = $state<{
  connected: boolean
  lockFile: LockFile | null
  baseUrl: string | null
}>({
  connected: false,
  lockFile: null,
  baseUrl: null,
})

export const listenAll = () =>
  Promise.all([
    listen<boolean>('lcu-connected', (event) => {
      lcu.connected = event.payload
    }),
    listen<LockFile>('lcu-lockfile', (event) => {
      lcu.lockFile = event.payload
    }),
    listen<string>('lcu-base-url', (event) => {
      lcu.baseUrl = event.payload
    }),
  ])
