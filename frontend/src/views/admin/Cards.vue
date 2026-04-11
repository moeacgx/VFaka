<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'

const { t } = useI18n()
const loading = ref(true)
const cards = ref<any[]>([])
const products = ref<any[]>([])
const filterProduct = ref<number | string>('')
const filterVariant = ref<number | string>('')
const filterStatus = ref('')
const showImport = ref(false)
const importForm = ref({ product_id: null as number | null, variant_id: null as number | null, cards: '' })

const statusMap = computed<Record<string, string>>(() => ({
  available: t('card.available'),
  sold: t('card.sold'),
  locked: t('card.locked'),
}))

const importVariants = computed(() => {
  if (!importForm.value.product_id) return []
  const product = products.value.find(p => p.id === importForm.value.product_id)
  return product?.variants || []
})

const filterVariants = computed(() => {
  if (!filterProduct.value) return []
  const product = products.value.find(p => p.id === Number(filterProduct.value))
  return product?.variants || []
})

async function load() {
  loading.value = true
  try {
    const params: any = {}
    if (filterProduct.value) params.product_id = filterProduct.value
    if (filterVariant.value) params.variant_id = filterVariant.value
    if (filterStatus.value) params.status = filterStatus.value
    const [cRes, pRes] = await Promise.all([adminApi.getCards(params), adminApi.getProducts()])
    cards.value = cRes.data || []
    products.value = pRes.data || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function onFilterProductChange() {
  filterVariant.value = ''
  load()
}

function getVariantName(variantId: number | null | undefined) {
  if (!variantId) return ''
  for (const p of products.value) {
    const v = p.variants?.find((v: any) => v.id === variantId)
    if (v) return v.name
  }
  return ''
}

function maskContent(s: string) {
  if (!s) return '-'
  if (s.length <= 4) return '****'
  return s.slice(0, 2) + '****' + s.slice(-2)
}

function getProductName(id: number) {
  return products.value.find(p => p.id === id)?.name || '-'
}

function openImport() {
  importForm.value = { product_id: products.value[0]?.id || null, variant_id: null, cards: '' }
  showImport.value = true
}

async function submitImport() {
  if (!importForm.value.product_id || !importForm.value.cards.trim()) {
    alert(t('card.select_product_cards'))
    return
  }
  try {
    const cardsList = importForm.value.cards.split('\n').map(s => s.trim()).filter(Boolean)
    await adminApi.importCards({
      product_id: importForm.value.product_id,
      variant_id: importForm.value.variant_id || undefined,
      cards: cardsList.join('\n'),
    })
    showImport.value = false
    await load()
    alert(t('card.import_success', { count: cardsList.length }))
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function remove(id: number) {
  if (!confirm(t('common.confirm_delete'))) return
  try {
    await adminApi.deleteCard(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('card.card_list') }}</h3>
      <div class="flex flex-wrap items-center gap-2">
        <select v-model="filterProduct" @change="onFilterProductChange" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
          <option value="">{{ $t('card.all_products') }}</option>
          <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
        <select v-if="filterVariants.length > 0" v-model="filterVariant" @change="load" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
          <option value="">{{ $t('product.all_variants') }}</option>
          <option v-for="v in filterVariants" :key="v.id" :value="v.id">{{ v.name }}</option>
        </select>
        <select v-model="filterStatus" @change="load" class="px-3 py-1.5 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
          <option value="">{{ $t('card.all_status') }}</option>
          <option value="available">{{ $t('card.available') }}</option>
          <option value="sold">{{ $t('card.sold') }}</option>
          <option value="locked">{{ $t('card.locked') }}</option>
        </select>
        <button @click="openImport" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">{{ $t('card.batch_import') }}</button>
      </div>
    </div>

    <!-- Import modal -->
    <div v-if="showImport" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showImport = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-lg p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('card.import_title') }}</h3>
        <form @submit.prevent="submitImport" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('card.product_select') }}</label>
            <select v-model="importForm.product_id" @change="importForm.variant_id = null" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div v-if="importVariants.length > 0">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('product.variant') }}</label>
            <select v-model="importForm.variant_id" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option :value="null">-- {{ $t('product.select_variant') }} --</option>
              <option v-for="v in importVariants" :key="v.id" :value="v.id">{{ v.name }} (¥{{ v.price?.toFixed(2) }})</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('card.import_hint') }}</label>
            <textarea v-model="importForm.cards" rows="8" :placeholder="$t('card.import_hint')" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showImport = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">{{ $t('common.submit') }}</button>
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
            <th class="px-4 py-3 font-medium">{{ $t('dashboard.product') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('product.variant') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('card.content') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.created_at') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="card in cards" :key="card.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ card.id }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ getProductName(card.product_id) }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300 text-xs">{{ getVariantName(card.variant_id) || '-' }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300 font-mono text-xs">{{ maskContent(card.content) }}</td>
            <td class="px-4 py-3">
              <span
                :class="{
                  'text-green-600 bg-green-50': card.status === 'available',
                  'text-gray-500 bg-gray-100': card.status === 'sold',
                  'text-yellow-600 bg-yellow-50': card.status === 'locked',
                }"
                class="inline-block px-2 py-0.5 rounded text-xs font-medium"
              >
                {{ statusMap[card.status] || card.status }}
              </span>
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 text-xs">{{ card.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3">
              <button v-if="card.status === 'available'" @click="remove(card.id)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('common.delete') }}</button>
            </td>
          </tr>
          <tr v-if="cards.length === 0">
            <td colspan="7" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
