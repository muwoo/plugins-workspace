<script>
  import { open } from '@tauri-apps/plugin-opener'

  export let onMessage

  const programs = [
    'firefox',
    'google chrome',
    'chromium',
    'safari',
    'open',
    'start',
    'xdg-open',
    'gio',
    'gnome-open',
    'kde-open',
    'wslview'
  ]

  let path = ''

  let program = 'Default'

  function openPath() {
    open(path, program === 'Default' ? undefined : program).catch(onMessage)
  }
</script>

<div class="flex flex-col">
  <div class="flex flex-row gap-2 items-center">
    <input
      class="input grow"
      placeholder="Type the path to watch..."
      bind:value={path}
    />

    <span> with </span>
    <select class="input" bind:value={program}>
      <option value="Default">Default</option>
      {#each programs as p}
        <option value={p}>{p}</option>
      {/each}
    </select>

    <button class="btn" on:click={openPath}>Open</button>
  </div>
</div>
