<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrent } from "@tauri-apps/api/window";
  import { open, type FileResponse } from "@tauri-apps/plugin-dialog";
  import { Circle } from "svelte-loading-spinners";

  getCurrent().setTitle("Marketing A/B");

  let fileResponse: FileResponse | null = null;
  let response: String = "";
  let text:String = "";
  let isLoading = false;

  const openFile = async () => {
    // Open a dialog
    fileResponse = await open({
      multiple: false,
      directory: false,
    });

    if (fileResponse) {
      const { path } = fileResponse;
      text = await invoke("get_pdf", { path });
      isLoading = true;

      const short = text.substring(0, 1000); // cut it short for the context length of small model
      // const prompt = `"Your task is to create a list of actions for the strategy:\n\nMarketing brief:\n${short}...\n\nStrategy:\n\n"`;
      const prompt = `"Your task is to create a list of actions for the strategy:\n\nMarketing brief:\n${short}...\n\nStrategy:\n\n"`;

      response = await invoke("get_answer", { prompt });
      isLoading = false;
    }
  };

  const generate_prompt_example = (text:string)
</script>

<main class="container">
  <h1>Marketing A/B</h1>

  {#if !isLoading}
    <button on:click={() => openFile()}>Open brief</button>
  {/if}

  {#if fileResponse}
    <div>Uploaded file: {fileResponse.name}</div>
  {/if}

  <pre>{response}</pre>

  {#if isLoading}
    <div class="centered">
      <Circle size="60" color="#FF3E00" unit="px" duration="1s" />
    </div>
  {/if}
</main>

<style>
  .centered {
    display: flex;
    flex-direction: row;
    justify-content: center;
    justify-items: center;
  }
</style>
