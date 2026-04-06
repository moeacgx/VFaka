<script setup lang="ts">
import { ref, computed } from 'vue'
import { publicApi } from '../../api/public'

interface AffInfo {
  email: string
  aff_code: string
  balance: number
  total_earned: number
  total_withdrawn: number
  created_at: string
}

interface AffLog {
  id: number
  aff_user_id: number
  order_id: number
  commission: number
  status: string
  created_at: string
}

// Section 1: Balance query
const queryEmail = ref('')
const affInfo = ref<AffInfo | null>(null)
const queryLoading = ref(false)
const queryError = ref('')
const notRegistered = ref(false)
const copied = ref(false)

// Section 2: Register
const regEmail = ref('')
const regPassword = ref('')
const regConfirm = ref('')
const regLoading = ref(false)
const regError = ref('')
const regSuccess = ref<{ aff_code: string; email: string } | null>(null)

// Section 3: Withdraw
const wdAmount = ref<number | undefined>()
const wdCurrency = ref('USDT')
const wdChain = ref('Tron')
const wdAddress = ref('')
const wdPassword = ref('')
const wdLoading = ref(false)
const wdError = ref('')
const wdSuccess = ref('')

// Logs
const affLogs = ref<AffLog[]>([])
const logsLoading = ref(false)

const affLink = computed(() => {
  if (!affInfo.value) return ''
  return `${window.location.origin}/?aff=${affInfo.value.aff_code}`
})

const showWithdraw = computed(() => affInfo.value && affInfo.value.balance > 0)

function formatTime(iso: string) {
  const d = new Date(iso)
  return d.toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

async function copyLink() {
  if (!affLink.value) return
  try {
    await navigator.clipboard.writeText(affLink.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = affLink.value
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  }
}

async function queryBalance() {
  if (!queryEmail.value.trim()) {
    queryError.value = '请输入邮箱地址'
    return
  }
  queryLoading.value = true
  queryError.value = ''
  notRegistered.value = false
  affInfo.value = null

  try {
    const res = await publicApi.queryAff(queryEmail.value.trim())
    affInfo.value = res.data
    // Also load logs
    loadLogs()
  } catch (e: any) {
    const status = e.response?.status
    if (status === 404) {
      notRegistered.value = true
      regEmail.value = queryEmail.value.trim()
    } else {
      queryError.value = e.response?.data?.message || e.response?.data?.error || '查询失败'
    }
  } finally {
    queryLoading.value = false
  }
}

async function loadLogs() {
  if (!affInfo.value) return
  logsLoading.value = true
  try {
    const res = await publicApi.getAffLogs(affInfo.value.email)
    affLogs.value = res.data
  } catch {
    // silent
  } finally {
    logsLoading.value = false
  }
}

async function register() {
  if (!regEmail.value.trim()) {
    regError.value = '请输入邮箱地址'
    return
  }
  if (!regPassword.value) {
    regError.value = '请输入提现密码'
    return
  }
  if (regPassword.value !== regConfirm.value) {
    regError.value = '两次密码输入不一致'
    return
  }
  if (regPassword.value.length < 6) {
    regError.value = '密码长度至少6位'
    return
  }

  regLoading.value = true
  regError.value = ''

  try {
    const res = await publicApi.registerAff({
      email: regEmail.value.trim(),
      withdraw_password: regPassword.value,
    })
    regSuccess.value = res.data
    // Auto-query after registration
    queryEmail.value = regEmail.value.trim()
    notRegistered.value = false
    await queryBalance()
  } catch (e: any) {
    regError.value = e.response?.data?.message || e.response?.data?.error || '注册失败'
  } finally {
    regLoading.value = false
  }
}

async function submitWithdraw() {
  if (!affInfo.value) return
  if (!wdAmount.value || wdAmount.value <= 0) {
    wdError.value = '请输入有效金额'
    return
  }
  if (wdAmount.value > affInfo.value.balance) {
    wdError.value = '提现金额不能超过可用余额'
    return
  }
  if (!wdAddress.value.trim()) {
    wdError.value = '请输入钱包地址'
    return
  }
  if (!wdPassword.value) {
    wdError.value = '请输入提现密码'
    return
  }

  wdLoading.value = true
  wdError.value = ''
  wdSuccess.value = ''

  try {
    await publicApi.submitWithdrawal({
      email: affInfo.value.email,
      password: wdPassword.value,
      amount: wdAmount.value,
      currency: wdCurrency.value,
      chain: wdChain.value,
      wallet_address: wdAddress.value.trim(),
    })
    wdSuccess.value = '提现申请已提交，请等待审核'
    wdAmount.value = undefined
    wdAddress.value = ''
    wdPassword.value = ''
    // Refresh balance
    await queryBalance()
  } catch (e: any) {
    wdError.value = e.response?.data?.message || e.response?.data?.error || '提现失败'
  } finally {
    wdLoading.value = false
  }
}
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16 space-y-6">
    <!-- Section 1: Balance Query -->
    <div class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">推广收益查询</h2>
      <div class="flex gap-3">
        <input
          v-model="queryEmail"
          type="email"
          placeholder="请输入注册邮箱"
          @keyup.enter="queryBalance"
          class="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <button
          @click="queryBalance"
          :disabled="queryLoading"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
        >
          <div v-if="queryLoading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          查询
        </button>
      </div>
      <p v-if="queryError" class="text-red-500 text-sm mt-3">{{ queryError }}</p>

      <!-- Balance info -->
      <div v-if="affInfo" class="mt-5">
        <div class="grid grid-cols-3 gap-4 mb-4">
          <div class="bg-gray-50 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 mb-1">可用余额</div>
            <div class="text-xl font-bold text-gray-900">¥{{ affInfo.balance.toFixed(2) }}</div>
          </div>
          <div class="bg-gray-50 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 mb-1">总收益</div>
            <div class="text-xl font-bold text-gray-900">¥{{ affInfo.total_earned.toFixed(2) }}</div>
          </div>
          <div class="bg-gray-50 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 mb-1">已提现</div>
            <div class="text-xl font-bold text-gray-900">¥{{ affInfo.total_withdrawn.toFixed(2) }}</div>
          </div>
        </div>

        <!-- Aff link -->
        <div class="bg-gray-50 rounded-xl p-4">
          <div class="text-xs text-gray-400 mb-1.5">推广链接</div>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-xs text-gray-600 bg-white px-3 py-2 rounded-lg border border-gray-200 truncate">{{ affLink }}</code>
            <button
              @click="copyLink"
              class="px-3 py-2 text-xs font-medium rounded-lg border transition-all"
              :class="copied ? 'bg-green-50 text-green-600 border-green-200' : 'bg-white text-gray-600 border-gray-200 hover:bg-gray-50'"
            >{{ copied ? '已复制' : '复制' }}</button>
          </div>
          <div class="text-xs text-gray-400 mt-2">推广码: <span class="font-mono font-medium text-gray-600">{{ affInfo.aff_code }}</span></div>
        </div>
      </div>
    </div>

    <!-- Section 2: Register (show if not registered) -->
    <div v-if="notRegistered" class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-1">注册推广账号</h2>
      <p class="text-sm text-gray-400 mb-4">该邮箱尚未注册，请设置提现密码完成注册</p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">邮箱</label>
          <input
            v-model="regEmail"
            type="email"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm bg-gray-50 focus:outline-none"
            readonly
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">提现密码</label>
          <input
            v-model="regPassword"
            type="password"
            placeholder="至少6位"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">确认密码</label>
          <input
            v-model="regConfirm"
            type="password"
            placeholder="再次输入密码"
            @keyup.enter="register"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>
      </div>

      <p v-if="regError" class="text-red-500 text-sm mt-3">{{ regError }}</p>

      <button
        @click="register"
        :disabled="regLoading"
        class="mt-4 w-full py-2.5 bg-blue-600 text-white rounded-xl text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
      >
        <div v-if="regLoading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
        {{ regLoading ? '注册中...' : '注册' }}
      </button>
    </div>

    <!-- Section 3: Withdraw -->
    <div v-if="showWithdraw" class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">申请提现</h2>

      <div class="space-y-4">
        <!-- Amount -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">提现金额</label>
          <input
            v-model.number="wdAmount"
            type="number"
            step="0.01"
            :placeholder="`最多 ${affInfo?.balance.toFixed(2)}`"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>

        <!-- Currency -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">币种</label>
          <div class="flex gap-2">
            <label
              v-for="c in ['USDT', 'USDC']"
              :key="c"
              :class="[
                'px-4 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                wdCurrency === c
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'
              ]"
            >
              <input type="radio" v-model="wdCurrency" :value="c" class="sr-only" />
              {{ c }}
            </label>
          </div>
        </div>

        <!-- Chain -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">链</label>
          <div class="flex gap-2">
            <label
              v-for="c in ['Tron', 'Polygon', 'Base']"
              :key="c"
              :class="[
                'px-4 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                wdChain === c
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'
              ]"
            >
              <input type="radio" v-model="wdChain" :value="c" class="sr-only" />
              {{ c }}
            </label>
          </div>
        </div>

        <!-- Wallet address -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">钱包地址</label>
          <input
            v-model="wdAddress"
            type="text"
            placeholder="请输入收款钱包地址"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>

        <!-- Password -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">提现密码</label>
          <input
            v-model="wdPassword"
            type="password"
            placeholder="请输入提现密码"
            class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>
      </div>

      <p v-if="wdError" class="text-red-500 text-sm mt-3">{{ wdError }}</p>
      <p v-if="wdSuccess" class="text-green-600 text-sm mt-3">{{ wdSuccess }}</p>

      <button
        @click="submitWithdraw"
        :disabled="wdLoading"
        class="mt-4 w-full py-2.5 bg-blue-600 text-white rounded-xl text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
      >
        <div v-if="wdLoading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
        {{ wdLoading ? '提交中...' : '提交提现' }}
      </button>
    </div>

    <!-- Aff logs -->
    <div v-if="affInfo && affLogs.length > 0" class="bg-white rounded-xl border border-gray-100 shadow-sm p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">佣金记录</h2>
      <div class="space-y-2">
        <div
          v-for="log in affLogs"
          :key="log.id"
          class="flex items-center justify-between py-3 border-b border-gray-50 last:border-0"
        >
          <div>
            <div class="text-sm text-gray-700">订单 #{{ log.order_id }}</div>
            <div class="text-xs text-gray-400">{{ formatTime(log.created_at) }}</div>
          </div>
          <div class="text-right">
            <div class="text-sm font-semibold text-green-600">+¥{{ log.commission.toFixed(2) }}</div>
            <div class="text-xs text-gray-400">{{ log.status === 'credited' ? '已入账' : log.status }}</div>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>
