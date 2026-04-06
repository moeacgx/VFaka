<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const saving = ref(false)
const settings = ref<Record<string, string>>({})

const settingLabels: Record<string, string> = {
  site_name: '站点名称',
  site_description: '站点描述',
  site_keywords: '站点关键词',
  site_logo: '站点 Logo URL',
  contact_email: '联系邮箱',
  contact_telegram: 'Telegram',
  aff_commission_rate: '推广佣金率 (%)',
  aff_min_withdraw: '最低提现金额',
  aff_withdraw_fee_rate: '提现手续费率 (%)',
}

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

function addSetting() {
  const key = prompt('请输入设置键名')
  if (key && !(key in settings.value)) {
    settings.value[key] = ''
  }
}

async function save() {
  saving.value = true
  try {
    await adminApi.updateSettings(settings.value)
    alert('设置已保存')
  } catch (e: any) {
    alert(e.response?.data?.error || '保存失败')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 text-sm">加载中...</div>
    <div v-else class="bg-white rounded-lg border border-gray-200 p-6">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-base font-medium text-gray-800">系统设置</h3>
        <button @click="addSetting" class="px-3 py-1.5 text-sm text-blue-600 hover:text-blue-800">+ 添加设置项</button>
      </div>
      <form @submit.prevent="save" class="space-y-4">
        <div v-for="(_value, key) in settings" :key="key" class="grid grid-cols-1 sm:grid-cols-3 gap-2 items-start">
          <label class="text-sm font-medium text-gray-700 sm:pt-2">{{ settingLabels[key as string] || key }}</label>
          <div class="sm:col-span-2">
            <input
              v-model="settings[key as string]"
              class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
            <div v-if="!settingLabels[key as string]" class="text-xs text-gray-400 mt-0.5">{{ key }}</div>
          </div>
        </div>
        <div v-if="Object.keys(settings).length === 0" class="text-gray-400 text-sm py-4 text-center">暂无设置项</div>
        <div class="pt-2">
          <button type="submit" :disabled="saving" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
            {{ saving ? '保存中...' : '保存设置' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
