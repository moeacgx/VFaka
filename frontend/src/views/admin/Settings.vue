<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const settings = ref<Record<string, string>>({})
const expandedSections = ref<Record<string, boolean>>({
  basic: true,
  announcement: false,
  telegram: false,
  smtp: false,
  storage: false,
})
const saving = ref<string | null>(null)
const testingTelegram = ref(false)
const testingEmail = ref(false)
const testEmail = ref('')

onMounted(async () => {
  try {
    const res = await adminApi.getSettings()
    settings.value = res.data || {}
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
})

function toggleSection(section: string) {
  expandedSections.value[section] = !expandedSections.value[section]
}

async function saveSection(section: string) {
  saving.value = section
  try {
    await adminApi.updateSettings(settings.value)
    alert(t('common.operation_success'))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  } finally {
    saving.value = null
  }
}

async function testTelegram() {
  testingTelegram.value = true
  try {
    const token = localStorage.getItem('admin_token')
    const res = await fetch('/api/admin/settings/test-telegram', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
    })
    const data = await res.json()
    if (res.ok) {
      alert(t('common.operation_success'))
    } else {
      alert(data.error || t('common.operation_failed'))
    }
  } catch (e: any) {
    alert(t('common.operation_failed'))
  } finally {
    testingTelegram.value = false
  }
}

async function testSmtp() {
  if (!testEmail.value) return
  testingEmail.value = true
  try {
    const token = localStorage.getItem('admin_token')
    const res = await fetch('/api/admin/settings/test-email', {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ test_email: testEmail.value }),
    })
    const data = await res.json()
    if (res.ok) {
      alert(t('common.operation_success'))
    } else {
      alert(data.error || t('common.operation_failed'))
    }
  } catch (e: any) {
    alert(t('common.operation_failed'))
  } finally {
    testingEmail.value = false
  }
}
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
    <div v-else class="space-y-4">

      <!-- Section 1: Basic Settings -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <button @click="toggleSection('basic')" class="w-full flex items-center justify-between px-6 py-4 text-left">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('settings.basic') }}</h3>
          <svg :class="{ 'rotate-180': expandedSections.basic }" class="w-5 h-5 text-gray-400 dark:text-gray-500 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
        </button>
        <div v-show="expandedSections.basic" class="px-6 pb-6 space-y-4 border-t border-gray-100 dark:border-gray-700 pt-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.site_name') }}</label>
              <input v-model="settings.site_name" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.site_keywords') }}</label>
              <input v-model="settings.site_keywords" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.site_description') }}</label>
            <input v-model="settings.site_description" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.site_logo') }}</label>
            <input v-model="settings.site_logo" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.contact_email') }}</label>
              <input v-model="settings.contact_email" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.contact_telegram') }}</label>
              <input v-model="settings.contact_telegram" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.aff_commission_rate') }}</label>
              <input v-model="settings.aff_commission_rate" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.aff_min_withdraw') }}</label>
              <input v-model="settings.aff_min_withdraw" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.aff_withdraw_fee_rate') }}</label>
              <input v-model="settings.aff_withdraw_fee_rate" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="pt-2">
            <button @click="saveSection('basic')" :disabled="saving === 'basic'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ saving === 'basic' ? $t('common.saving') : $t('settings.save_settings') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Section 2: Announcement -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <button @click="toggleSection('announcement')" class="w-full flex items-center justify-between px-6 py-4 text-left">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('settings.announcement') }}</h3>
          <svg :class="{ 'rotate-180': expandedSections.announcement }" class="w-5 h-5 text-gray-400 dark:text-gray-500 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
        </button>
        <div v-show="expandedSections.announcement" class="px-6 pb-6 space-y-4 border-t border-gray-100 dark:border-gray-700 pt-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200">
            <input type="checkbox" :checked="settings.announcement_enabled === 'true'" @change="settings.announcement_enabled = ($event.target as HTMLInputElement).checked ? 'true' : 'false'" />
            {{ $t('settings.announcement_enabled') }}
          </label>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.announcement_text') }}</label>
            <textarea v-model="settings.announcement_text" rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.announcement_type') }}</label>
            <select v-model="settings.announcement_type" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white">
              <option value="info">{{ $t('common.info') }}</option>
              <option value="warning">{{ $t('common.warning') }}</option>
              <option value="success">{{ $t('common.success') }}</option>
            </select>
          </div>
          <div class="pt-2">
            <button @click="saveSection('announcement')" :disabled="saving === 'announcement'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ saving === 'announcement' ? $t('common.saving') : $t('settings.save_settings') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Section 3: Telegram Notification -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <button @click="toggleSection('telegram')" class="w-full flex items-center justify-between px-6 py-4 text-left">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('settings.telegram') }}</h3>
          <svg :class="{ 'rotate-180': expandedSections.telegram }" class="w-5 h-5 text-gray-400 dark:text-gray-500 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
        </button>
        <div v-show="expandedSections.telegram" class="px-6 pb-6 space-y-4 border-t border-gray-100 dark:border-gray-700 pt-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200">
            <input type="checkbox" :checked="settings.telegram_enabled === 'true'" @change="settings.telegram_enabled = ($event.target as HTMLInputElement).checked ? 'true' : 'false'" />
            {{ $t('settings.telegram_enabled') }}
          </label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.telegram_bot_token') }}</label>
              <input v-model="settings.telegram_bot_token" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.telegram_chat_id') }}</label>
              <input v-model="settings.telegram_chat_id" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="flex items-center gap-3 pt-2">
            <button @click="saveSection('telegram')" :disabled="saving === 'telegram'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ saving === 'telegram' ? $t('common.saving') : $t('settings.save_settings') }}
            </button>
            <button @click="testTelegram" :disabled="testingTelegram" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 disabled:opacity-50">
              {{ testingTelegram ? $t('common.loading') : $t('settings.telegram_test') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Section 4: SMTP Email -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <button @click="toggleSection('smtp')" class="w-full flex items-center justify-between px-6 py-4 text-left">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('settings.smtp') }}</h3>
          <svg :class="{ 'rotate-180': expandedSections.smtp }" class="w-5 h-5 text-gray-400 dark:text-gray-500 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
        </button>
        <div v-show="expandedSections.smtp" class="px-6 pb-6 space-y-4 border-t border-gray-100 dark:border-gray-700 pt-4">
          <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200">
            <input type="checkbox" :checked="settings.smtp_enabled === 'true'" @change="settings.smtp_enabled = ($event.target as HTMLInputElement).checked ? 'true' : 'false'" />
            {{ $t('settings.smtp_enabled') }}
          </label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.smtp_host') }}</label>
              <input v-model="settings.smtp_host" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.smtp_port') }}</label>
              <input v-model="settings.smtp_port" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.smtp_username') }}</label>
              <input v-model="settings.smtp_username" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.smtp_password') }}</label>
              <input v-model="settings.smtp_password" type="password" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.smtp_from') }}</label>
            <input v-model="settings.smtp_from" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div class="flex flex-col sm:flex-row items-start sm:items-end gap-3 pt-2">
            <button @click="saveSection('smtp')" :disabled="saving === 'smtp'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ saving === 'smtp' ? $t('common.saving') : $t('settings.save_settings') }}
            </button>
            <div class="flex items-center gap-2">
              <input v-model="testEmail" :placeholder="$t('settings.test_email_address')" class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
              <button @click="testSmtp" :disabled="testingEmail || !testEmail" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 disabled:opacity-50 whitespace-nowrap">
                {{ testingEmail ? $t('common.loading') : $t('settings.smtp_test') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Section 5: File Storage -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <button @click="toggleSection('storage')" class="w-full flex items-center justify-between px-6 py-4 text-left">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('settings.storage') }}</h3>
          <svg :class="{ 'rotate-180': expandedSections.storage }" class="w-5 h-5 text-gray-400 dark:text-gray-500 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
        </button>
        <div v-show="expandedSections.storage" class="px-6 pb-6 space-y-4 border-t border-gray-100 dark:border-gray-700 pt-4">
          <div class="flex items-center gap-6">
            <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200 cursor-pointer">
              <input v-model="settings.storage_type" type="radio" value="local" name="storage_type" />
              {{ $t('settings.storage_local') }}
            </label>
            <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200 cursor-pointer">
              <input v-model="settings.storage_type" type="radio" value="s3" name="storage_type" />
              {{ $t('settings.storage_s3') }}
            </label>
          </div>
          <template v-if="settings.storage_type === 's3'">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.s3_endpoint') }}</label>
                <input v-model="settings.s3_endpoint" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.s3_bucket') }}</label>
                <input v-model="settings.s3_bucket" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
              </div>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.s3_access_key') }}</label>
                <input v-model="settings.s3_access_key" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.s3_secret_key') }}</label>
                <input v-model="settings.s3_secret_key" type="password" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
              </div>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('settings.s3_region') }}</label>
              <input v-model="settings.s3_region" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </template>
          <div class="pt-2">
            <button @click="saveSection('storage')" :disabled="saving === 'storage'" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ saving === 'storage' ? $t('common.saving') : $t('settings.save_settings') }}
            </button>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
