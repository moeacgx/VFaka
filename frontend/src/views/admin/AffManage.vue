<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const savingSettings = ref(false)
const affUsers = ref<any[]>([])

const settings = ref({
  commission_rate: 0,
  min_withdraw_amount: 0,
  withdraw_fee_rate: 0,
})

async function load() {
  loading.value = true
  try {
    const [settingsRes, usersRes] = await Promise.all([
      adminApi.getSettings(),
      adminApi.getAffUsers(),
    ])
    const s = settingsRes.data || {}
    settings.value.commission_rate = parseFloat(s.aff_commission_rate || s.commission_rate || '0')
    settings.value.min_withdraw_amount = parseFloat(s.aff_min_withdraw || s.min_withdraw_amount || '0')
    settings.value.withdraw_fee_rate = parseFloat(s.aff_withdraw_fee_rate || s.withdraw_fee_rate || '0')
    affUsers.value = usersRes.data || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function saveSettings() {
  savingSettings.value = true
  try {
    await adminApi.updateAffSettings({
      commission_rate: settings.value.commission_rate,
      min_withdraw_amount: settings.value.min_withdraw_amount,
      withdraw_fee_rate: settings.value.withdraw_fee_rate,
    })
    alert('推广设置已保存')
  } catch (e: any) {
    alert(e.response?.data?.error || '保存失败')
  } finally {
    savingSettings.value = false
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 text-sm">加载中...</div>
    <template v-else>
      <!-- Global settings -->
      <div class="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">推广全局设置</h3>
        <form @submit.prevent="saveSettings" class="space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">佣金比例 (%)</label>
              <input v-model.number="settings.commission_rate" type="number" step="0.1" min="0" max="100" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">最低提现金额</label>
              <input v-model.number="settings.min_withdraw_amount" type="number" step="0.01" min="0" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">提现手续费率 (%)</label>
              <input v-model.number="settings.withdraw_fee_rate" type="number" step="0.1" min="0" max="100" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
          </div>
          <button type="submit" :disabled="savingSettings" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
            {{ savingSettings ? '保存中...' : '保存设置' }}
          </button>
        </form>
      </div>

      <!-- AFF users table -->
      <div class="bg-white rounded-lg border border-gray-200">
        <div class="px-5 py-4 border-b border-gray-200">
          <h3 class="text-sm font-medium text-gray-800">推广用户列表</h3>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 text-left text-gray-500">
                <th class="px-4 py-3 font-medium">邮箱</th>
                <th class="px-4 py-3 font-medium">推广码</th>
                <th class="px-4 py-3 font-medium">余额</th>
                <th class="px-4 py-3 font-medium">总收入</th>
                <th class="px-4 py-3 font-medium">已提现</th>
                <th class="px-4 py-3 font-medium">注册时间</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100">
              <tr v-for="user in affUsers" :key="user.id" class="hover:bg-gray-50">
                <td class="px-4 py-3 text-gray-800">{{ user.email }}</td>
                <td class="px-4 py-3 text-gray-600 font-mono text-xs">{{ user.aff_code }}</td>
                <td class="px-4 py-3 text-gray-800">¥{{ (user.balance ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600">¥{{ (user.total_earned ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600">¥{{ (user.total_withdrawn ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-500 text-xs">{{ user.created_at?.replace('T', ' ').slice(0, 19) }}</td>
              </tr>
              <tr v-if="affUsers.length === 0">
                <td colspan="6" class="px-4 py-8 text-center text-gray-400">暂无推广用户</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>
</template>
