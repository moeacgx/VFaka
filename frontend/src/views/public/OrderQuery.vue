<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { publicApi } from '../../api/public'

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

const statusMap: Record<string, { label: string; class: string }> = {
  pending: { label: '待支付', class: 'bg-yellow-50 text-yellow-700 border-yellow-200' },
  paid: { label: '已支付', class: 'bg-blue-50 text-blue-700 border-blue-200' },
  delivered: { label: '已发货', class: 'bg-green-50 text-green-700 border-green-200' },
  failed: { label: '失败', class: 'bg-red-50 text-red-700 border-red-200' },
}

function getStatus(status: string) {
  return statusMap[status] || { label: status, class: 'bg-gray-50 text-gray-700 border-gray-200' }
}

function formatTime(iso: string) {
  const d = new Date(iso)
  return d.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

function toggleExpand(orderNo: string) {
  expandedOrder.value = expandedOrder.value === orderNo ? null : orderNo
}

async function queryByEmail() {
  if (!email.value.trim()) {
    error.value = '请输入邮箱地址'
    return
  }
  loading.value = true
  error.value = ''
  searched.value = true

  try {
    const res = await publicApi.queryOrders(email.value.trim())
    orders.value = res.data
  } catch (e: any) {
    error.value = e.response?.data?.message || e.response?.data?.error || '查询失败，请重试'
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
    error.value = e.response?.data?.message || e.response?.data?.error || '查询失败，请重试'
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
    <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6 mb-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">订单查询</h2>
      <div class="flex gap-3">
        <input
          v-model="email"
          type="email"
          placeholder="请输入下单时使用的邮箱"
          @keyup.enter="queryByEmail"
          class="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <button
          @click="queryByEmail"
          :disabled="loading"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
        >
          <div v-if="loading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          查询
        </button>
      </div>
      <p v-if="error" class="text-red-500 text-sm mt-3">{{ error }}</p>
    </div>

    <!-- Results -->
    <div v-if="searched && !loading">
      <div v-if="orders.length === 0" class="text-center py-12">
        <p class="text-gray-400 text-sm">未找到订单</p>
      </div>

      <div v-else class="space-y-3">
        <div
          v-for="order in orders"
          :key="order.order_no"
          class="bg-white rounded-xl border border-gray-100 shadow-sm overflow-hidden transition-shadow hover:shadow-md"
        >
          <!-- Order row -->
          <div
            class="p-5 cursor-pointer"
            @click="order.status === 'delivered' ? toggleExpand(order.order_no) : null"
          >
            <div class="flex items-start justify-between mb-2">
              <div>
                <span class="text-sm font-mono text-gray-500">{{ order.order_no }}</span>
              </div>
              <span :class="['text-xs px-2.5 py-0.5 rounded-full border font-medium', getStatus(order.status).class]">
                {{ getStatus(order.status).label }}
              </span>
            </div>
            <div class="flex items-end justify-between">
              <div>
                <div class="text-base font-semibold text-gray-900">¥{{ order.total_amount.toFixed(2) }}</div>
                <div class="text-xs text-gray-400 mt-1">× {{ order.quantity }} · {{ formatTime(order.created_at) }}</div>
              </div>
              <div v-if="order.status === 'delivered'" class="text-xs text-blue-500">
                {{ expandedOrder === order.order_no ? '收起' : '查看卡密' }} ▾
              </div>
            </div>
          </div>

          <!-- Expanded card content -->
          <div v-if="expandedOrder === order.order_no && order.cards_snapshot" class="border-t border-gray-100 bg-gray-50 p-5">
            <div class="text-xs text-gray-500 mb-2">卡密内容</div>
            <pre class="text-sm text-gray-800 whitespace-pre-wrap break-all font-mono bg-white rounded-lg p-3 border border-gray-200">{{ order.cards_snapshot }}</pre>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>
