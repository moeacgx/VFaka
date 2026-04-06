<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const withdrawals = ref<any[]>([])
const filterStatus = ref('')
const showComplete = ref(false)
const showReject = ref(false)
const selectedId = ref<number | null>(null)
const txHash = ref('')
const rejectNote = ref('')

const statusMap: Record<string, string> = {
  pending: '待审核', approved: '已批准', rejected: '已拒绝', completed: '已完成',
}

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
  if (!confirm('确定批准该提现申请？')) return
  try {
    await adminApi.approveWithdrawal(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '操作失败')
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
    alert(e.response?.data?.error || '操作失败')
  }
}

function openComplete(id: number) {
  selectedId.value = id
  txHash.value = ''
  showComplete.value = true
}

async function submitComplete() {
  if (selectedId.value === null || !txHash.value.trim()) {
    alert('请输入交易哈希')
    return
  }
  try {
    await adminApi.completeWithdrawal(selectedId.value, { tx_hash: txHash.value })
    showComplete.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '操作失败')
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800">提现管理</h3>
      <select v-model="filterStatus" @change="load" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
        <option value="">全部状态</option>
        <option value="pending">待审核</option>
        <option value="approved">已批准</option>
        <option value="rejected">已拒绝</option>
        <option value="completed">已完成</option>
      </select>
    </div>

    <!-- Complete modal -->
    <div v-if="showComplete" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showComplete = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">完成提现</h3>
        <form @submit.prevent="submitComplete" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">交易哈希 (TX Hash)</label>
            <input v-model="txHash" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" placeholder="0x..." />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showComplete = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700">确认完成</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Reject modal -->
    <div v-if="showReject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showReject = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">拒绝提现</h3>
        <form @submit.prevent="submitReject" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">拒绝原因</label>
            <textarea v-model="rejectNote" rows="3" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" placeholder="请输入拒绝原因（可选）" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showReject = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700">确认拒绝</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Table -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-x-auto">
      <div v-if="loading" class="p-8 text-center text-gray-400 text-sm">加载中...</div>
      <table v-else class="w-full text-sm">
        <thead>
          <tr class="bg-gray-50 text-left text-gray-500">
            <th class="px-4 py-3 font-medium">ID</th>
            <th class="px-4 py-3 font-medium">邮箱</th>
            <th class="px-4 py-3 font-medium">金额</th>
            <th class="px-4 py-3 font-medium">币种</th>
            <th class="px-4 py-3 font-medium">链</th>
            <th class="px-4 py-3 font-medium">钱包地址</th>
            <th class="px-4 py-3 font-medium">状态</th>
            <th class="px-4 py-3 font-medium">时间</th>
            <th class="px-4 py-3 font-medium">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="w in withdrawals" :key="w.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 text-gray-600">{{ w.id }}</td>
            <td class="px-4 py-3 text-gray-800">{{ w.email }}</td>
            <td class="px-4 py-3 text-gray-800">{{ w.amount?.toFixed(2) }}</td>
            <td class="px-4 py-3 text-gray-600">{{ w.currency || 'USDT' }}</td>
            <td class="px-4 py-3 text-gray-600">{{ w.chain || '-' }}</td>
            <td class="px-4 py-3 text-gray-600 font-mono text-xs max-w-[200px] truncate" :title="w.wallet_address">{{ w.wallet_address }}</td>
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
            <td class="px-4 py-3 text-gray-500 text-xs">{{ w.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3 space-x-1">
              <template v-if="w.status === 'pending'">
                <button @click="approve(w.id)" class="text-green-600 hover:text-green-800 text-xs">批准</button>
                <button @click="openReject(w.id)" class="text-red-600 hover:text-red-800 text-xs">拒绝</button>
              </template>
              <template v-else-if="w.status === 'approved'">
                <button @click="openComplete(w.id)" class="text-blue-600 hover:text-blue-800 text-xs">完成</button>
              </template>
              <span v-else class="text-gray-400 text-xs">-</span>
            </td>
          </tr>
          <tr v-if="withdrawals.length === 0">
            <td colspan="9" class="px-4 py-8 text-center text-gray-400">暂无提现记录</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
