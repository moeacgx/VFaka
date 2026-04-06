<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
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

const statusMap = computed<Record<string, string>>(() => ({
  pending: t('order.status.pending'),
  paid: t('order.status.paid'),
  delivered: t('order.status.delivered'),
  expired: t('order.status.expired'),
  failed: t('order.status.failed'),
}))

const payMethodMap = computed<Record<string, string>>(() => ({
  alipay: t('product.alipay'),
  wechat: t('product.wxpay'),
  qqpay: t('product.qqpay'),
  usdt_trc20: t('product.usdt_trc20'),
  usdt_erc20: t('product.usdt_erc20'),
  usdt_polygon: t('product.usdt_polygon'),
}))

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
      <select v-model="filterStatus" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
        <option value="">{{ $t('common.all') }} {{ $t('common.status') }}</option>
        <option value="pending">{{ $t('order.status.pending') }}</option>
        <option value="paid">{{ $t('order.status.paid') }}</option>
        <option value="delivered">{{ $t('order.status.delivered') }}</option>
        <option value="expired">{{ $t('order.status.expired') }}</option>
        <option value="failed">{{ $t('order.status.failed') }}</option>
      </select>
      <input v-model="filterEmail" :placeholder="$t('order.email')" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
      <input v-model="filterDateStart" type="date" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
      <span class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.to') }}</span>
      <input v-model="filterDateEnd" type="date" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
      <button @click="applyFilters" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">{{ $t('common.search') }}</button>
    </div>

    <!-- Order detail modal -->
    <div v-if="selectedOrder" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="selectedOrder = null">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-lg max-h-[80vh] overflow-y-auto p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('order.detail') }}</h3>
        <div v-if="detailLoading" class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
        <div v-else class="space-y-3 text-sm">
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.order_no') }}</span><span class="text-gray-800 dark:text-gray-100 font-mono">{{ selectedOrder.order_no }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.product_name') }}</span><span class="text-gray-800 dark:text-gray-100">{{ selectedOrder.product_name }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.total_amount') }}</span><span class="text-gray-800 dark:text-gray-100">¥{{ selectedOrder.amount?.toFixed(2) }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.quantity') }}</span><span class="text-gray-800 dark:text-gray-100">{{ selectedOrder.quantity }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.email') }}</span><span class="text-gray-800 dark:text-gray-100">{{ selectedOrder.email }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('order.payment_method') }}</span><span class="text-gray-800 dark:text-gray-100">{{ payMethodMap[selectedOrder.pay_method] || selectedOrder.pay_method }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('common.status') }}</span><span class="text-gray-800 dark:text-gray-100">{{ statusMap[selectedOrder.status] || selectedOrder.status }}</span></div>
          <div class="flex justify-between"><span class="text-gray-500 dark:text-gray-400">{{ $t('common.created_at') }}</span><span class="text-gray-800 dark:text-gray-100">{{ selectedOrder.created_at?.replace('T', ' ').slice(0, 19) }}</span></div>
          <div v-if="selectedOrder.cards_snapshot" class="pt-2 border-t border-gray-100 dark:border-gray-700">
            <div class="text-gray-500 dark:text-gray-400 mb-2">{{ $t('order.cards_snapshot') }}</div>
            <div class="bg-gray-50 dark:bg-gray-900 rounded p-3 font-mono text-xs whitespace-pre-wrap break-all text-gray-800 dark:text-gray-100">{{ selectedOrder.cards_snapshot }}</div>
          </div>
        </div>
        <div class="flex justify-end pt-4">
          <button @click="selectedOrder = null" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.close') }}</button>
        </div>
      </div>
    </div>

    <!-- Table -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-x-auto">
      <div v-if="loading" class="p-8 text-center text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="bg-gray-50 dark:bg-gray-900 text-left text-gray-500 dark:text-gray-400">
            <th class="px-4 py-3 font-medium">{{ $t('order.order_no') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.product_name') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.total_amount') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.email') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.payment_method') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.time') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="order in orders" :key="order.id" class="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer" @click="viewDetail(order)">
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300 font-mono text-xs">{{ order.order_no }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ order.product_name }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">¥{{ order.amount?.toFixed(2) }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ order.email }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ payMethodMap[order.pay_method] || order.pay_method }}</td>
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
          <tr v-if="orders.length === 0">
            <td colspan="7" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-2 mt-4">
      <button @click="changePage(page - 1)" :disabled="page <= 1" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed dark:text-gray-300">{{ $t('common.prev_page') }}</button>
      <span class="text-sm text-gray-600 dark:text-gray-300">{{ $t('common.page_info', { current: page, total: totalPages }) }}</span>
      <button @click="changePage(page + 1)" :disabled="page >= totalPages" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed dark:text-gray-300">{{ $t('common.next_page') }}</button>
    </div>
  </div>
</template>
