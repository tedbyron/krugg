<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'

  onMount(() => {
    invoke('show_main_window').catch(console.error)
    invoke('league_client_process').catch(console.error)
  })
</script>
