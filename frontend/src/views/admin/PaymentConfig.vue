<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const saving = ref<string | null>(null)

const epay = ref({
  is_active: false,
  pid: '',
  merchant_private_key: '',
  platform_public_key: '',
  api_url: '',
})

const tokenpay = ref({
  is_active: false,
  api_url: 'http://tokenpay:5000',
  notify_secret: '',
  tron_address: '',
  evm_address: '',
})
const showAdvanced = ref(false)

onMounted(async () => {
  try {
    const res = await adminApi.getPaymentConfigs()
    const configs = res.data || []
    for (const cfg of configs) {
      if (cfg.channel === 'epay') {
        epay.value.is_active = !!cfg.is_active
        const c = typeof cfg.config === 'string' ? JSON.parse(cfg.config) : (cfg.config || {})
        epay.value.pid = c.pid || ''
        epay.value.merchant_private_key = c.merchant_private_key || ''
        epay.value.platform_public_key = c.platform_public_key || ''
        epay.value.api_url = c.api_url || ''
      } else if (cfg.channel === 'tokenpay') {
        tokenpay.value.is_active = !!cfg.is_active
        const c = typeof cfg.config === 'string' ? JSON.parse(cfg.config) : (cfg.config || {})
        tokenpay.value.api_url = c.api_url || 'http://tokenpay:5000'
        tokenpay.value.notify_secret = c.notify_secret || ''
        tokenpay.value.tron_address = c.tron_address || ''
        tokenpay.value.evm_address = c.evm_address || ''
      }
    }
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
})

async function saveEpay() {
  saving.value = 'epay'
  try {
    await adminApi.updatePaymentConfig('epay', {
      is_active: epay.value.is_active,
      config: {
        pid: epay.value.pid,
        merchant_private_key: epay.value.merchant_private_key,
        platform_public_key: epay.value.platform_public_key,
        api_url: epay.value.api_url,
      },
    })
    alert(t('common.operation_success'))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  } finally {
    saving.value = null
  }
}

async function saveTokenpay() {
  saving.value = 'tokenpay'
  try {
    await adminApi.updatePaymentConfig('tokenpay', {
      is_active: tokenpay.value.is_active,
      config: {
        api_url: tokenpay.value.api_url,
        notify_secret: tokenpay.value.notify_secret,
        tron_address: tokenpay.value.tron_address,
        evm_address: tokenpay.value.evm_address,
      },
    })
    alert(t('common.operation_success'))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  } finally {
    saving.value = null
  }
}
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
    <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Epay -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('payment.epay_title') }}</h3>
          <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200">
            <input v-model="epay.is_active" type="checkbox" />
            {{ $t('common.enabled') }}
          </label>
        </div>
        <form @submit.prevent="saveEpay" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.pid') }}</label>
            <input v-model="epay.pid" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.api_url') }}</label>
            <input v-model="epay.api_url" placeholder="https://pay.example.com" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.merchant_key') }}</label>
            <textarea v-model="epay.merchant_private_key" rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.platform_key') }}</label>
            <textarea v-model="epay.platform_public_key" rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <button type="submit" :disabled="saving === 'epay'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
            {{ saving === 'epay' ? $t('common.saving') : $t('payment.save_config') }}
          </button>
        </form>
      </div>

      <!-- TokenPay -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('payment.tokenpay_title') }}</h3>
          <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200">
            <input v-model="tokenpay.is_active" type="checkbox" />
            {{ $t('common.enabled') }}
          </label>
        </div>
        <form @submit.prevent="saveTokenpay" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.tron_address') }}</label>
            <input v-model="tokenpay.tron_address" placeholder="T..." class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">USDT/USDC TRC20</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.evm_address') }}</label>
            <input v-model="tokenpay.evm_address" placeholder="0x..." class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            <p class="text-xs text-gray-400 dark:text-gray-500 mt-1">USDT/USDC ERC20, Polygon, Base</p>
          </div>

          <!-- Advanced toggle -->
          <div>
            <button type="button" @click="showAdvanced = !showAdvanced" class="text-xs text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors">
              {{ showAdvanced ? $t('payment.hide_advanced') : $t('payment.show_advanced') }}
            </button>
          </div>
          <div v-if="showAdvanced" class="space-y-4 pt-2 border-t border-gray-100 dark:border-gray-700">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.api_url') }}</label>
              <input v-model="tokenpay.api_url" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('payment.notify_secret') }}</label>
              <input v-model="tokenpay.notify_secret" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>

          <button type="submit" :disabled="saving === 'tokenpay'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
            {{ saving === 'tokenpay' ? $t('common.saving') : $t('payment.save_config') }}
          </button>
        </form>
      </div>
    </div>
  </div>
</template>
