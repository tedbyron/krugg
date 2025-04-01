<script lang="ts">
  import { onMount } from 'svelte'

  import { showMainWindow } from '$lib'

  onMount(async () => {
    await showMainWindow()
  })
</script>

<a href="/champs">Champions</a>
