import Aura from '@primeuix/themes/aura';

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  ssr: false, // Client-side Only Rendering (SPA)
  css: [
    "primeicons/primeicons.css",
    "~/assets/css/main.css",
  ],
  typescript: { typeCheck: true },
  modules: [
    '@primevue/nuxt-module',
    '@nuxtjs/tailwindcss',
    '@pinia/nuxt'
  ],
  primevue: {
    options: {
      theme: {
        preset: Aura,
        options: { darkModeSelector: '.my-app-dark' }
      }
    }
  }
})
