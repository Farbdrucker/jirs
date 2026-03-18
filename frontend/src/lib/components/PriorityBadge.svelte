<script lang="ts">
  let { priority }: { priority: string } = $props();

  const colors: Record<string, string> = {
    low: 'bg-gray-100 text-gray-600',
    medium: 'bg-blue-100 text-blue-700',
    high: 'bg-orange-100 text-orange-700',
    critical: 'bg-red-100 text-red-700'
  };
</script>

<span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium {colors[priority] || colors.medium}">
  {priority}
</span>
