export const useStyleStore = defineStore("style", () => {
  const color = reactive({
    green: "green", red: "red", grey: "grey",
    primary: "white", orange: "orange", orange_light: "orange",
  });
  const viewportHeight = ref(800);
  const viewportWidth = ref(800);

  onMounted(() => {
    // Get styles after computation.
    const styles = window.getComputedStyle(document.documentElement);

    // Set color shortcuts.
    color.green = styles.getPropertyValue("--p-emerald-300").trim();
    color.red = styles.getPropertyValue("--p-red-500").trim();
    color.grey = styles.getPropertyValue("--p-gray-400").trim();
    color.primary = styles.getPropertyValue("--p-button-primary-background").trim();
    color.orange_light = styles.getPropertyValue("--p-orange-400").trim();
    color.orange = styles.getPropertyValue("--p-orange-500").trim();

    // Get heights and widths.
    viewportHeight.value = window.innerHeight;
    viewportWidth.value = window.innerWidth;
    window.addEventListener("resize", () => {
      viewportHeight.value = window.innerHeight;
      viewportWidth.value = window.innerWidth;
    });
  });

  return { color, viewportHeight, viewportWidth }
});

/** Styling based on dark theme mode. */
export const useDarkStore = defineStore('dark', {
  state: () => ({ fontColor: "black", isDark: false }),
  actions: {
    setFontColor(isDark: boolean) {
      this.fontColor = isDark ? "white" : "black";
      this.isDark = isDark;
      console.log("setFontColor")
    }
  }
});
