import request from './request'

// Public API calls will be added here as features are implemented
export const publicApi = {
  // Site info
  getSiteInfo: () => request.get('/v1/site-info'),
  getPublicConfig: () => request.get('/v1/config'),

  // Announcement
  getAnnouncement: () => request.get('/v1/announcement'),

  // Products
  getCategories: () => request.get('/v1/categories'),
  getProducts: (params?: { category_id?: number }) => request.get('/v1/products', { params }),
  getProduct: (id: number) => request.get(`/v1/products/${id}`),

  // Orders
  createOrder: (data: any) => request.post('/v1/orders', data),
  getOrder: (orderNo: string, params: { token?: string; email?: string }) =>
    request.get(`/v1/orders/${orderNo}`, { params }),

  // AFF
  registerAff: (data: any) => request.post('/v1/aff/register', data),
  queryAff: (code: string, password: string) => request.get('/v1/aff/query', { params: { code, password } }),
  submitWithdrawal: (data: any) => request.post('/v1/aff/withdraw', data),
  getAffLogs: (code: string, password: string) => request.get('/v1/aff/logs', { params: { code, password } }),
  getAffTiers: () => request.get('/v1/aff/tiers'),

  // Coupons
  validateCoupon: (data: { code: string; product_id: number; amount: number }) => request.post('/v1/coupons/validate', data),
}
