import { listen, type EventCallback } from '@tauri-apps/api/event'
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

const applyPayload = <K extends keyof typeof lcu>(key: K) => {
  return (({ payload }) => {
    lcu[key] = payload
  }) satisfies EventCallback<(typeof lcu)[K]>
}

export const listenAll = () =>
  Promise.all([
    listen<boolean>('lcu-connected', applyPayload('connected')),
    listen<LockFile>('lcu-lockfile', applyPayload('lockFile')),
    listen<string>('lcu-base-url', applyPayload('baseUrl')),
  ])
