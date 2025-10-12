<script setup lang="ts">
import { useDarkStore, useStyleStore } from '~/stores/style';

const CLASS_DARK = "my-app-dark";
const KEY = "theme";
const DARK = "dark";

const dark = ref(false);
const storeDark = useDarkStore();

function setDark(dark: boolean) {
  // Update dark state.
  storeDark.setFontColor(dark);
  // switch css, especially prism codeblock css
  document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
}

function toggleTheme() {
  const isDark = document.documentElement.classList.toggle(CLASS_DARK);
  dark.value = isDark;
  localStorage.setItem(KEY, isDark ? DARK : "light");
  setDark(isDark);
}

const isInitDark = localStorage.getItem(KEY) === DARK;
setDark(isInitDark);
if (isInitDark) {
  document.documentElement.classList.add(CLASS_DARK);
  dark.value = true;
}

// Get current route path.
const route = useRoute();
const active = computed(() => route.path);
const color = useStyleStore().color;

// e.g. /base/
const baseURL = useRuntimeConfig().app.baseURL;
console.log("baseURL", baseURL);
watch(active, val => console.log(`route.path=\`${val}\` baseURL=\`${baseURL}\` stripped=\`${val.replace(baseURL, "")}\``))

function btnStyle(target: string) {
  // remove baseURL, leading and trailing /
  var stripped_active = active.value.replace(baseURL, "").replace(/^\/+/, "").replace(/\/+$/, "");
  console.log(`is_active=${target === stripped_active} target=${target} stripped_active=${stripped_active} route.path=${active.value}`);
  return { background: (target === stripped_active) ? color.orange_light : color.primary, "border-color": "transparent" };
}
</script>

<template>
  <div>
    <div class="flex justify-between my-1 px-2">
      <div class="flex gap-2">
        <NuxtLink to="/">
          <Button title="Table" icon="pi pi-table" :style="btnStyle('')" />
        </NuxtLink>
        <NuxtLink to="chart">
          <Button title="Chart" icon="pi pi-chart-bar" :style="btnStyle('chart')" />
        </NuxtLink>
      </div>
      <div>
        <Button :icon='dark ? "pi pi-sun" : "pi pi-moon"' @click="toggleTheme" severity="contrast" raised />
      </div>
    </div>

    <slot />
  </div>
</template>
