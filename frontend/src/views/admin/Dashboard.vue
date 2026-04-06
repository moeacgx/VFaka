<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const stats = ref({ today_orders: 0, today_income: 0, total_products: 0, low_stock_count: 0 })
const recentOrders = ref<any[]>([])

const statusMap: Record<string, string> = {
  pending: '待支付',
  paid: '已支付',
  delivered: '已发货',
  expired: '已过期',
  failed: '失败',
}

onMounted(async () => {
  try {
    const res = await adminApi.getDashboard()
    stats.value = res.data.stats || stats.value
    recentOrders.value = res.data.recent_orders || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 text-sm">加载中...</div>
    <template v-else>
      <!-- Stats cards -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div class="bg-white rounded-lg border border-gray-200 p-5">
          <div class="text-sm text-gray-500 mb-1">今日订单数</div>
          <div class="text-2xl font-semibold text-gray-800">{{ stats.today_orders }}</div>
        </div>
        <div class="bg-white rounded-lg border border-gray-200 p-5">
          <div class="text-sm text-gray-500 mb-1">今日收入</div>
          <div class="text-2xl font-semibold text-gray-800">¥{{ stats.today_income?.toFixed(2) }}</div>
        </div>
        <div class="bg-white rounded-lg border border-gray-200 p-5">
          <div class="text-sm text-gray-500 mb-1">商品总数</div>
          <div class="text-2xl font-semibold text-gray-800">{{ stats.total_products }}</div>
        </div>
        <div class="bg-white rounded-lg border border-gray-200 p-5">
          <div class="text-sm text-gray-500 mb-1">库存预警</div>
          <div class="text-2xl font-semibold" :class="stats.low_stock_count > 0 ? 'text-red-600' : 'text-gray-800'">
            {{ stats.low_stock_count }}
          </div>
        </div>
      </div>

      <!-- Recent orders -->
      <div class="bg-white rounded-lg border border-gray-200">
        <div class="px-5 py-4 border-b border-gray-200">
          <h3 class="text-sm font-medium text-gray-800">最近订单</h3>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 text-left text-gray-500">
                <th class="px-4 py-3 font-medium">订单号</th>
                <th class="px-4 py-3 font-medium">商品</th>
                <th class="px-4 py-3 font-medium">金额</th>
                <th class="px-4 py-3 font-medium">邮箱</th>
                <th class="px-4 py-3 font-medium">状态</th>
                <th class="px-4 py-3 font-medium">时间</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              <tr v-for="order in recentOrders" :key="order.id" class="hover:bg-gray-50">
                <td class="px-4 py-3 text-gray-600 font-mono text-xs">{{ order.order_no }}</td>
                <td class="px-4 py-3 text-gray-800">{{ order.product_name }}</td>
                <td class="px-4 py-3 text-gray-800">¥{{ order.amount?.toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600">{{ order.email }}</td>
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
              <tr v-if="recentOrders.length === 0">
                <td colspan="6" class="px-4 py-8 text-center text-gray-400">暂无订单</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>
