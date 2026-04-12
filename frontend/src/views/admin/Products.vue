<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const products = ref<any[]>([])
const categories = ref<any[]>([])
const showForm = ref(false)
const showRestock = ref(false)
const editing = ref<any>(null)
const uploadingImage = ref(false)
const uploadingVideo = ref(false)
const selectedIds = ref<number[]>([])
const allowCommandAction = ref(false)

const allSelected = computed({
  get: () => products.value.length > 0 && selectedIds.value.length === products.value.length,
  set: (val: boolean) => {
    selectedIds.value = val ? products.value.map(p => p.id) : []
  }
})

const defaultForm = () => ({
  name: '', category_id: null as number | null, price: 0, description: '',
  allow_alipay: true, allow_wxpay: true, allow_qqpay: false,
  allow_usdt_trc20: false, allow_usdt_erc20: false, allow_trx: false,
  post_pay_action_type: 'none', post_pay_action_value: '',
  aff_commission_rate: null as number | null,
  sort_order: 0, min_quantity: 1, max_quantity: 10, is_active: true,
  image_url: null as string | null, video_url: null as string | null,
})
const form = ref(defaultForm())
const restockForm = ref({ product_id: null as number | null, cards: '' })

// Variant management state
const showVariants = ref<number | null>(null)
const variantList = ref<any[]>([])
const variantLoading = ref(false)
const showVariantForm = ref(false)
const editingVariant = ref<any>(null)
const variantForm = ref({ name: '', price: 0, description: '', sort_order: 0, is_active: true })
const restockVariantId = ref<number | null>(null)

async function uploadFile(file: File): Promise<string> {
  const formData = new FormData()
  formData.append('file', file)
  const res = await adminApi.upload(formData)
  return res.data.url
}

async function handleImageUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploadingImage.value = true
  try {
    form.value.image_url = await uploadFile(file)
  } catch (err: any) {
    alert(err.message || t('common.operation_failed'))
  } finally {
    uploadingImage.value = false
  }
}

async function handleVideoUpload(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (!file) return
  uploadingVideo.value = true
  try {
    form.value.video_url = await uploadFile(file)
  } catch (err: any) {
    alert(err.message || t('common.operation_failed'))
  } finally {
    uploadingVideo.value = false
  }
}

async function load() {
  loading.value = true
  try {
    const [pRes, cRes, cfgRes] = await Promise.all([
      adminApi.getProducts(),
      adminApi.getCategories(),
      adminApi.getAdminConfig().catch(() => ({ data: {} })),
    ])
    products.value = pRes.data || []
    categories.value = cRes.data || []
    allowCommandAction.value = cfgRes.data?.allow_command_action ?? false
    selectedIds.value = []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function openAdd() {
  editing.value = null
  form.value = defaultForm()
  if (categories.value.length) form.value.category_id = categories.value[0].id
  showForm.value = true
}

function openEdit(p: any) {
  editing.value = p
  form.value = {
    name: p.name, category_id: p.category_id, price: p.price, description: p.description || '',
    allow_alipay: !!p.allow_alipay, allow_wxpay: !!p.allow_wxpay, allow_qqpay: !!p.allow_qqpay,
    allow_usdt_trc20: !!p.allow_usdt_trc20, allow_usdt_erc20: !!p.allow_usdt_erc20, allow_trx: !!p.allow_trx,
    post_pay_action_type: p.post_pay_action_type || 'none', post_pay_action_value: p.post_pay_action_value || '',
    aff_commission_rate: p.aff_commission_rate ?? null,
    sort_order: p.sort_order || 0, min_quantity: p.min_quantity || 1, max_quantity: p.max_quantity || 10, is_active: p.is_active !== false,
    image_url: p.image_url || null, video_url: p.video_url || null,
  }
  showForm.value = true
}

async function save() {
  try {
    const data = { ...form.value }
    if (editing.value) {
      await adminApi.updateProduct(editing.value.id, data)
    } else {
      await adminApi.createProduct(data)
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
    await adminApi.deleteProduct(id)
    selectedIds.value = selectedIds.value.filter(sid => sid !== id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function duplicate(id: number) {
  try {
    await adminApi.duplicateProduct(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function batchDelete() {
  if (selectedIds.value.length === 0) return
  if (!confirm(t('common.confirm_delete') + ` (${selectedIds.value.length})`)) return
  try {
    await adminApi.batchDeleteProducts(selectedIds.value)
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

function openRestock() {
  restockForm.value = { product_id: products.value[0]?.id || null, cards: '' }
  restockVariantId.value = null
  showRestock.value = true
}

async function submitRestock() {
  if (!restockForm.value.product_id || !restockForm.value.cards.trim()) {
    alert(t('product.select_product_cards'))
    return
  }
  try {
    const cards = restockForm.value.cards.split('\n').map(s => s.trim()).filter(Boolean)
    await adminApi.restockProduct(restockForm.value.product_id, {
      cards: cards.join('\n'),
      variant_id: restockVariantId.value || undefined,
    })
    showRestock.value = false
    await load()
    alert(t('product.import_success', { count: cards.length }))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

// --- Variant management ---
async function toggleVariants(productId: number) {
  if (showVariants.value === productId) {
    showVariants.value = null
    return
  }
  showVariants.value = productId
  await loadVariants(productId)
}

async function loadVariants(productId: number) {
  variantLoading.value = true
  try {
    const res = await adminApi.getVariants(productId)
    variantList.value = res.data || []
  } catch (e) {
    console.error(e)
    variantList.value = []
  } finally {
    variantLoading.value = false
  }
}

function openAddVariant() {
  editingVariant.value = null
  variantForm.value = { name: '', price: 0, description: '', sort_order: 0, is_active: true }
  showVariantForm.value = true
}

function openEditVariant(v: any) {
  editingVariant.value = v
  variantForm.value = {
    name: v.name,
    price: v.price,
    description: v.description || '',
    sort_order: v.sort_order || 0,
    is_active: v.is_active !== false,
  }
  showVariantForm.value = true
}

async function saveVariant() {
  if (!showVariants.value) return
  try {
    if (editingVariant.value) {
      await adminApi.updateVariant(editingVariant.value.id, variantForm.value)
    } else {
      await adminApi.createVariant(showVariants.value, variantForm.value)
    }
    showVariantForm.value = false
    await loadVariants(showVariants.value)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function removeVariant(id: number) {
  if (!confirm(t('common.confirm_delete'))) return
  if (!showVariants.value) return
  try {
    await adminApi.deleteVariant(id)
    await loadVariants(showVariants.value)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

// Compute available variants for restock
const restockVariants = computed(() => {
  if (!restockForm.value.product_id) return []
  const product = products.value.find(p => p.id === restockForm.value.product_id)
  return product?.variants || []
})

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('product.product_list') }}</h3>
      <div class="flex gap-2">
        <button v-if="selectedIds.length > 0" @click="batchDelete" class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors">{{ $t('common.batch_delete') }} ({{ selectedIds.length }})</button>
        <button @click="openRestock" class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 transition-colors">{{ $t('product.restock') }}</button>
        <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">{{ $t('product.add_product') }}</button>
      </div>
    </div>

    <!-- Product form modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showForm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-2xl max-h-[90vh] overflow-y-auto p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ editing ? $t('product.edit_product') : $t('product.add_product') }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.name') }}</label>
              <input v-model="form.name" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.category') }}</label>
              <select v-model="form.category_id" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.price') }} (¥)</label>
              <input v-model.number="form.price" type="number" step="0.01" min="0" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.sort_order') }}</label>
              <input v-model.number="form.sort_order" type="number" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.min_quantity') }}</label>
              <input v-model.number="form.min_quantity" type="number" min="1" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.max_quantity') }}</label>
              <input v-model.number="form.max_quantity" type="number" min="1" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.description') }}</label>
            <textarea v-model="form.description" rows="3" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <!-- Image upload -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.image') }}</label>
            <div class="flex items-center gap-3">
              <label class="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-sm rounded-md cursor-pointer hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors">
                {{ uploadingImage ? $t('common.loading') : $t('product.upload_image') }}
                <input type="file" accept="image/*" class="hidden" @change="handleImageUpload" :disabled="uploadingImage" />
              </label>
              <span v-if="form.image_url" class="text-xs text-green-600">✓</span>
            </div>
            <img v-if="form.image_url" :src="form.image_url" class="mt-2 max-h-32 rounded border border-gray-200 dark:border-gray-700" />
          </div>
          <!-- Video upload -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.video') }}</label>
            <div class="flex items-center gap-3">
              <label class="px-3 py-1.5 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 text-sm rounded-md cursor-pointer hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors">
                {{ uploadingVideo ? $t('common.loading') : $t('product.upload_video') }}
                <input type="file" accept="video/*" class="hidden" @change="handleVideoUpload" :disabled="uploadingVideo" />
              </label>
              <span v-if="form.video_url" class="text-xs text-green-600">✓</span>
            </div>
            <video v-if="form.video_url" :src="form.video_url" controls class="mt-2 max-h-32 rounded border border-gray-200 dark:border-gray-700" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-2">{{ $t('product.payment_methods') }}</label>
            <div class="flex flex-wrap gap-4">
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_alipay" type="checkbox" /> {{ $t('product.alipay') }}</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_wxpay" type="checkbox" /> {{ $t('product.wxpay') }}</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_qqpay" type="checkbox" /> {{ $t('product.qqpay') }}</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_usdt_trc20" type="checkbox" /> {{ $t('product.usdt_trc20') }}</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_usdt_erc20" type="checkbox" /> {{ $t('product.usdt_erc20') }}</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700 dark:text-gray-200"><input v-model="form.allow_trx" type="checkbox" /> {{ $t('product.trx') }}</label>
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.post_action_type') }}</label>
              <select v-model="form.post_pay_action_type" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
                <option value="none">{{ $t('common.none') }}</option>
                <option value="webhook">{{ $t('product.webhook') }}</option>
                <option v-if="allowCommandAction" value="command">{{ $t('product.command') }}</option>
              </select>
            </div>
            <div v-if="form.post_pay_action_type !== 'none'">
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.post_action_value') }}</label>
              <input v-model="form.post_pay_action_value" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.aff_rate') }} (%)</label>
              <input v-model.number="form.aff_commission_rate" type="number" step="0.1" min="0" max="100" :placeholder="$t('product.aff_rate_hint')" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
            </div>
            <div class="flex items-end">
              <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200 pb-2"><input v-model="form.is_active" type="checkbox" /> {{ $t('product.enable_product') }}</label>
            </div>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">{{ $t('common.save') }}</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Restock modal -->
    <div v-if="showRestock" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showRestock = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-lg p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('product.restock_title') }}</h3>
        <form @submit.prevent="submitRestock" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('card.product_select') }}</label>
            <select v-model="restockForm.product_id" @change="restockVariantId = null" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div v-if="restockVariants.length > 0">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.variant') }}</label>
            <select v-model="restockVariantId" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option :value="null">-- {{ $t('product.select_variant') }} --</option>
              <option v-for="v in restockVariants" :key="v.id" :value="v.id">{{ v.name }} (¥{{ v.price?.toFixed(2) }})</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('card.import_hint') }}</label>
            <textarea v-model="restockForm.cards" rows="8" :placeholder="$t('card.import_hint')" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showRestock = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700">{{ $t('common.submit') }}</button>
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
            <th class="px-4 py-3 font-medium">ID</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.name') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.category') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.price') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.stock') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.sales') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <template v-for="p in products" :key="p.id">
          <tr class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3"><input type="checkbox" :checked="selectedIds.includes(p.id)" @change="toggleSelect(p.id)" /></td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ p.id }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ p.name }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ p.category_name || '-' }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">¥{{ p.price?.toFixed(2) }}</td>
            <td class="px-4 py-3" :class="(p.stock_count ?? 0) < 5 ? 'text-red-600 font-medium' : 'text-gray-600 dark:text-gray-300'">{{ p.stock_count ?? 0 }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ p.sales_count ?? 0 }}</td>
            <td class="px-4 py-3">
              <span :class="p.is_active ? 'text-green-600 bg-green-50' : 'text-gray-500 bg-gray-100'" class="inline-block px-2 py-0.5 rounded text-xs font-medium">
                {{ p.is_active ? $t('product.active') : $t('product.inactive') }}
              </span>
            </td>
            <td class="px-4 py-3 space-x-2">
              <button @click="toggleVariants(p.id)" class="text-purple-600 hover:text-purple-800 text-xs">{{ $t('product.variants') }}</button>
              <button @click="duplicate(p.id)" class="text-green-600 hover:text-green-800 text-xs">{{ $t('product.duplicate') }}</button>
              <button @click="openEdit(p)" class="text-blue-600 hover:text-blue-800 text-xs">{{ $t('common.edit') }}</button>
              <button @click="remove(p.id)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('common.delete') }}</button>
            </td>
          </tr>
          <!-- Variant expansion row -->
          <tr v-if="showVariants === p.id" :key="'v-' + p.id" class="bg-gray-50/50 dark:bg-gray-800/50">
            <td :colspan="9" class="px-6 py-4">
              <div class="flex items-center justify-between mb-3">
                <span class="text-sm font-medium text-gray-700 dark:text-gray-200">{{ $t('product.variants') }}</span>
                <button @click="openAddVariant" class="px-2.5 py-1 bg-purple-600 text-white text-xs rounded hover:bg-purple-700 transition-colors">+ {{ $t('product.add_variant') }}</button>
              </div>
              <div v-if="variantLoading" class="text-gray-400 text-sm py-2">{{ $t('common.loading') }}</div>
              <div v-else-if="variantList.length === 0" class="text-gray-400 text-sm py-2">{{ $t('product.no_variants') }}</div>
              <table v-else class="w-full text-xs">
                <thead>
                  <tr class="text-left text-gray-500 dark:text-gray-400 border-b border-gray-200 dark:border-gray-700">
                    <th class="px-3 py-2 font-medium">{{ $t('product.variant_name') }}</th>
                    <th class="px-3 py-2 font-medium">{{ $t('product.price') }}</th>
                    <th class="px-3 py-2 font-medium">{{ $t('product.stock') }}</th>
                    <th class="px-3 py-2 font-medium">{{ $t('product.sales') }}</th>
                    <th class="px-3 py-2 font-medium">{{ $t('common.status') }}</th>
                    <th class="px-3 py-2 font-medium">{{ $t('common.actions') }}</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                  <tr v-for="v in variantList" :key="v.id" class="hover:bg-white dark:hover:bg-gray-700">
                    <td class="px-3 py-2 text-gray-800 dark:text-gray-100">{{ v.name }}</td>
                    <td class="px-3 py-2 text-gray-800 dark:text-gray-100">¥{{ v.price?.toFixed(2) }}</td>
                    <td class="px-3 py-2" :class="(v.stock_count ?? 0) < 5 ? 'text-red-600 font-medium' : 'text-gray-600 dark:text-gray-300'">{{ v.stock_count ?? 0 }}</td>
                    <td class="px-3 py-2 text-gray-600 dark:text-gray-300">{{ v.sales_count ?? 0 }}</td>
                    <td class="px-3 py-2">
                      <span :class="v.is_active ? 'text-green-600 bg-green-50' : 'text-gray-500 bg-gray-100'" class="inline-block px-1.5 py-0.5 rounded text-xs font-medium">
                        {{ v.is_active ? $t('product.active') : $t('product.inactive') }}
                      </span>
                    </td>
                    <td class="px-3 py-2 space-x-2">
                      <button @click="openEditVariant(v)" class="text-blue-600 hover:text-blue-800">{{ $t('common.edit') }}</button>
                      <button @click="removeVariant(v.id)" class="text-red-600 hover:text-red-800">{{ $t('common.delete') }}</button>
                    </td>
                  </tr>
                </tbody>
              </table>
            </td>
          </tr>
          </template>
          <tr v-if="products.length === 0">
            <td colspan="9" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Variant form modal -->
    <div v-if="showVariantForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showVariantForm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ editingVariant ? $t('product.edit_variant') : $t('product.add_variant') }}</h3>
        <form @submit.prevent="saveVariant" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.variant_name') }}</label>
            <input v-model="variantForm.name" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.price') }} (¥)</label>
              <input v-model.number="variantForm.price" type="number" step="0.01" min="0" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.sort_order') }}</label>
              <input v-model.number="variantForm.sort_order" type="number" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.description') }}</label>
            <textarea v-model="variantForm.description" rows="2" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-200"><input v-model="variantForm.is_active" type="checkbox" /> {{ $t('product.enable_product') }}</label>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showVariantForm = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-purple-600 text-white text-sm rounded-md hover:bg-purple-700">{{ $t('common.save') }}</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
