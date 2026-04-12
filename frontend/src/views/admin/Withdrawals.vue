<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'
import { useConfirm } from '../../composables/useConfirm'

const { t } = useI18n()
const { confirm } = useConfirm()
const loading = ref(true)
const withdrawals = ref<any[]>([])
const filterStatus = ref('')
const showComplete = ref(false)
const showReject = ref(false)
const selectedId = ref<number | null>(null)
const txHash = ref('')
const rejectNote = ref('')

const statusMap = computed<Record<string, string>>(() => ({
  pending: t('withdrawal.status.pending'),
  approved: t('withdrawal.status.approved'),
  rejected: t('withdrawal.status.rejected'),
  completed: t('withdrawal.status.completed'),
}))

async function load() {
  loading.value = true
  try {
    const params: any = {}
    if (filterStatus.value) params.status = filterStatus.value
    const res = await adminApi.getWithdrawals(params)
    withdrawals.value = res.data || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function approve(id: number) {
  if (!await confirm(t('common.confirm') + '?')) return
  try {
    await adminApi.approveWithdrawal(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

function openReject(id: number) {
  selectedId.value = id
  rejectNote.value = ''
  showReject.value = true
}

async function submitReject() {
  if (selectedId.value === null) return
  try {
    await adminApi.rejectWithdrawal(selectedId.value, { note: rejectNote.value })
    showReject.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

function openComplete(id: number) {
  selectedId.value = id
  txHash.value = ''
  showComplete.value = true
}

async function submitComplete() {
  if (selectedId.value === null || !txHash.value.trim()) {
    alert(t('withdrawal.enter_tx_hash'))
    return
  }
  try {
    await adminApi.completeWithdrawal(selectedId.value, { tx_hash: txHash.value })
    showComplete.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('withdrawal.title') }}</h3>
      <select v-model="filterStatus" @change="load" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
        <option value="">{{ $t('common.all') }} {{ $t('common.status') }}</option>
        <option value="pending">{{ $t('withdrawal.status.pending') }}</option>
        <option value="approved">{{ $t('withdrawal.status.approved') }}</option>
        <option value="rejected">{{ $t('withdrawal.status.rejected') }}</option>
        <option value="completed">{{ $t('withdrawal.status.completed') }}</option>
      </select>
    </div>

    <!-- Complete modal -->
    <div v-if="showComplete" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showComplete = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('withdrawal.complete') }}</h3>
        <form @submit.prevent="submitComplete" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('withdrawal.tx_hash') }}</label>
            <input v-model="txHash" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" placeholder="0x..." />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showComplete = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700">{{ $t('common.confirm') }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Reject modal -->
    <div v-if="showReject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showReject = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('withdrawal.reject') }}</h3>
        <form @submit.prevent="submitReject" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('withdrawal.reject_reason') }}</label>
            <textarea v-model="rejectNote" rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" :placeholder="$t('withdrawal.reject_reason')" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showReject = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700">{{ $t('common.confirm') }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Table -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-x-auto">
      <div v-if="loading" class="p-8 text-center text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.loading') }}</div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="bg-gray-50 dark:bg-gray-900 text-left text-gray-500 dark:text-gray-400">
            <th class="px-4 py-3 font-medium">ID</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.email') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('withdrawal.amount') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('withdrawal.currency') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('withdrawal.chain') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('withdrawal.wallet') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('order.time') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="w in withdrawals" :key="w.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ w.id }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ w.email }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ w.amount?.toFixed(2) }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ w.currency || 'USDT' }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ w.chain || '-' }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300 font-mono text-xs max-w-[200px] truncate" :title="w.wallet_address">{{ w.wallet_address }}</td>
            <td class="px-4 py-3">
              <span
                :class="{
                  'text-yellow-600 bg-yellow-50': w.status === 'pending',
                  'text-blue-600 bg-blue-50': w.status === 'approved',
                  'text-red-600 bg-red-50': w.status === 'rejected',
                  'text-green-600 bg-green-50': w.status === 'completed',
                }"
                class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              >
                {{ statusMap[w.status] || w.status }}
              </span>
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 text-xs">{{ w.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3 space-x-1">
              <template v-if="w.status === 'pending'">
                <button @click="approve(w.id)" class="text-green-600 hover:text-green-800 text-xs">{{ $t('withdrawal.approve') }}</button>
                <button @click="openReject(w.id)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('withdrawal.reject') }}</button>
              </template>
              <template v-else-if="w.status === 'approved'">
                <button @click="openComplete(w.id)" class="text-blue-600 hover:text-blue-800 text-xs">{{ $t('withdrawal.complete') }}</button>
              </template>
              <span v-else class="text-gray-400 dark:text-gray-500 text-xs">-</span>
            </td>
          </tr>
          <tr v-if="withdrawals.length === 0">
            <td colspan="9" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
