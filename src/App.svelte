<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrent } from "@tauri-apps/api/window";
  import { open, type FileResponse } from "@tauri-apps/plugin-dialog";

  getCurrent().setTitle("Marketing A/B");

  let fileResponse: FileResponse | null = null;

  const openFile = async () => {
    // Open a dialog
    fileResponse = await open({
      multiple: false,
      directory: false,
    });

    if (fileResponse) {
      const { path } = fileResponse;
      const response = await invoke("get_pdf", { path });
    }
  };
</script>

<main class="container">
  <h1>Marketing A/B</h1>

  {#if !fileResponse}
    <button on:click={() => openFile()}>Open brief</button>
  {:else}
    <div>Uploaded file: {fileResponse.name}</div>
  {/if}
</main>
