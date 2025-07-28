<script>
  import { onMount, tick } from 'svelte';
  import { listen, emitTo } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { loadPDF } from '../../lib/pdfRenderer.js';

  let pdfPath = '';
  let pdfDoc = null;
  let totalPages = 0;
  let currentPage = 1;
  let canvas;
  let fitMode = 'page';
  let renderTask = null; // Track the active render

  function computeScale(viewport) {
    const containerWidth = window.innerWidth - 40;
    const containerHeight = window.innerHeight - 40;
    const widthScale = containerWidth / viewport.width;
    const heightScale = containerHeight / viewport.height;
    return fitMode === 'width' ? widthScale : Math.min(widthScale, heightScale);
  }

  function notifyCurrentPage() {
    emitTo('main', 'current-page', { page: currentPage });
  }

  async function renderPage(page) {
    if (!canvas || !pdfDoc) return;

    // If there's an ongoing render, cancel it first
    if (renderTask) {
      try {
        renderTask.cancel();
      } catch (e) {
        console.warn('Render task cancel failed:', e);
      }
    }

    const pageObj = await pdfDoc.getPage(page);
    const unscaledViewport = pageObj.getViewport({ scale: 1.0 });
    const scale = computeScale(unscaledViewport);
    const viewport = pageObj.getViewport({ scale });

    const context = canvas.getContext('2d');
    canvas.width = viewport.width;
    canvas.height = viewport.height;

    // Start a new render task
    renderTask = pageObj.render({ canvasContext: context, viewport });

    try {
      await renderTask.promise;
    } catch (err) {
      if (err?.name !== 'RenderingCancelledException') {
        console.error('PDF render error:', err);
      }
    } finally {
      renderTask = null;
    }

    notifyCurrentPage();
  }

  async function loadDocument() {
    const result = await loadPDF(pdfPath, canvas, 1);
    pdfDoc = result.pdfDoc;
    totalPages = result.totalPages;
    currentPage = 1;
    await renderPage(currentPage);
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
      renderPage(currentPage);
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      renderPage(currentPage);
    }
  }

  function toggleFitMode() {
    fitMode = fitMode === 'page' ? 'width' : 'page';
    renderPage(currentPage);
  }

  function goToPage(page) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
      renderPage(currentPage);
    }
  }

  onMount(async () => {
    pdfPath = await invoke('get_current_pdf_path', { window: 'singleview' });
    if (pdfPath) {
      await tick();
      await loadDocument();
    }

    // Handle window resizing
    window.addEventListener('resize', () => renderPage(currentPage));

    // Remote controls from main window
    await listen('control-pdf', ({ payload }) => {
      const { action, value } = payload;
      if (action === 'next') nextPage();
      if (action === 'prev') prevPage();
      if (action === 'fitToggle') toggleFitMode();
      if (action === 'goToPage') goToPage(value);
    });
  });
</script>

<div class="flex flex-col items-center justify-center gap-4 p-4 h-screen bg-gray-100">
  {#if pdfPath}
    <canvas bind:this={canvas} class="border shadow-lg"></canvas>
    <!-- <div class="flex gap-4 items-center">
      <span class="text-lg">{currentPage} / {totalPages}</span>
    </div> -->
  {:else}
    <p class="text-gray-500">Waiting for PDF selection...</p>
  {/if}
</div>
