<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const orders = ref<any[]>([])
const page = ref(1)
const totalPages = ref(1)
const filterStatus = ref('')
const filterEmail = ref('')
const filterDateStart = ref('')
const filterDateEnd = ref('')
const selectedOrder = ref<any>(null)
const detailLoading = ref(false)

const statusMap: Record<string, string> = {
  pending: '待支付', paid: '已支付', delivered: '已发货', expired: '已过期', failed: '失败',
}
const payMethodMap: Record<string, string> = {
  alipay: '支付宝', wechat: '微信', qqpay: 'QQ钱包',
  usdt_trc20: 'USDT-TRC20', usdt_erc20: 'USDT-ERC20', usdt_polygon: 'USDT-Polygon',
}

async function load() {
  loading.value = true
  try {
    const params: any = { page: page.value, per_page: 20 }
    if (filterStatus.value) params.status = filterStatus.value
    if (filterEmail.value) params.email = filterEmail.value
    if (filterDateStart.value) params.date_start = filterDateStart.value
    if (filterDateEnd.value) params.date_end = filterDateEnd.value
    const res = await adminApi.getOrders(params)
    orders.value = res.data?.orders || res.data || []
    totalPages.value = res.data?.total_pages || 1
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function viewDetail(order: any) {
  detailLoading.value = true
  try {
    const res = await adminApi.getOrder(order.id)
    selectedOrder.value = res.data
  } catch (e) {
    selectedOrder.value = order
  } finally {
    detailLoading.value = false
  }
}

function changePage(p: number) {
  if (p < 1 || p > totalPages.value) return
  page.value = p
  load()
}

function applyFilters() {
  page.value = 1
  load()
}

onMounted(load)
</script>

<template>
  <div>
    <!-- Filters -->
    <div class="flex flex-col sm:flex-row flex-wrap items-start sm:items-center gap-2 mb-4">
      <select v-model="filterStatus" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
        <option value="">全部状态</option>
        <option value="pending">待支付</option>
        <option value="paid">已支付</option>
        <option value="delivered">已发货</option>
        <option value="expired">已过期</option>
        <option value="failed">失败</option>
      </select>
      <input v-model="filterEmail" placeholder="邮箱搜索" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
      <input v-model="filterDateStart" type="date" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
      <span class="text-gray-400 text-sm">至</span>
      <input v-model="filterDateEnd" type="date" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
      <button @click="applyFilters" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">搜索</button>
    </div>

    <!-- Order detail modal -->
    <div v-if="selectedOrder" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="selectedOrder = null">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-lg max-h-[80vh] overflow-y-auto p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">订单详情</h3>
        <div v-if="detailLoading" class="text-gray-400 text-sm">加载中...</div>
        <div v-else class="space-y-3 text-sm">
          <div class="flex justify-between"><span class="text-gray-500">订单号</span><span class="text-gray-800 font-mono">{{ selectedOrder.order_no }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">商品</span><span class="text-gray-800">{{ selectedOrder.product_name }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">金额</span><span class="text-gray-800">¥{{ selectedOrder.amount?.toFixed(2) }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">数量</span><span class="text-gray-800">{{ selectedOrder.quantity }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">邮箱</span><span class="text-gray-800">{{ selectedOrder.email }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">支付方式</span><span class="text-gray-800">{{ payMethodMap[selectedOrder.pay_method] || selectedOrder.pay_method }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">状态</span><span class="text-gray-800">{{ statusMap[selectedOrder.status] || selectedOrder.status }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500">创建时间</span><span class="text-gray-800">{{ selectedOrder.created_at?.replace('T', ' ').slice(0, 19) }}</span></div>
          <div v-if="selectedOrder.cards_snapshot" class="pt-2 border-t border-gray-100">
            <div class="text-gray-500 mb-2">卡密信息</div>
            <div class="bg-gray-50 rounded p-3 font-mono text-xs whitespace-pre-wrap break-all">{{ selectedOrder.cards_snapshot }}</div>
          </div>
        </div>
        <div class="flex justify-end pt-4">
          <button @click="selectedOrder = null" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">关闭</button>
        </div>
      </div>
    </div>

    <!-- Table -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
      <div v-if="loading" class="p-8 text-center text-gray-400 text-sm">加载中...</div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="bg-gray-50 text-left text-gray-500">
            <th class="px-4 py-3 font-medium">订单号</th>
            <th class="px-4 py-3 font-medium">商品</th>
            <th class="px-4 py-3 font-medium">金额</th>
            <th class="px-4 py-3 font-medium">邮箱</th>
            <th class="px-4 py-3 font-medium">支付方式</th>
            <th class="px-4 py-3 font-medium">状态</th>
            <th class="px-4 py-3 font-medium">时间</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="order in orders" :key="order.id" class="hover:bg-gray-50 cursor-pointer" @click="viewDetail(order)">
            <td class="px-4 py-3 text-gray-600 font-mono text-xs">{{ order.order_no }}</td>
            <td class="px-4 py-3 text-gray-800">{{ order.product_name }}</td>
            <td class="px-4 py-3 text-gray-800">¥{{ order.amount?.toFixed(2) }}</td>
            <td class="px-4 py-3 text-gray-600">{{ order.email }}</td>
            <td class="px-4 py-3 text-gray-600">{{ payMethodMap[order.pay_method] || order.pay_method }}</td>
            <td class="px-4 py-3">
              <span
                :class="{
                  'text-yellow-600 bg-yellow-50': order.status === 'pending',
                  'text-green-600 bg-green-50': order.status === 'paid' || order.status === 'delivered',
                  'text-gray-500 bg-gray-100': order.status === 'expired',
                  'text-red-600 bg-red-50': order.status === 'failed',
                }"
                class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              >
                {{ statusMap[order.status] || order.status }}
              </span>
            </td>
            <td class="px-4 py-3 text-gray-500 text-xs">{{ order.created_at?.replace('T', ' ').slice(0, 19) }}</td>
          </tr>
          <tr v-if="orders.length === 0">
            <td colspan="7" class="px-4 py-8 text-center text-gray-400">暂无订单</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-2 mt-4">
      <button @click="changePage(page - 1)" :disabled="page <= 1" class="px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed">上一页</button>
      <span class="text-sm text-gray-600">第 {{ page }} / {{ totalPages }} 页</span>
      <button @click="changePage(page + 1)" :disabled="page >= totalPages" class="px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed">下一页</button>
    </div>
  </div>
</template>
