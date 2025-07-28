<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';  // <-- comes from plugin now!
  import { listen, emitTo } from '@tauri-apps/api/event';
  import { loadThumbnails } from '../lib/pdfRenderer.js';

  let pdfPath = '';
  let pageInput = 1;
  let pdfDoc = null;
  let thumbnails;
  let currentPage = 1;

  async function selectPDF() {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: 'PDF Files', extensions: ['pdf'] }]
    });
    if (selected) {
      openPDF(selected);
    }
  }

  async function openPDF(selected){
      await closePresentation();
      pdfPath = selected;

      thumbnails = await loadThumbnails(pdfPath);

      await invoke('set_current_pdf_path', { pdfPath });
      await startPresentation();
  }

  async function closePresentation() {
    try {
      await invoke('close_singleview');
    } catch (err) {
      console.error('Failed to close singleview:', err);
    }
  }

  async function startPresentation() {
    if (!pdfPath) {
      alert('Please select a PDF first!');
      return;
    }
    console.log("start", pdfPath)
    await invoke('open_pdf_presenter', { pdfPath });
  }

  async function nextPage() {
    await emitTo('singleview', 'control-pdf', { action: 'next' });
  }

  async function prevPage() {
    await emitTo('singleview', 'control-pdf', { action: 'prev' });
  }

  async function toggleFit() {
    await emitTo('singleview', 'control-pdf', { action: 'fitToggle' });
  }

  async function goToPage(page) {
    await emitTo('singleview', 'control-pdf', { action: 'goToPage', value: page });
  }

  function handleKeyDown(e) {
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {
      e.preventDefault();
      nextPage();
    }
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      prevPage();
    }
  }

  onMount(async () => {

    window.addEventListener('keydown', handleKeyDown);

    await listen('current-page', ({ payload }) => {
      currentPage = payload.page;
    });

    return () =>  async () => {
      await closePresentation();
      window.removeEventListener('keydown', handleKeyDown);
    }

  })

</script>

<main class="flex flex-col">
  
  <div class="flex flex-row items-center justify-between p-4">
    <button on:click={selectPDF} class="px-6 py-3 bg-gray-300 rounded hover:bg-gray-400">
      {pdfPath ? 'Change PDF' : 'Select PDF'}
    </button>

    {#if pdfPath}
      <p class="text-sm text-gray-600 w-72 text-center">{pdfPath.split('/').pop()}</p>
    {/if}

    {#if pdfPath}
      <button on:click={startPresentation} class="px-6 py-3 bg-blue-500 text-white rounded hover:bg-blue-600">
        Start Presentation
      </button>
    {/if}
  </div>

  {#if pdfPath}
    
    <div class="p-4">
      {#if thumbnails && thumbnails.length}
        <div class="grid grid-cols-4 gap-4">
          {#each thumbnails as thumb}
            <button
              class="cursor-pointer border-2 rounded p-1"
              class:border-blue-500={thumb.page === currentPage}
              on:click={() => goToPage(thumb.page)}
            >
              <img src={thumb.dataUrl} alt="Page {thumb.page}" class="w-full" />
              <div class="text-center text-sm">Page {thumb.page}</div>
            </button>
          {/each}
        </div>
      {:else}
        <p class="text-gray-500">No PDF loaded yet.</p>
      {/if}
    </div>

    <div class="flex flex-row items-center justify-between gap-4 p-4">
      <button on:click={prevPage} class="px-4 py-2 bg-gray-300 rounded">Previous</button>
      <button on:click={nextPage} class="px-4 py-2 bg-gray-300 rounded">Next</button>
      <!-- <button on:click={toggleFit} class="px-4 py-2 bg-blue-500 text-white rounded">Toggle Fit</button> -->
    </div>

  {/if}

</main>