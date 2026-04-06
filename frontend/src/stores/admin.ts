import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { adminApi } from '../api/admin'
import router from '../router'

function parseJwtRole(token: string): string {
  try {
    const payload = token.split('.')[1]
    const decoded = JSON.parse(atob(payload))
    return decoded.role || 'admin'
  } catch {
    return 'admin'
  }
}

export const useAdminStore = defineStore('admin', () => {
  const token = ref(localStorage.getItem('admin_token') || '')
  const username = ref(localStorage.getItem('admin_username') || '')
  const role = ref(token.value ? parseJwtRole(token.value) : 'admin')
  const isLoggedIn = computed(() => !!token.value)
  const isSuperAdmin = computed(() => role.value === 'super_admin')

  async function login(user: string, password: string) {
    const res = await adminApi.login({ username: user, password })
    const data = res.data
    token.value = data.token
    username.value = data.username || user
    role.value = data.admin?.role || parseJwtRole(data.token)
    localStorage.setItem('admin_token', data.token)
    localStorage.setItem('admin_username', username.value)
  }

  function logout() {
    token.value = ''
    username.value = ''
    role.value = 'admin'
    localStorage.removeItem('admin_token')
    localStorage.removeItem('admin_username')
    router.push('/admin/login')
  }

  return { token, username, role, isLoggedIn, isSuperAdmin, login, logout }
})
