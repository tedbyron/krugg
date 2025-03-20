<script lang="ts">
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import { onMount, type Snippet } from 'svelte'

  import { lcuConnected, lcuBaseUrl, lcuLockFile } from '$lib'
  import '../app.css'

  import type { LockFile } from 'tauri-plugin-lcu-api'

  interface Props {
    children?: Snippet
  }

  const { children }: Props = $props()

  const setup = async () => [
    await listen<LockFile>('lcu-lockfile', (event) => {
      $lcuLockFile = event.payload
    }),
    await listen<string>('lcu-base-url', (event) => {
      $lcuBaseUrl = event.payload
    }),
    await listen('lcu-connected', () => {
      $lcuConnected = true
    }),
    await listen('lcu-disconnected', () => {
      $lcuConnected = false
    }),
  ]

  onMount(() => {
    let unlistenFns: UnlistenFn[] | undefined
    setup()
      .then((fns) => {
        unlistenFns = fns
      })
      .catch(console.error)

    return () => {
      if (unlistenFns !== undefined) {
        for (const f of unlistenFns) {
          f()
        }
      }
    }
  })
</script>

<main>
  {@render children?.()}
</main>
