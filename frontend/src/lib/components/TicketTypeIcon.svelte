<script lang="ts">
  let { type }: { type: string } = $props();

  const icons: Record<string, { icon: string; color: string }> = {
    epic: { icon: '⚡', color: 'text-purple-600' },
    story: { icon: '📖', color: 'text-green-600' },
    task: { icon: '✓', color: 'text-blue-600' },
    subtask: { icon: '↳', color: 'text-gray-500' },
    bug: { icon: '🐛', color: 'text-red-600' }
  };

  const item = $derived(icons[type] || icons.task);
</script>

<span class="text-sm {item.color}" title={type}>{item.icon}</span>
