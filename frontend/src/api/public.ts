import request from './request'

// Public API calls will be added here as features are implemented
export const publicApi = {
  // Site info
  getSiteInfo: () => request.get('/v1/site-info'),
  getPublicConfig: () => request.get('/v1/config'),

  // Products
  getCategories: () => request.get('/v1/categories'),
  getProducts: (params?: { category_id?: number }) => request.get('/v1/products', { params }),
  getProduct: (id: number) => request.get(`/v1/products/${id}`),

  // Orders
  createOrder: (data: any) => request.post('/v1/orders', data),
  queryOrders: (email: string) => request.get('/v1/orders/query', { params: { email } }),
  getOrder: (orderNo: string, email: string) => request.get(`/v1/orders/${orderNo}`, { params: { email } }),

  // AFF
  registerAff: (data: any) => request.post('/v1/aff/register', data),
  queryAff: (email: string) => request.get('/v1/aff/query', { params: { email } }),
  submitWithdrawal: (data: any) => request.post('/v1/aff/withdraw', data),
  getAffLogs: (email: string) => request.get('/v1/aff/logs', { params: { email } }),
  getAffTiers: () => request.get('/v1/aff/tiers'),
}
