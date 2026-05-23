<script lang="ts">
  interface Props {
    variant?: 'image' | 'sprite' | 'tileset' | 'material' | 'audio' | 'voice' | 'video' | 'default';
    status?: 'active' | 'archived' | 'error';
  }

  const { variant = 'default', status }: Props = $props();

  const KIND_COLORS: Record<string, { bg: string; text: string }> = {
    image:    { bg: 'bg-[var(--color-cyan)]/20',   text: 'text-[var(--color-cyan)]'   },
    sprite:   { bg: 'bg-[var(--color-neon)]/20',  text: 'text-[var(--color-neon)]'  },
    tileset:  { bg: 'bg-[var(--color-magenta)]/20', text: 'text-[var(--color-magenta)]' },
    material: { bg: 'bg-[var(--color-neon)]/20', text: 'text-[var(--color-neon)]' },
    audio:    { bg: 'bg-[var(--color-neon)]/20', text: 'text-[var(--color-neon)]' },
    voice:    { bg: 'bg-[var(--color-magenta)]/20',  text: 'text-[var(--color-magenta)]'  },
    video:    { bg: 'bg-[var(--color-muted)]/20',    text: 'text-[var(--color-muted)]'    },
    default:  { bg: 'bg-[var(--color-muted)]/20',  text: 'text-[var(--color-muted)]'   },
  };

  const STATUS_COLORS: Record<string, { bg: string; text: string }> = {
    active:   { bg: 'bg-[var(--color-neon)]/20', text: 'text-[var(--color-neon)]' },
    archived: { bg: 'bg-[var(--color-neon)]/20', text: 'text-[var(--color-neon)]' },
    error:    { bg: 'bg-[var(--color-destructive)]/20', text: 'text-[var(--color-destructive)]' },
  };

  const colors = $derived(
    status ? (STATUS_COLORS[status] ?? STATUS_COLORS.active)
    : (KIND_COLORS[variant] ?? KIND_COLORS.default)
  );
</script>

<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {colors.bg} {colors.text}">
  <slot />
</span>
