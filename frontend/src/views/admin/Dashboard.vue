<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const stats = ref({ today_orders: 0, today_income: 0, total_products: 0, low_stock_count: 0 })
const recentOrders = ref<any[]>([])

const statusMap = computed<Record<string, string>>(() => ({
  pending: t('order.status.pending'),
  paid: t('order.status.paid'),
  delivered: t('order.status.delivered'),
  expired: t('order.status.expired'),
  failed: t('order.status.failed'),
}))

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
    <div v-if="loading" class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
    <template v-else>
      <!-- Stats cards -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
          <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">{{ $t('dashboard.today_orders') }}</div>
          <div class="text-2xl font-semibold text-gray-800 dark:text-gray-100">{{ stats.today_orders }}</div>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
          <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">{{ $t('dashboard.today_income') }}</div>
          <div class="text-2xl font-semibold text-gray-800 dark:text-gray-100">¥{{ stats.today_income?.toFixed(2) }}</div>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
          <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">{{ $t('dashboard.total_products') }}</div>
          <div class="text-2xl font-semibold text-gray-800 dark:text-gray-100">{{ stats.total_products }}</div>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
          <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">{{ $t('dashboard.low_stock') }}</div>
          <div class="text-2xl font-semibold" :class="stats.low_stock_count > 0 ? 'text-red-600' : 'text-gray-800 dark:text-gray-100'">
            {{ stats.low_stock_count }}
          </div>
        </div>
      </div>

      <!-- Recent orders -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div class="px-5 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-800 dark:text-gray-100">{{ $t('dashboard.recent_orders') }}</h3>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 dark:bg-gray-900 text-left text-gray-500 dark:text-gray-400">
                <th class="px-4 py-3 font-medium">{{ $t('dashboard.order_no') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('dashboard.product') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('dashboard.amount') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.email') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.created_at') }}</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
              <tr v-for="order in recentOrders" :key="order.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300 font-mono text-xs">{{ order.order_no }}</td>
                <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ order.product_name }}</td>
                <td class="px-4 py-3 text-gray-800 dark:text-gray-100">¥{{ order.total_amount?.toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ order.email }}</td>
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
                <td class="px-4 py-3 text-gray-500 dark:text-gray-400 text-xs">{{ order.created_at?.replace('T', ' ').slice(0, 19) }}</td>
              </tr>
              <tr v-if="recentOrders.length === 0">
                <td colspan="6" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>
