/// Read the AFF code from URL params or cookie
export function useAffCode() {
  const AFF_COOKIE_KEY = 'aff_code'
  const AFF_COOKIE_DAYS = 30

  function getAffCode(): string | null {
    // Check URL params first
    const params = new URLSearchParams(window.location.search)
    const urlCode = params.get('aff')
    if (urlCode) {
      saveAffCode(urlCode)
      return urlCode
    }
    // Fall back to cookie
    return getCookie(AFF_COOKIE_KEY)
  }

  function saveAffCode(code: string) {
    const expires = new Date()
    expires.setDate(expires.getDate() + AFF_COOKIE_DAYS)
    document.cookie = `${AFF_COOKIE_KEY}=${code};expires=${expires.toUTCString()};path=/`
  }

  function getCookie(name: string): string | null {
    const match = document.cookie.match(new RegExp(`(^| )${name}=([^;]+)`))
    return match ? match[2] : null
  }

  return { getAffCode, saveAffCode }
}
