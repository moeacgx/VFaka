import request from './request'

// Admin API calls will be added here as features are implemented
export const adminApi = {
  login: (data: { username: string; password: string }) => request.post('/admin/auth/login', data),
  getDashboard: () => request.get('/admin/dashboard'),

  // Categories
  getCategories: () => request.get('/admin/categories'),
  createCategory: (data: any) => request.post('/admin/categories', data),
  updateCategory: (id: number, data: any) => request.put(`/admin/categories/${id}`, data),
  deleteCategory: (id: number) => request.delete(`/admin/categories/${id}`),
  batchDeleteCategories: (ids: number[]) => request.post('/admin/categories/batch-delete', { ids }),

  // Products
  getProducts: (params?: any) => request.get('/admin/products', { params }),
  createProduct: (data: any) => request.post('/admin/products', data),
  updateProduct: (id: number, data: any) => request.put(`/admin/products/${id}`, data),
  deleteProduct: (id: number) => request.delete(`/admin/products/${id}`),
  batchDeleteProducts: (ids: number[]) => request.post('/admin/products/batch-delete', { ids }),
  restockProduct: (id: number, data: any) => request.post(`/admin/products/${id}/restock`, data),

  // Cards
  getCards: (params?: any) => request.get('/admin/cards', { params }),
  importCards: (data: any) => request.post('/admin/cards/import', data),
  deleteCard: (id: number) => request.delete(`/admin/cards/${id}`),

  // Orders
  getOrders: (params?: any) => request.get('/admin/orders', { params }),
  getOrder: (id: number) => request.get(`/admin/orders/${id}`),

  // Payment Config
  getPaymentConfigs: () => request.get('/admin/payment-configs'),
  updatePaymentConfig: (channel: string, data: any) => request.put(`/admin/payment-configs/${channel}`, data),

  // AFF
  getAffUsers: (params?: any) => request.get('/admin/aff/users', { params }),
  updateAffSettings: (data: any) => request.put('/admin/aff/settings', data),
  getAffTiers: () => request.get('/admin/aff/tiers'),
  createAffTier: (data: any) => request.post('/admin/aff/tiers', data),
  updateAffTier: (level: number, data: any) => request.put(`/admin/aff/tiers/${level}`, data),
  deleteAffTier: (level: number) => request.delete(`/admin/aff/tiers/${level}`),

  // Withdrawals
  getWithdrawals: (params?: any) => request.get('/admin/withdrawals', { params }),
  approveWithdrawal: (id: number) => request.put(`/admin/withdrawals/${id}/approve`),
  rejectWithdrawal: (id: number, data?: any) => request.put(`/admin/withdrawals/${id}/reject`, data),
  completeWithdrawal: (id: number, data: any) => request.put(`/admin/withdrawals/${id}/complete`, data),

  // Admin management
  getAdmins: () => request.get('/admin/admins'),
  createAdmin: (data: any) => request.post('/admin/admins', data),
  deleteAdmin: (id: number) => request.delete(`/admin/admins/${id}`),

  // Settings
  getSettings: () => request.get('/admin/settings'),
  updateSettings: (data: any) => request.put('/admin/settings', data),
}
