<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const savingSettings = ref(false)
const affUsers = ref<any[]>([])
const tiers = ref<any[]>([])
const showTierModal = ref(false)
const editingTier = ref<any>(null)
const tierForm = ref({ level: 0, name: '', commission_rate: 0, required_amount: 0 })
const savingTier = ref(false)

const settings = ref({
  min_withdraw_amount: 0,
  withdraw_fee_rate: 0,
})

async function load() {
  loading.value = true
  try {
    const [settingsRes, usersRes, tiersRes] = await Promise.all([
      adminApi.getSettings(),
      adminApi.getAffUsers(),
      adminApi.getAffTiers(),
    ])
    const s = settingsRes.data || {}
    settings.value.min_withdraw_amount = parseFloat(s.min_withdraw_amount || '0')
    settings.value.withdraw_fee_rate = parseFloat(s.withdraw_fee_rate || '0')
    affUsers.value = usersRes.data || []
    tiers.value = tiersRes.data || []
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
      min_withdraw_amount: String(settings.value.min_withdraw_amount),
      withdraw_fee_rate: String(settings.value.withdraw_fee_rate),
    })
    alert(t('common.operation_success'))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  } finally {
    savingSettings.value = false
  }
}

function openAddTier() {
  editingTier.value = null
  const maxLevel = tiers.value.length > 0 ? Math.max(...tiers.value.map((t: any) => t.level)) : 0
  tierForm.value = { level: maxLevel + 1, name: '', commission_rate: 0, required_amount: 0 }
  showTierModal.value = true
}

function openEditTier(tier: any) {
  editingTier.value = tier
  tierForm.value = { level: tier.level, name: tier.name, commission_rate: tier.commission_rate, required_amount: tier.required_amount }
  showTierModal.value = true
}

async function saveTier() {
  savingTier.value = true
  try {
    if (editingTier.value) {
      await adminApi.updateAffTier(editingTier.value.level, {
        name: tierForm.value.name,
        commission_rate: tierForm.value.commission_rate,
        required_amount: tierForm.value.required_amount,
      })
    } else {
      await adminApi.createAffTier(tierForm.value)
    }
    showTierModal.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  } finally {
    savingTier.value = false
  }
}

async function deleteTier(level: number) {
  if (!confirm(t('common.confirm_delete'))) return
  try {
    await adminApi.deleteAffTier(level)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

function tierName(level: number) {
  const tier = tiers.value.find((t: any) => t.level === level)
  return tier ? tier.name : `Level ${level}`
}

async function deleteUser(user: any) {
  if (!confirm(`${t('common.confirm_delete')} ${user.email} (${user.aff_code})?`)) return
  try {
    await adminApi.deleteAffUser(user.id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div v-if="loading" class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
    <template v-else>
      <!-- Tier management -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('aff.manage_tiers') }}</h3>
          <button @click="openAddTier" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">+ {{ $t('aff.add_tier') }}</button>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 dark:bg-gray-900 text-left text-gray-500 dark:text-gray-400">
                <th class="px-4 py-3 font-medium">{{ $t('aff.tier_level') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.tier_name') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.tier_rate') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.required_amount') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
              <tr v-for="tier in tiers" :key="tier.level" class="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td class="px-4 py-3">
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium" :class="tier.level <= 1 ? 'bg-gray-100 text-gray-700' : tier.level === 2 ? 'bg-blue-100 text-blue-700' : tier.level === 3 ? 'bg-yellow-100 text-yellow-700' : 'bg-purple-100 text-purple-700'">
                    Level {{ tier.level }}
                  </span>
                </td>
                <td class="px-4 py-3 text-gray-800 dark:text-gray-100 font-medium">{{ tier.name }}</td>
                <td class="px-4 py-3 text-green-600 font-medium">{{ (tier.commission_rate * 100).toFixed(1) }}%</td>
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ tier.required_amount > 0 ? `¥${tier.required_amount.toFixed(2)}` : '—' }}</td>
                <td class="px-4 py-3 space-x-2">
                  <button @click="openEditTier(tier)" class="text-blue-600 hover:text-blue-800 text-xs">{{ $t('common.edit') }}</button>
                  <button v-if="tier.level > 1" @click="deleteTier(tier.level)" class="text-red-500 hover:text-red-700 text-xs">{{ $t('common.delete') }}</button>
                </td>
              </tr>
              <tr v-if="tiers.length === 0">
                <td colspan="5" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Global settings -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('aff.withdraw_settings') }}</h3>
        <form @submit.prevent="saveSettings" class="space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.min_withdraw_amount') }}</label>
              <input v-model.number="settings.min_withdraw_amount" type="number" step="0.01" min="0" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.withdraw_fee_rate') }}</label>
              <input v-model.number="settings.withdraw_fee_rate" type="number" step="0.1" min="0" max="100" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <button type="submit" :disabled="savingSettings" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
            {{ savingSettings ? $t('common.saving') : $t('settings.save_settings') }}
          </button>
        </form>
      </div>

      <!-- AFF users table -->
      <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
        <div class="px-5 py-4 border-b border-gray-200 dark:border-gray-700">
          <h3 class="text-sm font-medium text-gray-800 dark:text-gray-100">{{ $t('aff.manage_users') }}</h3>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="bg-gray-50 dark:bg-gray-900 text-left text-gray-500 dark:text-gray-400">
                <th class="px-4 py-3 font-medium">{{ $t('common.email') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.your_code') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.tier_level') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.balance') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.total_earned') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('aff.total_withdrawn') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.created_at') }}</th>
                <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
              <tr v-for="user in affUsers" :key="user.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ user.email }}</td>
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300 font-mono text-xs">{{ user.aff_code }}</td>
                <td class="px-4 py-3">
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium" :class="user.level <= 1 ? 'bg-gray-100 text-gray-700' : user.level === 2 ? 'bg-blue-100 text-blue-700' : user.level === 3 ? 'bg-yellow-100 text-yellow-700' : 'bg-purple-100 text-purple-700'">
                    {{ tierName(user.level) }}
                  </span>
                </td>
                <td class="px-4 py-3 text-gray-800 dark:text-gray-100">¥{{ (user.balance ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300">¥{{ (user.total_earned ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-600 dark:text-gray-300">¥{{ (user.total_withdrawn ?? 0).toFixed(2) }}</td>
                <td class="px-4 py-3 text-gray-500 dark:text-gray-400 text-xs">{{ user.created_at?.replace('T', ' ').slice(0, 19) }}</td>
                <td class="px-4 py-3">
                  <button @click="deleteUser(user)" class="text-red-500 hover:text-red-700 text-xs">{{ $t('common.delete') }}</button>
                </td>
              </tr>
              <tr v-if="affUsers.length === 0">
                <td colspan="8" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>

    <!-- Tier modal -->
    <div v-if="showTierModal" class="fixed inset-0 bg-black/40 flex items-center justify-center z-50" @click.self="showTierModal = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ editingTier ? $t('aff.edit_tier') : $t('aff.add_tier') }}</h3>
        <form @submit.prevent="saveTier" class="space-y-4">
          <div v-if="!editingTier">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.tier_level') }}</label>
            <input v-model.number="tierForm.level" type="number" min="1" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.tier_name') }}</label>
            <input v-model="tierForm.name" type="text" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.tier_rate') }} (0~1)</label>
            <input v-model.number="tierForm.commission_rate" type="number" step="0.01" min="0" max="1" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('aff.required_amount') }} (¥)</label>
            <input v-model.number="tierForm.required_amount" type="number" step="0.01" min="0" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div class="flex justify-end gap-3 pt-2">
            <button type="button" @click="showTierModal = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" :disabled="savingTier" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">
              {{ savingTier ? $t('common.saving') : $t('common.save') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
