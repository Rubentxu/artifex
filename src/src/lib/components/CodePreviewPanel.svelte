<script lang="ts">
  import type { CodeFileOutput } from '$lib/types/asset';

  interface Props {
    files: CodeFileOutput[];
    onclose?: () => void;
  }

  let { files, onclose }: Props = $props();

  let activeTabIndex = $state(0);
  let copiedIndex = $state<number | null>(null);

  let activeFile = $derived(files[activeTabIndex]);

  async function copyToClipboard(content: string, index: number) {
    try {
      await navigator.clipboard.writeText(content);
      copiedIndex = index;
      setTimeout(() => {
        copiedIndex = null;
      }, 2000);
    } catch (e) {
      console.error('Failed to copy:', e);
    }
  }

  function getLanguageClass(language: string): string {
    switch (language.toLowerCase()) {
      case 'gdscript':
        return 'language-python'; // GDScript is similar to Python
      case 'csharp':
      case 'c#':
        return 'language-csharp';
      case 'javascript':
      case 'js':
        return 'language-javascript';
      case 'typescript':
      case 'ts':
        return 'language-typescript';
      case 'json':
        return 'language-json';
      case 'html':
        return 'language-html';
      case 'css':
        return 'language-css';
      default:
        return 'language-plaintext';
    }
  }

  function getFileExtension(path: string): string {
    const parts = path.split('.');
    return parts.length > 1 ? parts[parts.length - 1] : '';
  }
</script>

{#if files.length > 0}
  <div class="flex flex-col h-full bg-[var(--color-panel)] rounded-lg border border-[var(--color-surface)] overflow-hidden">
    <!-- Tab bar for multi-file -->
    {#if files.length > 1}
      <div class="flex items-center gap-1 px-2 py-1.5 bg-[var(--color-canvas)] border-b border-[var(--color-surface)] overflow-x-auto">
        {#each files as file, index}
          <button
            onclick={() => (activeTabIndex = index)}
            class="px-3 py-1.5 rounded text-sm font-medium whitespace-nowrap transition-colors
              {activeTabIndex === index
                ? 'bg-[var(--color-accent)] text-white'
                : 'hover:bg-[var(--color-surface)] text-[var(--color-text-muted)]'}"
          >
            {file.path.split('/').pop()}
          </button>
        {/each}
      </div>
    {/if}

    <!-- File header -->
    <div class="flex items-center justify-between px-4 py-2 bg-[var(--color-canvas)] border-b border-[var(--color-surface)]">
      <div class="flex items-center gap-3">
        <span class="text-sm font-medium text-[var(--color-text)]">{activeFile.path}</span>
        <span class="px-2 py-0.5 rounded text-xs bg-[var(--color-surface)] text-[var(--color-text-muted)]">
          {activeFile.language}
        </span>
        {#if activeFile.description}
          <span class="text-xs text-[var(--color-text-muted)]">— {activeFile.description}</span>
        {/if}
      </div>
      <button
        onclick={() => copyToClipboard(activeFile.content, activeTabIndex)}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors text-sm font-medium"
      >
        {#if copiedIndex === activeTabIndex}
          <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-green-400">Copied!</span>
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
          </svg>
          Copy
        {/if}
      </button>
    </div>

    <!-- Code content -->
    <div class="flex-1 overflow-auto p-4 bg-[var(--color-canvas)]">
      <pre class="text-sm font-mono leading-relaxed whitespace-pre-wrap break-words"><code class="{getLanguageClass(activeFile.language)}">{activeFile.content}</code></pre>
    </div>

    <!-- Actions footer -->
    <div class="flex items-center justify-between px-4 py-2 bg-[var(--color-canvas)] border-t border-[var(--color-surface)]">
      <span class="text-xs text-[var(--color-text-muted)]">
        {files.length} file{files.length !== 1 ? 's' : ''} generated
      </span>
      {#if onclose}
        <button
          onclick={onclose}
          class="px-3 py-1.5 rounded-lg hover:bg-[var(--color-surface)] transition-colors text-sm font-medium"
        >
          Close
        </button>
      {/if}
    </div>
  </div>
{:else}
  <div class="flex flex-col items-center justify-center h-full text-center p-8">
    <svg class="w-16 h-16 text-[var(--color-text-muted)] mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
    </svg>
    <h3 class="text-lg font-semibold mb-2">No code to display</h3>
    <p class="text-sm text-[var(--color-text-muted)]">Generated code files will appear here</p>
  </div>
{/if}
