/// Read the AFF code from URL params or cookie
export function useAffCode() {
  const AFF_COOKIE_KEY = 'aff_code'
  const AFF_COOKIE_DAYS = 30
  const AFF_CODE_PATTERN = /^[a-zA-Z0-9_-]+$/

  function getAffCode(): string | null {
    // Check URL params first
    const params = new URLSearchParams(window.location.search)
    const urlCode = params.get('aff')
    if (urlCode && AFF_CODE_PATTERN.test(urlCode)) {
      saveAffCode(urlCode)
      return urlCode
    }
    // Fall back to cookie
    const cookieCode = getCookie(AFF_COOKIE_KEY)
    if (cookieCode && AFF_CODE_PATTERN.test(cookieCode)) {
      return cookieCode
    }
    return null
  }

  function saveAffCode(code: string) {
    if (!AFF_CODE_PATTERN.test(code)) return
    const expires = new Date()
    expires.setDate(expires.getDate() + AFF_COOKIE_DAYS)
    document.cookie = `${AFF_COOKIE_KEY}=${encodeURIComponent(code)};expires=${expires.toUTCString()};path=/;SameSite=Lax`
  }

  function getCookie(name: string): string | null {
    const match = document.cookie.match(new RegExp(`(^| )${name}=([^;]+)`))
    return match ? decodeURIComponent(match[2]) : null
  }

  return { getAffCode, saveAffCode }
}
