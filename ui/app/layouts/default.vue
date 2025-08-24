<script setup lang="ts">
import { useDarkStore } from '~/stores/style';

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

</script>

<template>
  <div>
    <div class="flex justify-between p-1">
      <div></div>
      <div>
        <Button :icon='dark ? "pi pi-sun" : "pi pi-moon"' @click="toggleTheme" severity="contrast" raised />
      </div>
    </div>

    <slot />
  </div>
</template>
