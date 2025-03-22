<script lang="ts">
  import type { UnlistenFn } from '@tauri-apps/api/event'
  import { onMount, type Snippet } from 'svelte'

  import { listenAll } from '$lib/events.svelte'
  import '../app.css'

  interface Props {
    children?: Snippet
  }

  const { children }: Props = $props()

  onMount(() => {
    let unlistenFns: UnlistenFn[] | undefined
    listenAll()
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
