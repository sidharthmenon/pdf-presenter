import { invoke } from '@tauri-apps/api/core';

// Import PDF.js core (legacy build works better in Vite)
import * as pdfjsLib from 'pdfjs-dist/legacy/build/pdf';

// Tell PDF.js where the worker is (use the bundled one)
import pdfjsWorker from 'pdfjs-dist/legacy/build/pdf.worker?worker';

// Set the worker for PDF.js
pdfjsLib.GlobalWorkerOptions.workerPort = new pdfjsWorker();

/**
 * Loads and renders a PDF page into a canvas.
 *
 * @param {string} pdfPath - Path to the PDF (Tauri file path)
 * @param {HTMLCanvasElement} canvas - Canvas to render into
 * @param {number} pageNumber - Page number (1-based)
 * @returns {Promise<{ pdfDoc: any, totalPages: number }>}
 */
export async function loadPDF(pdfPath, canvas, pageNumber = 1) {
  // Load PDF
  const bytes = await invoke('load_pdf_bytes', { path: pdfPath });

  const pdfDoc = await pdfjsLib.getDocument({ data: new Uint8Array(bytes) }).promise;

  const totalPages = pdfDoc.numPages;
  const page = await pdfDoc.getPage(pageNumber);

  // Scale for display
  const viewport = page.getViewport({ scale: 1.5 });
  const context = canvas.getContext('2d');

  canvas.width = viewport.width;
  canvas.height = viewport.height;

  await page.render({ canvasContext: context, viewport }).promise;

  return { pdfDoc, totalPages };
}

export async function loadThumbnails(pdfPath) {
    const bytes = await invoke('load_pdf_bytes', { path: pdfPath });
    const pdfDoc = await pdfjsLib.getDocument({ data: new Uint8Array(bytes) }).promise;

    let thumbnails = [];

    for (let i = 1; i <= pdfDoc.numPages; i++) {
      const page = await pdfDoc.getPage(i);
      const viewport = page.getViewport({ scale: 0.5 }); // Small scale for thumbnail
      const canvas = document.createElement('canvas');
      const context = canvas.getContext('2d');
      canvas.width = viewport.width;
      canvas.height = viewport.height;

      await page.render({ canvasContext: context, viewport }).promise;

      thumbnails.push({ page: i, dataUrl: canvas.toDataURL() });
    }

    return thumbnails;
  }
