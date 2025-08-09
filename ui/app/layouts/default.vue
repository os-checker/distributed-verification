<script setup lang="ts">

const CLASS_DARK = "my-app-dark";
const KEY = "theme";
const DARK = "dark";

const dark = ref(false);

function toggleTheme() {
  const isDark = document.documentElement.classList.toggle(CLASS_DARK);
  dark.value = isDark;
  localStorage.setItem(KEY, isDark ? DARK : "light");
}

if (localStorage.getItem(KEY) === DARK) {
  document.documentElement.classList.add(CLASS_DARK);
  dark.value = true;
}

</script>

<template>
  <div>
    <div class="flex justify-between p-2" id="topbar">
      <div></div>
      <div>
        <Button :icon='dark ? "pi pi-sun" : "pi pi-moon"' @click="toggleTheme" severity="contrast" raised />
      </div>
    </div>

    <slot />
  </div>
</template>
