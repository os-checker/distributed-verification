<script setup lang="ts">

const CLASS_DARK = "my-app-dark";
function toggleTheme() {
  const isDark = document.documentElement.classList.toggle(CLASS_DARK);
  localStorage.setItem("theme", isDark ? "dark" : "light");
}

if (localStorage.getItem("theme") === "dark") {
  document.documentElement.classList.add(CLASS_DARK);
}

</script>

<template>
  <div>
    <Button icon="pi pi-refresh" @click="toggleTheme" rounded raised />

    <slot />
  </div>
</template>
