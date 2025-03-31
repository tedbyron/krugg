<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { onMount } from 'svelte'

  onMount(async () => {
    await invoke('show_main_window')
  })
</script>

<a href="/champs">Champions</a>
