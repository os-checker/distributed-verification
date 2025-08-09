export const useStyleStore = defineStore("style", () => {
  const color = reactive({
    green: "green", red: "red", grey: "grey",
    topButton: "white", orange: "orange", orange_light: "orange",
  });
  const viewportHeight = ref(800);

  onMounted(() => {
    // Get styles after computation.
    const styles = window.getComputedStyle(document.documentElement);

    // Set color shortcuts.
    color.green = styles.getPropertyValue("--p-emerald-500").trim();
    color.red = styles.getPropertyValue("--p-red-500").trim();
    color.grey = styles.getPropertyValue("--p-gray-400").trim();
    color.topButton = styles.getPropertyValue("--p-button-primary-background").trim();
    color.orange_light = styles.getPropertyValue("--p-orange-400").trim();
    color.orange = styles.getPropertyValue("--p-orange-500").trim();

    // Get heights
    viewportHeight.value = window.innerHeight;
    window.addEventListener("resize", () => {
      viewportHeight.value = window.innerHeight;
    });
  });

  return { color, viewportHeight }
});

