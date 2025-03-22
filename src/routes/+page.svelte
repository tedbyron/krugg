<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'

  onMount(() => {
    invoke('show_main_window').catch(console.error)
  })
</script>

<a href="/champions">Champions</a>
