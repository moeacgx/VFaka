import axios from 'axios'

const request = axios.create({
  baseURL: '/api',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor — attach JWT token for admin routes
request.interceptors.request.use((config) => {
  const token = localStorage.getItem('admin_token')
  if (token && config.url?.startsWith('/admin')) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// Response interceptor — handle errors globally
request.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('admin_token')
      if (window.location.pathname.startsWith('/admin')) {
        window.location.href = '/admin/login'
      }
    }
    if (error.response?.status === 403) {
      if (window.location.pathname.startsWith('/admin')) {
        alert(error.response?.data?.message || 'Permission denied')
        window.location.href = '/admin/dashboard'
      }
    }
    return Promise.reject(error)
  }
)

export default request
