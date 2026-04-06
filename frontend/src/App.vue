<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useTheme } from './composables/useTheme'
import { useSiteInfo } from './composables/useSiteInfo'

const route = useRoute()
const { locale } = useI18n()
const { themeMode, toggleTheme } = useTheme()
const { siteInfo } = useSiteInfo()

const isAdmin = computed(() => route.path.startsWith('/admin'))

// Announcement bar
const announcement = ref<{ enabled: boolean; text: string; type: string } | null>(null)
const announcementDismissed = ref(false)

onMounted(async () => {
  if (sessionStorage.getItem('announcement_dismissed')) {
    announcementDismissed.value = true
  }
  try {
    const res = await fetch('/api/v1/announcement')
    const data = await res.json()
    if (data.enabled) {
      announcement.value = data
    }
  } catch {
    // ignore
  }
})

function dismissAnnouncement() {
  announcementDismissed.value = true
  sessionStorage.setItem('announcement_dismissed', '1')
}

const announcementClasses = computed(() => {
  const type = announcement.value?.type || 'info'
  const map: Record<string, string> = {
    info: 'bg-blue-50 text-blue-800 border-blue-200 dark:bg-blue-900/30 dark:text-blue-300 dark:border-blue-800',
    warning: 'bg-yellow-50 text-yellow-800 border-yellow-200 dark:bg-yellow-900/30 dark:text-yellow-300 dark:border-yellow-800',
    success: 'bg-green-50 text-green-800 border-green-200 dark:bg-green-900/30 dark:text-green-300 dark:border-green-800',
  }
  return map[type] || map.info
})

// Language switcher
const langOpen = ref(false)
const languages = [
  { code: 'zh-CN', label: '简体中文' },
  { code: 'zh-TW', label: '繁體中文' },
  { code: 'en', label: 'English' },
]

function setLocale(code: string) {
  locale.value = code
  localStorage.setItem('locale', code)
  langOpen.value = false
}

const themeIcon = computed(() => {
  if (themeMode.value === 'light') return 'sun'
  if (themeMode.value === 'dark') return 'moon'
  return 'system'
})
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
    <!-- Announcement Bar -->
    <div
      v-if="announcement && announcement.enabled && !announcementDismissed"
      :class="['border-b px-4 py-2 text-sm flex items-center justify-between', announcementClasses]"
    >
      <span>{{ announcement.text }}</span>
      <button
        class="ml-4 shrink-0 opacity-70 hover:opacity-100 transition-opacity"
        @click="dismissAnnouncement"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>

    <!-- Public Header -->
    <header v-if="!isAdmin" class="bg-white dark:bg-gray-800 border-b border-gray-100 dark:border-gray-700">
      <div class="max-w-4xl mx-auto px-4 py-4 flex items-center justify-between">
        <router-link to="/" class="flex items-center gap-2 text-xl font-semibold text-gray-900 dark:text-white hover:text-gray-900 dark:hover:text-white no-underline">
          <img v-if="siteInfo.site_logo" :src="siteInfo.site_logo" class="h-7 w-7 object-contain rounded" alt="" />
          {{ siteInfo.site_name || 'AFF Card Shop' }}
        </router-link>
        <div class="flex items-center gap-4">
          <nav class="flex gap-4 text-sm text-gray-500 dark:text-gray-400">
            <router-link to="/" class="hover:text-gray-900 dark:hover:text-white transition-colors">{{ $t('nav.home') }}</router-link>
            <router-link to="/order" class="hover:text-gray-900 dark:hover:text-white transition-colors">{{ $t('nav.order_query') }}</router-link>
            <router-link to="/aff" class="hover:text-gray-900 dark:hover:text-white transition-colors">{{ $t('nav.affiliate') }}</router-link>
          </nav>

          <!-- Theme Toggle -->
          <button
            class="p-1.5 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors"
            :title="$t('theme.' + themeMode)"
            @click="toggleTheme"
          >
            <svg v-if="themeIcon === 'sun'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m8.66-13.66l-.71.71M4.05 19.95l-.71.71M21 12h-1M4 12H3m16.66 7.66l-.71-.71M4.05 4.05l-.71-.71M16 12a4 4 0 11-8 0 4 4 0 018 0z"/></svg>
            <svg v-else-if="themeIcon === 'moon'" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/></svg>
            <svg v-else class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
          </button>

          <!-- Language Switcher -->
          <div class="relative">
            <button
              class="p-1.5 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white transition-colors text-sm"
              @click="langOpen = !langOpen"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9"/></svg>
            </button>
            <div
              v-if="langOpen"
              class="absolute right-0 mt-1 w-36 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-sm dark:shadow-none py-1 z-50"
            >
              <button
                v-for="lang in languages"
                :key="lang.code"
                class="block w-full text-left px-3 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                :class="{ 'font-medium text-blue-600 dark:text-blue-400': locale === lang.code }"
                @click="setLocale(lang.code)"
              >
                {{ lang.label }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Click-away for language dropdown -->
    <div v-if="langOpen" class="fixed inset-0 z-40" @click="langOpen = false" />

    <router-view />
  </div>
</template>
