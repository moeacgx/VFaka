<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { publicApi } from '../../api/public'

const { t, locale } = useI18n()

interface AffInfo {
  email: string
  aff_code: string
  balance: number
  total_earned: number
  total_withdrawn: number
  level: number
  level_name: string
  commission_rate: number
  next_level: { level: number; name: string; commission_rate: number; required_amount: number; remaining: number } | null
  created_at: string
}

interface AffTier {
  id: number
  level: number
  name: string
  commission_rate: number
  required_amount: number
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
const queryCode = ref('')
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
const allTiers = ref<AffTier[]>([])

const affLink = computed(() => {
  if (!affInfo.value) return ''
  return `${window.location.origin}/?aff=${affInfo.value.aff_code}`
})

const showWithdraw = computed(() => affInfo.value && affInfo.value.balance > 0)

function formatTime(iso: string) {
  const d = new Date(iso)
  return d.toLocaleString(locale.value, { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
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
  if (!queryCode.value.trim()) {
    queryError.value = t('aff.enter_code_required')
    return
  }
  queryLoading.value = true
  queryError.value = ''
  notRegistered.value = false
  affInfo.value = null

  try {
    const [res, tiersRes] = await Promise.all([
      publicApi.queryAff(queryCode.value.trim()),
      publicApi.getAffTiers(),
    ])
    affInfo.value = res.data
    allTiers.value = tiersRes.data || []
    // Also load logs
    loadLogs()
  } catch (e: any) {
    const status = e.response?.status
    if (status === 404) {
      notRegistered.value = true
    } else {
      queryError.value = e.response?.data?.message || e.response?.data?.error || t('common.query_failed')
    }
  } finally {
    queryLoading.value = false
  }
}

async function loadLogs() {
  if (!affInfo.value) return
  logsLoading.value = true
  try {
    const res = await publicApi.getAffLogs(affInfo.value.aff_code)
    affLogs.value = res.data
  } catch {
    // silent
  } finally {
    logsLoading.value = false
  }
}

async function register() {
  if (!regEmail.value.trim()) {
    regError.value = t('common.enter_email_required')
    return
  }
  if (!regPassword.value) {
    regError.value = t('aff.enter_password_required')
    return
  }
  if (regPassword.value !== regConfirm.value) {
    regError.value = t('aff.password_mismatch')
    return
  }
  if (regPassword.value.length < 6) {
    regError.value = t('aff.password_min_length')
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
    // Auto-query by the new AFF code after registration
    queryCode.value = res.data.aff_code
    notRegistered.value = false
    await queryBalance()
  } catch (e: any) {
    regError.value = e.response?.data?.message || e.response?.data?.error || t('aff.register_failed')
  } finally {
    regLoading.value = false
  }
}

async function submitWithdraw() {
  if (!affInfo.value) return
  if (!wdAmount.value || wdAmount.value <= 0) {
    wdError.value = t('aff.enter_valid_amount')
    return
  }
  if (wdAmount.value > affInfo.value.balance) {
    wdError.value = t('aff.withdraw_exceed_balance')
    return
  }
  if (!wdAddress.value.trim()) {
    wdError.value = t('aff.enter_wallet_required')
    return
  }
  if (!wdPassword.value) {
    wdError.value = t('aff.enter_withdraw_password')
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
    wdSuccess.value = t('aff.withdraw_success')
    wdAmount.value = undefined
    wdAddress.value = ''
    wdPassword.value = ''
    // Refresh balance
    await queryBalance()
  } catch (e: any) {
    wdError.value = e.response?.data?.message || e.response?.data?.error || t('aff.withdraw_failed')
  } finally {
    wdLoading.value = false
  }
}
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16 space-y-6">
    <!-- Section 1: Balance Query -->
    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{{ $t('aff.title') }}</h2>
      <div class="flex gap-3">
        <input
          v-model="queryCode"
          type="text"
          :placeholder="$t('aff.code_placeholder')"
          @keyup.enter="queryBalance"
          class="flex-1 px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
        />
        <button
          @click="queryBalance"
          :disabled="queryLoading"
          class="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center gap-2"
        >
          <div v-if="queryLoading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
          {{ $t('order.query_button') }}
        </button>
      </div>
      <p v-if="queryError" class="text-red-500 text-sm mt-3">{{ queryError }}</p>

      <!-- Balance info -->
      <div v-if="affInfo" class="mt-5">
        <!-- Tier badge -->
        <div class="flex items-center gap-3 mb-4">
          <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium"
            :class="affInfo.level <= 1 ? 'bg-gray-100 text-gray-700' : affInfo.level === 2 ? 'bg-blue-100 text-blue-700' : affInfo.level === 3 ? 'bg-yellow-100 text-yellow-700' : 'bg-purple-100 text-purple-700'">
            {{ affInfo.level_name }} · {{ $t('aff.commission_rate') }} {{ (affInfo.commission_rate * 100).toFixed(1) }}%
          </span>
          <span v-if="affInfo.next_level" class="text-xs text-gray-400 dark:text-gray-500">
            {{ $t('aff.upgrade_remaining', { level: affInfo.next_level.name, amount: affInfo.next_level.remaining.toFixed(2) }) }}
          </span>
        </div>

        <!-- Progress bar to next level -->
        <div v-if="affInfo.next_level" class="mb-4">
          <div class="flex justify-between text-xs text-gray-400 dark:text-gray-500 mb-1">
            <span>{{ affInfo.level_name }}</span>
            <span>{{ affInfo.next_level.name }} ({{ (affInfo.next_level.commission_rate * 100).toFixed(0) }}%)</span>
          </div>
          <div class="w-full bg-gray-100 dark:bg-gray-700 rounded-full h-2">
            <div class="bg-blue-500 h-2 rounded-full transition-all" :style="{ width: Math.min(100, (affInfo.total_earned / affInfo.next_level.required_amount) * 100).toFixed(1) + '%' }"></div>
          </div>
          <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">{{ $t('aff.total_referral_amount', { earned: affInfo.total_earned.toFixed(2), required: affInfo.next_level.required_amount.toFixed(2) }) }}</div>
        </div>

        <div class="grid grid-cols-3 gap-4 mb-4">
          <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 dark:text-gray-500 mb-1">{{ $t('aff.balance') }}</div>
            <div class="text-xl font-bold text-gray-900 dark:text-white">¥{{ affInfo.balance.toFixed(2) }}</div>
          </div>
          <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 dark:text-gray-500 mb-1">{{ $t('aff.total_earned') }}</div>
            <div class="text-xl font-bold text-gray-900 dark:text-white">¥{{ affInfo.total_earned.toFixed(2) }}</div>
          </div>
          <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 text-center">
            <div class="text-xs text-gray-400 dark:text-gray-500 mb-1">{{ $t('aff.total_withdrawn') }}</div>
            <div class="text-xl font-bold text-gray-900 dark:text-white">¥{{ affInfo.total_withdrawn.toFixed(2) }}</div>
          </div>
        </div>

        <!-- Tier levels overview -->
        <div v-if="allTiers.length > 1" class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 mb-4">
          <div class="text-xs text-gray-400 dark:text-gray-500 mb-2">{{ $t('aff.tier_overview') }}</div>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-2">
            <div v-for="tier in allTiers" :key="tier.level"
              class="text-center p-2 rounded-lg border text-xs"
              :class="affInfo.level === tier.level ? 'bg-blue-50 dark:bg-blue-900/30 border-blue-200 dark:border-blue-800 text-blue-700 dark:text-blue-400' : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-600 text-gray-500 dark:text-gray-400'">
              <div class="font-medium">{{ tier.name }}</div>
              <div class="text-green-600 font-semibold">{{ (tier.commission_rate * 100).toFixed(0) }}%</div>
              <div v-if="tier.required_amount > 0" class="text-gray-400 dark:text-gray-500">≥¥{{ tier.required_amount }}</div>
            </div>
          </div>
        </div>

        <!-- Aff link -->
        <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4">
          <div class="text-xs text-gray-400 dark:text-gray-500 mb-1.5">{{ $t('aff.your_link') }}</div>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-xs text-gray-600 dark:text-gray-300 bg-white dark:bg-gray-800 px-3 py-2 rounded-lg border border-gray-200 dark:border-gray-600 truncate">{{ affLink }}</code>
            <button
              @click="copyLink"
              class="px-3 py-2 text-xs font-medium rounded-lg border transition-all"
              :class="copied ? 'bg-green-50 dark:bg-green-900/30 text-green-600 dark:text-green-400 border-green-200 dark:border-green-800' : 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-300 border-gray-200 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700'"
            >{{ copied ? $t('common.copied') : $t('common.copy') }}</button>
          </div>
          <div class="text-xs text-gray-400 dark:text-gray-500 mt-2">{{ $t('aff.your_code') }}: <span class="font-mono font-medium text-gray-600 dark:text-gray-300">{{ affInfo.aff_code }}</span></div>
        </div>
      </div>
    </div>

    <!-- Section 2: Register (show if not registered) -->
    <div v-if="notRegistered" class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">{{ $t('aff.register_title') }}</h2>
      <p class="text-sm text-gray-400 dark:text-gray-500 mb-4">{{ $t('aff.not_registered_hint') }}</p>

      <div class="space-y-3">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('common.email') }}</label>
          <input
            v-model="regEmail"
            type="email"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none"
            readonly
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.withdraw_password') }}</label>
          <input
            v-model="regPassword"
            type="password"
            :placeholder="$t('aff.password_min_hint')"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('admin.confirm_password') }}</label>
          <input
            v-model="regConfirm"
            type="password"
            :placeholder="$t('aff.confirm_password_hint')"
            @keyup.enter="register"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
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
        {{ regLoading ? $t('aff.registering') : $t('aff.register') }}
      </button>
    </div>

    <!-- Section 3: Withdraw -->
    <div v-if="showWithdraw" class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{{ $t('aff.withdraw_title') }}</h2>

      <div class="space-y-4">
        <!-- Amount -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.withdraw_amount') }}</label>
          <input
            v-model.number="wdAmount"
            type="number"
            step="0.01"
            :placeholder="$t('aff.max_amount', { amount: affInfo?.balance.toFixed(2) })"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>

        <!-- Currency -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-2">{{ $t('aff.withdraw_currency') }}</label>
          <div class="flex gap-2">
            <label
              v-for="c in ['USDT', 'USDC']"
              :key="c"
              :class="[
                'px-4 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                wdCurrency === c
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
                  : 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500'
              ]"
            >
              <input type="radio" v-model="wdCurrency" :value="c" class="sr-only" />
              {{ c }}
            </label>
          </div>
        </div>

        <!-- Chain -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-2">{{ $t('aff.withdraw_chain') }}</label>
          <div class="flex gap-2">
            <label
              v-for="c in ['Tron', 'Polygon', 'Base']"
              :key="c"
              :class="[
                'px-4 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                wdChain === c
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
                  : 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500'
              ]"
            >
              <input type="radio" v-model="wdChain" :value="c" class="sr-only" />
              {{ c }}
            </label>
          </div>
        </div>

        <!-- Wallet address -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.withdraw_address') }}</label>
          <input
            v-model="wdAddress"
            type="text"
            :placeholder="$t('aff.wallet_placeholder')"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm font-mono bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
          />
        </div>

        <!-- Password -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.withdraw_password') }}</label>
          <input
            v-model="wdPassword"
            type="password"
            :placeholder="$t('aff.enter_withdraw_password')"
            class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
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
        {{ wdLoading ? $t('common.submitting') : $t('aff.withdraw_submit') }}
      </button>
    </div>

    <!-- Aff logs -->
    <div v-if="affInfo && affLogs.length > 0" class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{{ $t('aff.commission_logs') }}</h2>
      <div class="space-y-2">
        <div
          v-for="log in affLogs"
          :key="log.id"
          class="flex items-center justify-between py-3 border-b border-gray-50 dark:border-gray-700 last:border-0"
        >
          <div>
            <div class="text-sm text-gray-700 dark:text-gray-200">{{ $t('order.order_no') }} #{{ log.order_id }}</div>
            <div class="text-xs text-gray-400 dark:text-gray-500">{{ formatTime(log.created_at) }}</div>
          </div>
          <div class="text-right">
            <div class="text-sm font-semibold text-green-600">+¥{{ log.commission.toFixed(2) }}</div>
            <div class="text-xs text-gray-400 dark:text-gray-500">{{ log.status === 'credited' ? $t('aff.credited') : log.status }}</div>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>
