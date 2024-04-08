<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrent } from "@tauri-apps/api/window";
  import { open, type FileResponse } from "@tauri-apps/plugin-dialog";
  import { Circle } from "svelte-loading-spinners";

  getCurrent().setTitle("Marketing A/B");

  let fileResponse: FileResponse | null = null;
  let response: String = "";
  let text: String = "";
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

  const generate_prompt_example = async (paragraph: string) => {
    isLoading = true;

    const short = text.substring(0, 1000);
    const prompt = `"Your task is to create a prompt to generate an image for this action item:\n\n${paragraph}\n\nMarketing brief:\n${short}...\n\nResult:\n\n"`;

    const paragraphResponse = await invoke("get_answer", { prompt });
    isLoading = false;

    response =
      paragraphResponse + "\n\n========================\n\n" + response;
  };

  const generate_budget = async () => {
    isLoading = true;

    const short = text.substring(0, 1000);
    const prompt = `"Generate a budget breakdown for the marketing campaign:\n\nMarketing brief:\n${short}...\n\nBreakdown:\n\n $"`;

    const result = await invoke("get_answer", { prompt });
    isLoading = false;

    response = result + "\n\n========================\n\n" + response;
  };
</script>

<main class=" flex flex-col mx-auto w-screen">
  {#if !isLoading}
    <div class="flex flex-row w-screen justify-center mt-2 space-x-2">
      <button on:click={() => openFile()}>Open brief</button>
      <button on:click={() => generate_budget()}>Get budget</button>
    </div>
  {/if}

  {#if fileResponse}
    <div class="flex flex-row w-screen justify-center">
      Uploaded file: {fileResponse.name}
    </div>
  {/if}

  {#if isLoading}
    <div class="flex flex-row justify-center items-center m-4">
      <Circle size="60" color="#FF3E00" unit="px" duration="1s" />
    </div>
  {/if}

  <!-- <pre>{response}</pre> -->
  {#if response.length > 0}
    <div class="flex flex-col justify-center">
      {#each response
        .split("\n\n")
        .filter((a) => a.length > 0) as paragraph, index}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          class="hover:bg-blue-100 flex flex-row justify-center"
          on:click={() => generate_prompt_example(paragraph)}
        >
          {paragraph}
        </div>
      {/each}
    </div>
  {/if}
</main>
