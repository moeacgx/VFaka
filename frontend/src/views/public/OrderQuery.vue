<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
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
  post_action_status: string | null
  created_at: string
  updated_at: string
}

const route = useRoute()

const queryMode = ref<'order' | 'email'>('order')
const orderNo = ref('')
const queryToken = ref('')
const emailQuery = ref('')
const emailPassword = ref('')
const orderNoInput = ref<HTMLInputElement | null>(null)
const tokenInput = ref<HTMLInputElement | null>(null)
const emailInput = ref<HTMLInputElement | null>(null)
const emailPasswordInput = ref<HTMLInputElement | null>(null)
const orders = ref<Order[]>([])
const loading = ref(false)
const error = ref('')
const searched = ref(false)
const expandedOrder = ref<string | null>(null)
const autoQueried = ref(false)

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

function switchMode(mode: 'order' | 'email') {
  queryMode.value = mode
  error.value = ''
  nextTick(() => {
    if (mode === 'email') emailInput.value?.focus()
    else orderNoInput.value?.focus()
  })
}

async function queryOrder() {
  if (queryMode.value === 'email') {
    return queryByEmail()
  }

  if (!orderNo.value.trim()) {
    error.value = t('order.enter_order_no_required')
    return
  }
  if (!queryToken.value.trim()) {
    error.value = t('order.enter_token_required')
    return
  }
  loading.value = true
  error.value = ''
  searched.value = true

  try {
    const res = await publicApi.getOrder(orderNo.value.trim(), { token: queryToken.value.trim() })
    orders.value = [res.data]
  } catch (e: any) {
    error.value = e.response?.data?.message || e.response?.data?.error || t('common.query_failed')
    orders.value = []
  } finally {
    loading.value = false
  }
}

async function queryByEmail() {
  if (!emailQuery.value.trim()) {
    error.value = t('order.enter_email_required')
    return
  }
  if (!emailPassword.value.trim()) {
    error.value = t('order.enter_password_required')
    return
  }
  loading.value = true
  error.value = ''
  searched.value = true

  try {
    const res = await publicApi.getOrdersByEmail(emailQuery.value.trim(), emailPassword.value.trim())
    orders.value = res.data || []
  } catch (e: any) {
    error.value = e.response?.data?.message || e.response?.data?.error || t('common.query_failed')
    orders.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  const no = route.query.no as string | undefined
  const token = route.query.token as string | undefined
  if (no && token) {
    orderNo.value = no
    queryToken.value = token
    autoQueried.value = true
    queryOrder()
  } else if (no) {
    orderNo.value = no
    nextTick(() => tokenInput.value?.focus())
  } else {
    nextTick(() => orderNoInput.value?.focus())
  }
})
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16">
    <!-- Search -->
    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{{ $t('order.query_title') }}</h2>
      <div v-if="autoQueried && orders.length > 0" class="text-sm text-green-600 dark:text-green-400 mb-3">
        {{ $t('order.auto_query_success') }}
      </div>

      <!-- Mode tabs -->
      <div class="flex mb-4 border-b border-gray-200 dark:border-gray-700">
        <button
          @click="switchMode('order')"
          :class="['px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors', queryMode === 'order' ? 'border-blue-600 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200']"
        >{{ $t('order.query_by_order') }}</button>
        <button
          @click="switchMode('email')"
          :class="['px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors', queryMode === 'email' ? 'border-blue-600 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200']"
        >{{ $t('order.query_by_email') }}</button>
      </div>

      <!-- Order No + Token mode -->
      <div v-if="queryMode === 'order'" class="space-y-3">
        <input
          ref="orderNoInput"
          v-model="orderNo"
          type="text"
          :placeholder="$t('order.enter_order_no')"
          @keyup.enter="tokenInput?.focus()"
          class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <div class="flex gap-3">
          <input
            ref="tokenInput"
            v-model="queryToken"
            type="text"
            :placeholder="$t('order.enter_token')"
            @keyup.enter="queryOrder"
            class="flex-1 px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
          <button
            @click="queryOrder"
            :disabled="loading"
            class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
          >
            <div v-if="loading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            {{ $t('order.query_button') }}
          </button>
        </div>
      </div>

      <!-- Email mode -->
      <div v-else class="space-y-3">
        <input
          ref="emailInput"
          v-model="emailQuery"
          type="email"
          :placeholder="$t('order.enter_email')"
          @keyup.enter="emailPasswordInput?.focus()"
          class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <div class="flex gap-3">
          <input
            ref="emailPasswordInput"
            v-model="emailPassword"
            type="text"
            :placeholder="$t('order.enter_query_password')"
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
            <div v-if="order.status === 'pending'" class="mt-2 text-xs text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 px-3 py-1.5 rounded-lg">
              {{ $t('order.pending_payment_hint') }}
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
