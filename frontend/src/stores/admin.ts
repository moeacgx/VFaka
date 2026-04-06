import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminApi } from '../api/admin'
import router from '../router'

export const useAdminStore = defineStore('admin', () => {
  const token = ref(localStorage.getItem('admin_token') || '')
  const username = ref(localStorage.getItem('admin_username') || '')
  const isLoggedIn = computed(() => !!token.value)

  async function login(user: string, password: string) {
    const res = await adminApi.login({ username: user, password })
    const data = res.data
    token.value = data.token
    username.value = data.username || user
    localStorage.setItem('admin_token', data.token)
    localStorage.setItem('admin_username', username.value)
  }

  function logout() {
    token.value = ''
    username.value = ''
    localStorage.removeItem('admin_token')
    localStorage.removeItem('admin_username')
    router.push('/admin/login')
  }

  return { token, username, isLoggedIn, login, logout }
})
