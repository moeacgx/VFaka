<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { publicApi } from '../../api/public'

const { t, locale } = useI18n()

interface Order {
  id: number
  order_no: string
  product_id: number
  quantity: number
  total_amount: number
  email: string
  payment_method: string
  payment_channel: string
  status: string
  trade_no: string | null
  pay_time: string | null
  aff_code: string | null
  cards_snapshot: string | null
  post_action_result: string | null
  created_at: string
  updated_at: string
}

const route = useRoute()

const email = ref('')
const orders = ref<Order[]>([])
const loading = ref(false)
const error = ref('')
const searched = ref(false)
const expandedOrder = ref<string | null>(null)

const statusMap = computed<Record<string, { label: string; class: string }>>(() => ({
  pending: { label: t('order.status.pending'), class: 'bg-yellow-50 text-yellow-700 border-yellow-200' },
  paid: { label: t('order.status.paid'), class: 'bg-blue-50 text-blue-700 border-blue-200' },
  delivered: { label: t('order.status.delivered'), class: 'bg-green-50 text-green-700 border-green-200' },
  failed: { label: t('order.status.failed'), class: 'bg-red-50 text-red-700 border-red-200' },
}))

function getStatus(status: string) {
  return statusMap.value[status] || { label: status, class: 'bg-gray-50 text-gray-700 border-gray-200' }
}

function formatTime(iso: string) {
  const d = new Date(iso)
  return d.toLocaleString(locale.value, { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

function toggleExpand(orderNo: string) {
  expandedOrder.value = expandedOrder.value === orderNo ? null : orderNo
}

async function queryByEmail() {
  if (!email.value.trim()) {
    error.value = t('common.enter_email_required')
    return
  }
  loading.value = true
  error.value = ''
  searched.value = true

  try {
    const res = await publicApi.queryOrders(email.value.trim())
    orders.value = res.data
  } catch (e: any) {
    error.value = e.response?.data?.message || e.response?.data?.error || t('common.query_failed')
    orders.value = []
  } finally {
    loading.value = false
  }
}

async function querySingleOrder(orderNo: string, orderEmail: string) {
  email.value = orderEmail
  loading.value = true
  error.value = ''
  searched.value = true

  try {
    const res = await publicApi.getOrder(orderNo, orderEmail)
    orders.value = [res.data]
  } catch (e: any) {
    error.value = e.response?.data?.message || e.response?.data?.error || t('common.query_failed')
    orders.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  const no = route.query.no as string | undefined
  const em = route.query.email as string | undefined
  if (no && em) {
    querySingleOrder(no, em)
  }
})
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16">
    <!-- Search -->
    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{{ $t('order.query_title') }}</h2>
      <div class="flex gap-3">
        <input
          v-model="email"
          type="email"
          :placeholder="$t('order.query_hint')"
          @keyup.enter="queryByEmail"
          class="flex-1 px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <button
          @click="queryByEmail"
          :disabled="loading"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
        >
          <div v-if="loading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          {{ $t('order.query_button') }}
        </button>
      </div>
      <p v-if="error" class="text-red-500 text-sm mt-3">{{ error }}</p>
    </div>

    <!-- Results -->
    <div v-if="searched && !loading">
      <div v-if="orders.length === 0" class="text-center py-12">
        <p class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('order.no_orders') }}</p>
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="order in orders"
          :key="order.order_no"
          class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none overflow-hidden transition-shadow hover:shadow-md"
        >
          <!-- Order row -->
          <div
            class="p-5 cursor-pointer"
            @click="order.status === 'delivered' ? toggleExpand(order.order_no) : null"
          >
            <div class="flex items-start justify-between mb-2">
              <div>
                <span class="text-sm font-mono text-gray-500 dark:text-gray-400">{{ order.order_no }}</span>
              </div>
              <span :class="['text-xs px-2.5 py-0.5 rounded-full border font-medium', getStatus(order.status).class]">
                {{ getStatus(order.status).label }}
              </span>
            </div>
            <div class="flex items-end justify-between">
              <div>
                <div class="text-base font-semibold text-gray-900 dark:text-white">¥{{ order.total_amount.toFixed(2) }}</div>
                <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">× {{ order.quantity }} · {{ formatTime(order.created_at) }}</div>
              </div>
              <div v-if="order.status === 'delivered'" class="text-xs text-blue-500">
                {{ expandedOrder === order.order_no ? $t('common.close') : $t('order.cards_snapshot') }} ▾
              </div>
            </div>
          </div>

          <!-- Expanded card content -->
          <div v-if="expandedOrder === order.order_no && order.cards_snapshot" class="border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 p-5">
            <div class="text-xs text-gray-500 dark:text-gray-400 mb-2">{{ $t('order.cards_snapshot') }}</div>
            <pre class="text-sm text-gray-800 dark:text-gray-100 whitespace-pre-wrap break-all font-mono bg-white dark:bg-gray-800 rounded-lg p-3 border border-gray-200 dark:border-gray-700">{{ order.cards_snapshot }}</pre>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>
