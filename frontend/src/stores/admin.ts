import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAdminStore = defineStore('admin', () => {
  const token = ref(localStorage.getItem('admin_token') || '')
  const isLoggedIn = ref(!!token.value)

  function setToken(newToken: string) {
    token.value = newToken
    localStorage.setItem('admin_token', newToken)
    isLoggedIn.value = true
  }

  function logout() {
    token.value = ''
    localStorage.removeItem('admin_token')
    isLoggedIn.value = false
  }

  return { token, isLoggedIn, setToken, logout }
})
