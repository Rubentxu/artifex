<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateAudioRequest, GenerateTtsRequest } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Generate Audio"
  width="lg"
  bordered
  {error}
>
  <!-- Tabs -->
  <div class="flex border-b border-[var(--color-surface)]">
    <button
      class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'sfx' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-neon)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
      onclick={() => activeTab = 'sfx'}
    >
      SFX
    </button>
    <button
      class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'music' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-neon)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
      onclick={() => activeTab = 'music'}
    >
      Music
    </button>
    <button
      class="flex-1 px-4 py-2 text-sm font-medium transition-colors {activeTab === 'tts' ? 'text-[var(--color-accent)] border-b-2 border-[var(--color-neon)]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'}"
      onclick={() => activeTab = 'tts'}
    >
      TTS
    </button>
  </div>

  <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="space-y-4">
    {#if activeTab === 'tts'}
      <!-- TTS: Text input -->
      <div>
        <label for="tts-text" class="block text-sm font-medium mb-1">
          Text <span class="text-[var(--color-destructive)]">*</span>
        </label>
        <textarea
          id="tts-text"
          bind:value={text}
          placeholder="Enter the text to synthesize..."
          rows="4"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
          Prompt <span class="text-[var(--color-destructive)]">*</span>
        </label>
        <textarea
          id="audio-prompt"
          bind:value={prompt}
          placeholder={activeTab === 'music' ? 'Epic orchestral battle music with drums and brass' : 'Explosion sound effect, deep bass, debris falling'}
          rows="3"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
    {/if}
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleGenerate} disabled={loading}>
      {loading ? 'Generating...' : 'Generate'}
    </Button>
  {/snippet}
</Dialog>
