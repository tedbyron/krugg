import { listen } from '@tauri-apps/api/event'
import type { LockFile } from 'tauri-plugin-lcu-api'

export const lcuState = $state<{
  connected: boolean
  lockFile: LockFile | null
  baseUrl: string | null
}>({
  connected: false,
  lockFile: null,
  baseUrl: null,
})

export const listenAll = async () => [
  await listen<boolean>('lcu-connected', (event) => {
    lcuState.connected = event.payload
  }),
  await listen<LockFile>('lcu-lockfile', (event) => {
    lcuState.lockFile = event.payload
  }),
  await listen<string>('lcu-base-url', (event) => {
    lcuState.baseUrl = event.payload
  }),
]
