import { ref } from 'vue'
import { publicApi } from '../api/public'

interface SiteInfo {
  site_name: string
  site_description: string
  site_keywords: string
  site_logo: string
  contact_email: string
  contact_telegram: string
}

const siteInfo = ref<SiteInfo>({
  site_name: '',
  site_description: '',
  site_keywords: '',
  site_logo: '',
  contact_email: '',
  contact_telegram: '',
})
const loaded = ref(false)
let fetchPromise: Promise<void> | null = null

async function fetchSiteInfo() {
  if (loaded.value) return
  if (fetchPromise) return fetchPromise

  fetchPromise = (async () => {
    try {
      const res = await publicApi.getSiteInfo()
      const data = res.data
      if (data && typeof data === 'object') {
        siteInfo.value = { ...siteInfo.value, ...data }
      }
      loaded.value = true

      if (siteInfo.value.site_name) {
        document.title = siteInfo.value.site_name
      }
    } catch {
      // fallback to defaults
    } finally {
      fetchPromise = null
    }
  })()

  return fetchPromise
}

export function useSiteInfo() {
  fetchSiteInfo()
  return { siteInfo }
}
