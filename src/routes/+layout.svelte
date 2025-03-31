<script lang="ts">
  import type { UnlistenFn } from '@tauri-apps/api/event'
  import { onMount, type Snippet } from 'svelte'

  import Header from '$components/Header.svelte'
  import { getOrInitChannel } from '$lib'
  import { listenAll } from '$lib/events.svelte'
  import '../app.css'

  interface Props {
    children?: Snippet
  }

  const { children }: Props = $props()

  onMount(() => {
    let unlistenFns: UnlistenFn[] = []
    listenAll()
      .then((fns) => {
        unlistenFns = fns
      })
      .catch(console.error)

    getOrInitChannel()

    return () => {
      for (const f of unlistenFns) {
        f()
      }
    }
  })
</script>

<Header />

<main class="overflow-auto py-2">
  {@render children?.()}
</main>
