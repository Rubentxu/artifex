<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import PropertiesPanel from './PropertiesPanel.svelte';
  import StatusBar from './StatusBar.svelte';
  import { sidebarCollapsed, propertiesCollapsed } from '$lib/stores/ui';

  let { children } = $props();
</script>

<div class="h-screen w-screen flex flex-col overflow-hidden bg-[var(--color-canvas)]">
  <!-- Main content area -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Sidebar -->
    <Sidebar />

    <!-- Center workspace -->
    <main class="flex-1 overflow-hidden bg-[var(--color-canvas)]">
      {@render children()}
    </main>

    <!-- Properties Panel -->
    <PropertiesPanel />
  </div>

  <!-- Status Bar -->
  <StatusBar />
</div>
