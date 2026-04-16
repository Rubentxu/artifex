<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateAudioRequest, GenerateTtsRequest } from '$lib/types/asset';

  interface Props {
    open: boolean;
    projectId: string;
    onclose: () => void;
  }

  let { open, projectId, onclose }: Props = $props();

  // Tab state: 'sfx' | 'music' | 'tts'
  let activeTab = $state<'sfx' | 'music' | 'tts'>('sfx');

  // SFX/Music fields
  let prompt = $state('');
  let durationSecs = $state<number | undefined>(undefined);

  // TTS fields
  let text = $state('');
  let voiceId = $state<string>('');
  let speed = $state<number>(1.0);

  let loading = $state(false);
  let error = $state<string | null>(null);

  // Pre-defined voice IDs for ElevenLabs (commonly used voices)
  const voiceOptions = [
    { id: '21m00Tcm4TlvDq8ikWAM', name: 'Rachel (Female, US)' },
    { id: '29vD33nK6H0toGmPJ5zk', name: 'Dani (Female, US)' },
    { id: 'ErXwobaYiN019pkygt10', name: 'Bella (Female, US)' },
    { id: 'EXAVITQu4vr4xnSDxMaL', name: ' Antoni (Male, US)' },
    { id: 'VR6AewLTigWG4xSOukaG', name: 'Arnold (Male, US)' },
    { id: 'pFZP5JQG7iQjIDomC2DO', name: 'Sarah (Female, UK)' },
  ];

  async function handleGenerate() {
    error = null;

    if (activeTab === 'tts') {
      if (!text.trim()) {
        error = 'Text is required for TTS';
        return;
      }
    } else {
      if (!prompt.trim()) {
        error = 'Prompt is required';
        return;
      }
    }

    loading = true;
    try {
      if (activeTab === 'tts') {
        const request: GenerateTtsRequest = {
          project_id: projectId,
          params: {
            text: text.trim(),
            voice_id: voiceId || undefined,
            speed,
          },
        };
        await assetStore.synthesizeSpeech(request);
      } else {
        const request: GenerateAudioRequest = {
          project_id: projectId,
          params: {
            prompt: prompt.trim(),
            kind: activeTab,
            duration_secs: durationSecs,
          },
        };
        await assetStore.generateAudio(request);
      }
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    prompt = '';
    durationSecs = undefined;
    text = '';
    voiceId = '';
    speed = 1.0;
    error = null;
    activeTab = 'sfx';
    onclose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    onclick={handleClose}
    role="dialog"
    aria-modal="true"
  >
    <!-- Dialog -->
    <div
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-lg mx-4 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">Generate Audio</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-[var(--color-surface)]">
        <button
          class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'sfx' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-accent)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
          onclick={() => activeTab = 'sfx'}
        >
          SFX
        </button>
        <button
          class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'music' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-accent)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
          onclick={() => activeTab = 'music'}
        >
          Music
        </button>
        <button
          class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'tts' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-accent)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
          onclick={() => activeTab = 'tts'}
        >
          TTS
        </button>
      </div>

      <!-- Form -->
      <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="p-4 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        {#if activeTab === 'tts'}
          <!-- TTS: Text input -->
          <div>
            <label for="tts-text" class="block text-sm font-medium mb-1">
              Text <span class="text-red-400">*</span>
            </label>
            <textarea
              id="tts-text"
              bind:value={text}
              placeholder="Enter the text to synthesize..."
              rows="4"
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              required
            ></textarea>
          </div>

          <!-- Voice selection -->
          <div>
            <label for="tts-voice" class="block text-sm font-medium mb-1">
              Voice
            </label>
            <select
              id="tts-voice"
              bind:value={voiceId}
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
            >
              <option value="">Default (Rachel)</option>
              {#each voiceOptions as voice}
                <option value={voice.id}>{voice.name}</option>
              {/each}
            </select>
          </div>

          <!-- Speed -->
          <div>
            <label for="tts-speed" class="block text-sm font-medium mb-1">
              Speed: {speed.toFixed(1)}x
            </label>
            <input
              id="tts-speed"
              type="range"
              bind:value={speed}
              min="0.5"
              max="2.0"
              step="0.1"
              class="w-full"
            />
            <div class="flex justify-between text-xs text-[var(--color-text-muted)]">
              <span>0.5x</span>
              <span>1.0x</span>
              <span>2.0x</span>
            </div>
          </div>
        {:else}
          <!-- SFX/Music: Prompt + Duration -->
          <div>
            <label for="audio-prompt" class="block text-sm font-medium mb-1">
              Prompt <span class="text-red-400">*</span>
            </label>
            <textarea
              id="audio-prompt"
              bind:value={prompt}
              placeholder={activeTab === 'music' ? 'Epic orchestral battle music with drums and brass' : 'Explosion sound effect, deep bass, debris falling'}
              rows="3"
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              required
            ></textarea>
          </div>

          <!-- Duration -->
          <div>
            <label for="audio-duration" class="block text-sm font-medium mb-1">
              Duration (seconds)
            </label>
            <input
              id="audio-duration"
              type="number"
              bind:value={durationSecs}
              min="1"
              max="300"
              placeholder={activeTab === 'music' ? '30' : '5'}
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
            />
          </div>
        {/if}

        <!-- Actions -->
        <div class="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={handleClose}
            class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
            disabled={loading}
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
            disabled={loading}
          >
            {loading ? 'Generating...' : 'Generate'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
