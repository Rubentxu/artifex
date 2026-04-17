<script lang="ts">
  interface Props {
    imageUrl: string;
    width: number;
    height: number;
    brushSize?: number;
    initialMask?: string; // base64 PNG of existing mask
    onmaskchange?: (maskBase64: string) => void;
  }

  let {
    imageUrl,
    width,
    height,
    brushSize = 20,
    initialMask = '',
    onmaskchange,
  }: Props = $props();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let imageObj: HTMLImageElement | null = null;
  let isDrawing = false;
  let lastX = 0;
  let lastY = 0;
  let currentMode: 'brush' | 'eraser' = 'brush';
  let maskData = $state<ImageData | null>(null);
  let hasChanges = $state(false);

  // Scale factor between canvas display size and actual image resolution
  let scaleX = 1;
  let scaleY = 1;

  function initCanvas() {
    if (!canvas) return;
    ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Load the image
    imageObj = new Image();
    imageObj.crossOrigin = 'anonymous';
    imageObj.onload = () => {
      if (!imageObj || !ctx) return;

      // Set canvas to actual image dimensions
      canvas.width = imageObj.naturalWidth;
      canvas.height = imageObj.naturalHeight;

      // Calculate scale to fit display
      scaleX = imageObj.naturalWidth / width;
      scaleY = imageObj.naturalHeight / height;

      // Draw image
      ctx.drawImage(imageObj, 0, 0);

      // Load initial mask if provided
      if (initialMask) {
        const maskImg = new Image();
        maskImg.crossOrigin = 'anonymous';
        maskImg.onload = () => {
          ctx?.drawImage(maskImg, 0, 0);
          maskData = ctx?.getImageData(0, 0, canvas.width, canvas.height) || null;
        };
        maskImg.src = initialMask;
      } else {
        // Initialize empty mask (black = keep, transparent)
        maskData = ctx.createImageData(canvas.width, canvas.height);
      }

      hasChanges = false;
    };
    imageObj.src = imageUrl;
  }

  function getCanvasCoords(e: MouseEvent): [number, number] {
    const rect = canvas.getBoundingClientRect();
    const displayWidth = rect.width;
    const displayHeight = rect.height;

    // Calculate position relative to displayed canvas
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Scale to actual canvas coordinates
    const canvasX = Math.floor((x / displayWidth) * canvas.width);
    const canvasY = Math.floor((y / displayHeight) * canvas.height);

    return [canvasX, canvasY];
  }

  function startDrawing(e: MouseEvent) {
    isDrawing = true;
    const [x, y] = getCanvasCoords(e);
    lastX = x;
    lastY = y;
  }

  function draw(e: MouseEvent) {
    if (!isDrawing || !ctx) return;

    const [x, y] = getCanvasCoords(e);

    ctx.beginPath();
    ctx.moveTo(lastX, lastY);
    ctx.lineTo(x, y);
    ctx.strokeStyle = currentMode === 'brush' ? 'white' : 'black';
    ctx.lineWidth = brushSize;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.stroke();

    lastX = x;
    lastY = y;
    hasChanges = true;
  }

  function stopDrawing() {
    if (isDrawing && ctx) {
      isDrawing = false;
      // Update mask data
      maskData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      // Notify parent
      if (onmaskchange) {
        exportMask();
      }
    }
  }

  function clearMask() {
    if (!ctx || !imageObj) return;

    // Redraw original image
    ctx.drawImage(imageObj, 0, 0);

    // Reset mask data
    maskData = ctx.createImageData(canvas.width, canvas.height);
    hasChanges = true;

    if (onmaskchange) {
      exportMask();
    }
  }

  function exportMask(): string {
    if (!ctx || !canvas) return '';

    // Get the current canvas content
    const dataUrl = canvas.toDataURL('image/png');
    return dataUrl;
  }

  function setBrush() {
    currentMode = 'brush';
  }

  function setEraser() {
    currentMode = 'eraser';
  }

  function handleMaskChange() {
    if (onmaskchange) {
      onmaskchange(exportMask());
    }
  }

  // Export function for parent component to get the mask
  export function getMaskBase64(): string {
    return exportMask();
  }

  $effect(() => {
    initCanvas();
  });
</script>

<div class="flex flex-col gap-4">
  <!-- Canvas container with fixed dimensions -->
  <div
    class="relative border border-[var(--color-surface)] rounded-lg overflow-hidden"
    style="width: {width}px; height: {height}px;"
  >
    <canvas
      bind:this={canvas}
      class="w-full h-full cursor-crosshair"
      style="width: {width}px; height: {height}px;"
      onmousedown={startDrawing}
      onmousemove={draw}
      onmouseup={stopDrawing}
      onmouseleave={stopDrawing}
    ></canvas>
  </div>

  <!-- Controls -->
  <div class="flex items-center gap-4">
    <!-- Brush size -->
    <div class="flex items-center gap-2">
      <label class="text-sm text-[var(--color-text-muted)]">Brush:</label>
      <input
        type="range"
        min="5"
        max="100"
        value={brushSize}
        oninput={(e) => brushSize = parseInt((e.target as HTMLInputElement).value)}
        class="w-32"
      />
      <span class="text-sm w-8">{brushSize}px</span>
    </div>

    <!-- Mode buttons -->
    <div class="flex items-center gap-1">
      <button
        onclick={setBrush}
        class="px-3 py-1 rounded text-sm font-medium transition-colors"
        class:bg-[var(--color-accent)]={currentMode === 'brush'}
        class:text-white={currentMode === 'brush'}
        class:bg-[var(--color-surface)]={currentMode !== 'brush'}
      >
        Brush
      </button>
      <button
        onclick={setEraser}
        class="px-3 py-1 rounded text-sm font-medium transition-colors"
        class:bg-[var(--color-accent)]={currentMode === 'eraser'}
        class:text-white={currentMode === 'eraser'}
        class:bg-[var(--color-surface)]={currentMode !== 'eraser'}
      >
        Eraser
      </button>
    </div>

    <!-- Clear button -->
    <button
      onclick={clearMask}
      class="px-3 py-1 rounded text-sm font-medium bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors"
    >
      Clear
    </button>
  </div>

  <!-- Hint -->
  <p class="text-xs text-[var(--color-text-muted)]">
    Draw white to mark areas to regenerate. Use eraser to remove marks.
  </p>
</div>
