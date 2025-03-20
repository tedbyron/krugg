import { writable } from 'svelte/store'

import type { LockFile } from 'tauri-plugin-lcu-api'

export const lcuConnected = writable(false)
export const lcuLockFile = writable<LockFile | undefined>()
export const lcuBaseUrl = writable<string | undefined>()
