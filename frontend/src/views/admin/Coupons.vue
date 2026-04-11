<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const coupons = ref<any[]>([])
const total = ref(0)
const page = ref(1)
const perPage = 20
const showForm = ref(false)
const editing = ref<any>(null)
const products = ref<any[]>([])
const selectedIds = ref<number[]>([])

const defaultForm = () => ({
  code: '',
  discount_type: 'fixed',
  discount_value: 0,
  product_id: null as number | null,
  min_amount: 0,
  max_uses: null as number | null,
  valid_from: '',
  valid_to: '',
  is_active: true,
})
const form = ref(defaultForm())

const allSelected = computed({
  get: () => coupons.value.length > 0 && selectedIds.value.length === coupons.value.length,
  set: (val: boolean) => {
    selectedIds.value = val ? coupons.value.map(c => c.id) : []
  }
})

const totalPages = computed(() => Math.ceil(total.value / perPage))

async function load() {
  loading.value = true
  try {
    const res = await adminApi.getCoupons({ page: page.value, per_page: perPage })
    coupons.value = res.data.items || []
    total.value = res.data.total || 0
    selectedIds.value = []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function loadProducts() {
  try {
    const res = await adminApi.getProducts({ per_page: 200 })
    products.value = res.data.items || res.data || []
  } catch { /* ignore */ }
}

function openAdd() {
  editing.value = null
  form.value = defaultForm()
  showForm.value = true
}

function openEdit(coupon: any) {
  editing.value = coupon
  form.value = {
    code: coupon.code,
    discount_type: coupon.discount_type,
    discount_value: coupon.discount_value,
    product_id: coupon.product_id,
    min_amount: coupon.min_amount || 0,
    max_uses: coupon.max_uses,
    valid_from: coupon.valid_from ? coupon.valid_from.slice(0, 16) : '',
    valid_to: coupon.valid_to ? coupon.valid_to.slice(0, 16) : '',
    is_active: coupon.is_active !== false,
  }
  showForm.value = true
}

async function save() {
  try {
    const data: any = {
      ...form.value,
      product_id: form.value.product_id || null,
      max_uses: form.value.max_uses || null,
      valid_from: form.value.valid_from ? new Date(form.value.valid_from).toISOString() : null,
      valid_to: form.value.valid_to ? new Date(form.value.valid_to).toISOString() : null,
    }
    if (editing.value) {
      await adminApi.updateCoupon(editing.value.id, data)
    } else {
      await adminApi.createCoupon(data)
    }
    showForm.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function remove(id: number) {
  if (!confirm(t('common.confirm_delete'))) return
  try {
    await adminApi.deleteCoupon(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function batchDelete() {
  if (selectedIds.value.length === 0) return
  if (!confirm(t('common.confirm_delete') + ` (${selectedIds.value.length})`)) return
  try {
    await adminApi.batchDeleteCoupons(selectedIds.value)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

function toggleSelect(id: number) {
  const idx = selectedIds.value.indexOf(id)
  if (idx >= 0) selectedIds.value.splice(idx, 1)
  else selectedIds.value.push(id)
}

async function toggleStatus(coupon: any) {
  try {
    await adminApi.updateCoupon(coupon.id, { is_active: !coupon.is_active })
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

function productName(pid: number | null) {
  if (!pid) return t('coupon.global')
  const p = products.value.find(x => x.id === pid)
  return p ? p.name : `#${pid}`
}

function formatDate(d: string | null) {
  if (!d) return '-'
  return new Date(d).toLocaleString()
}

onMounted(() => {
  load()
  loadProducts()
})
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('coupon.title') }}</h3>
      <div class="flex gap-2">
        <button v-if="selectedIds.length > 0" @click="batchDelete" class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors">
          {{ $t('common.batch_delete') }} ({{ selectedIds.length }})
        </button>
        <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">
          {{ $t('coupon.add') }}
        </button>
      </div>
    </div>

    <!-- Form modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showForm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-lg p-6 max-h-[90vh] overflow-y-auto">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ editing ? $t('coupon.edit') : $t('coupon.add') }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.code') }}</label>
            <input v-model="form.code" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white uppercase" placeholder="e.g. SAVE20" />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.discount_type') }}</label>
              <select v-model="form.discount_type" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white">
                <option value="fixed">{{ $t('coupon.fixed') }}</option>
                <option value="percentage">{{ $t('coupon.percentage') }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.discount_value') }}</label>
              <input v-model.number="form.discount_value" type="number" step="0.01" min="0" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.product') }}</label>
            <select v-model="form.product_id" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white">
              <option :value="null">{{ $t('coupon.global') }}</option>
              <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.min_amount') }}</label>
              <input v-model.number="form.min_amount" type="number" step="0.01" min="0" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.max_uses') }}</label>
              <input v-model.number="form.max_uses" type="number" min="0" :placeholder="$t('coupon.unlimited')" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.valid_from') }}</label>
              <input v-model="form.valid_from" type="datetime-local" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('coupon.valid_to') }}</label>
              <input v-model="form.valid_to" type="datetime-local" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div class="flex items-center gap-2">
            <input v-model="form.is_active" type="checkbox" id="coupon-active" class="rounded" />
            <label for="coupon-active" class="text-sm text-gray-700 dark:text-gray-200">{{ $t('common.enabled') }}</label>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">{{ $t('common.save') }}</button>
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
            <th class="px-4 py-3 w-10"><input type="checkbox" v-model="allSelected" /></th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.code') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.discount_type') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.discount_value') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.product') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.usage') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('coupon.valid_to') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="c in coupons" :key="c.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3"><input type="checkbox" :checked="selectedIds.includes(c.id)" @change="toggleSelect(c.id)" /></td>
            <td class="px-4 py-3 font-mono text-gray-800 dark:text-gray-100">{{ c.code }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ c.discount_type === 'percentage' ? $t('coupon.percentage') : $t('coupon.fixed') }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ c.discount_type === 'percentage' ? `${c.discount_value}%` : `¥${c.discount_value.toFixed(2)}` }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ productName(c.product_id) }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ c.used_count }}{{ c.max_uses != null ? ` / ${c.max_uses}` : '' }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ formatDate(c.valid_to) }}</td>
            <td class="px-4 py-3">
              <button @click="toggleStatus(c)" :class="c.is_active ? 'text-green-600 bg-green-50 dark:bg-green-900/30' : 'text-gray-500 bg-gray-100 dark:bg-gray-700'" class="px-2 py-0.5 rounded text-xs font-medium">
                {{ c.is_active ? $t('common.enabled') : $t('common.disabled') }}
              </button>
            </td>
            <td class="px-4 py-3 space-x-2">
              <button @click="openEdit(c)" class="text-blue-600 hover:text-blue-800 text-xs">{{ $t('common.edit') }}</button>
              <button @click="remove(c.id)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('common.delete') }}</button>
            </td>
          </tr>
          <tr v-if="coupons.length === 0">
            <td colspan="9" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex justify-center items-center gap-4 mt-4 text-sm text-gray-600 dark:text-gray-300">
      <button @click="page > 1 && (page--, load())" :disabled="page <= 1" class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded disabled:opacity-30">{{ $t('common.prev_page') }}</button>
      <span>{{ $t('common.page_info', { current: page, total: totalPages }) }}</span>
      <button @click="page < totalPages && (page++, load())" :disabled="page >= totalPages" class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded disabled:opacity-30">{{ $t('common.next_page') }}</button>
    </div>
  </div>
</template>
