import { ref, watch, onMounted, onUnmounted } from 'vue'

type ThemeMode = 'light' | 'dark' | 'system'

const themeMode = ref<ThemeMode>('system')
const isDark = ref(false)

function getSystemDark(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyTheme() {
  if (themeMode.value === 'system') {
    isDark.value = getSystemDark()
  } else {
    isDark.value = themeMode.value === 'dark'
  }

  if (isDark.value) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

let mediaQueryHandler: (() => void) | null = null

export function useTheme() {
  onMounted(() => {
    const saved = localStorage.getItem('theme_preference') as ThemeMode | null
    if (saved && ['light', 'dark', 'system'].includes(saved)) {
      themeMode.value = saved
    }
    applyTheme()

    mediaQueryHandler = () => {
      if (themeMode.value === 'system') {
        applyTheme()
      }
    }
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', mediaQueryHandler)
  })

  onUnmounted(() => {
    if (mediaQueryHandler) {
      window.matchMedia('(prefers-color-scheme: dark)').removeEventListener('change', mediaQueryHandler)
      mediaQueryHandler = null
    }
  })

  watch(themeMode, (val) => {
    localStorage.setItem('theme_preference', val)
    applyTheme()
  })

  function toggleTheme() {
    const modes: ThemeMode[] = ['light', 'dark', 'system']
    const idx = modes.indexOf(themeMode.value)
    themeMode.value = modes[(idx + 1) % modes.length]
  }

  return { themeMode, isDark, toggleTheme }
}
