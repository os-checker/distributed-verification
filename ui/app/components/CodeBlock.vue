<script setup lang="ts">
import Prism from 'prismjs';
import 'prismjs/components/prism-rust';
import 'prismjs/plugins/line-numbers/prism-line-numbers.js';

defineProps<{ code: string }>();

const codeElement = ref<HTMLElement>();
function highlight() {
  if (codeElement.value) {
    Prism.highlightAllUnder(codeElement.value);
  }
};

onMounted(() => nextTick(highlight));
onUpdated(() => nextTick(highlight));

</script>

<template>
  <div ref="codeElement">
    <pre class="line-numbers"><code class="language-rust">{{ code }}</code></pre>
  </div>
</template>

<style lang="scss">
@use "sass:meta";

@import 'prismjs/plugins/line-numbers/prism-line-numbers.css';

html[data-theme="light"] {
  @include meta.load-css('prismjs/themes/prism.min.css');
}

html[data-theme="dark"] {
  @include meta.load-css('prismjs/themes/prism-tomorrow.min.css');
}

pre[class*="language-"].line-numbers {
  position: relative;
  padding-left: 3.8em !important;
  counter-reset: start;
}
</style>
