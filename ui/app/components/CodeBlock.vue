<script setup lang="ts">
import Prism from 'prismjs';
import 'prismjs/components/prism-rust';
import 'prismjs/plugins/line-numbers/prism-line-numbers.js';

defineProps<{ code: string }>();
onMounted(Prism.highlightAll);
onUpdated(Prism.highlightAll);

</script>

<template>
  <div>
    <pre class="line-numbers"><code class="language-rust">{{ code }}</code></pre>
  </div>
</template>
