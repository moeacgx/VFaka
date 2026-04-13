<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { publicApi } from '../../api/public'
import { useAffCode } from '../../composables/useAffCode'

const { t } = useI18n()

interface Variant {
  id: number
  product_id: number
  name: string
  price: number
  description: string | null
  sort_order: number
  is_active: boolean
  stock_count: number
  sales_count: number
}

interface Category {
  id: number
  name: string
  sort_order: number
  is_active: boolean
}

interface Product {
  id: number
  category_id: number | null
  category_name: string | null
  name: string
  description: string | null
  price: number
  stock_count: number
  sales_count: number
  is_active: boolean
  allow_alipay: boolean
  allow_wxpay: boolean
  allow_qqpay: boolean
  allow_usdt_trc20: boolean
  allow_trx: boolean
  allow_usdt_erc20: boolean
  min_quantity: number
  max_quantity: number
  image_url?: string
  video_url?: string
  variants?: Variant[]
}

const router = useRouter()
const { getAffCode } = useAffCode()

const categories = ref<Category[]>([])
const products = ref<Product[]>([])
const activeCategory = ref<number | null>(null)
const loading = ref(true)
const error = ref('')

// Modal state
const showModal = ref(false)
const selectedProduct = ref<Product | null>(null)
const selectedVariantId = ref<number | null>(null)
const orderEmail = ref('')
const orderPassword = ref('')
const orderQuantity = ref(1)
const paymentMethod = ref('')
const orderLoading = ref(false)
const orderError = ref('')
const paymentFallbackUrl = ref('')
const pendingOrderNo = ref('')
const pendingOrderToken = ref('')

// Coupon state
const couponCode = ref('')
const couponLoading = ref(false)
const couponResult = ref<{
  valid: boolean
  discount_type?: string
  discount_value?: number
  discount_amount?: number
  message?: string
} | null>(null)
let couponTimer: ReturnType<typeof setTimeout> | null = null

const filteredProducts = computed(() => {
  if (activeCategory.value === null) return products.value
  return products.value.filter(p => p.category_id === activeCategory.value)
})

const activeVariants = computed(() => {
  if (!selectedProduct.value?.variants) return []
  return selectedProduct.value.variants.filter(v => v.is_active)
})

const hasVariants = computed(() => activeVariants.value.length > 0)

const selectedVariant = computed(() => {
  if (!hasVariants.value || !selectedVariantId.value) return null
  return activeVariants.value.find(v => v.id === selectedVariantId.value) || null
})

const currentPrice = computed(() => {
  if (selectedVariant.value) return selectedVariant.value.price
  return selectedProduct.value?.price || 0
})

const currentStock = computed(() => {
  if (selectedVariant.value) return selectedVariant.value.stock_count
  return selectedProduct.value?.stock_count || 0
})

const availableFiat = computed(() => {
  if (!selectedProduct.value) return []
  const p = selectedProduct.value
  const methods: { key: string; label: string }[] = []
  if (p.allow_alipay) methods.push({ key: 'alipay', label: t('product.alipay') })
  if (p.allow_wxpay) methods.push({ key: 'wxpay', label: t('product.wxpay') })
  if (p.allow_qqpay) methods.push({ key: 'qqpay', label: t('product.qqpay') })
  return methods
})

const availableCrypto = computed(() => {
  if (!selectedProduct.value) return []
  const p = selectedProduct.value
  const methods: { key: string; label: string }[] = []
  if (p.allow_usdt_trc20) methods.push({ key: 'usdt_trc20', label: t('product.usdt_trc20') })
  if (p.allow_trx) methods.push({ key: 'trx', label: t('product.trx') })
  if (p.allow_usdt_erc20) methods.push({ key: 'usdt_erc20', label: t('product.usdt_erc20') })
  return methods
})

const totalPrice = computed(() => {
  const subtotal = currentPrice.value * orderQuantity.value
  if (couponResult.value?.valid && couponResult.value.discount_amount) {
    return Math.max(subtotal - couponResult.value.discount_amount, 0.01)
  }
  return subtotal
})

const subtotalPrice = computed(() => {
  return currentPrice.value * orderQuantity.value
})

// Reset variant when modal opens or product changes
watch(selectedProduct, () => {
  if (selectedProduct.value && hasVariants.value) {
    const inStock = activeVariants.value.find(v => v.stock_count > 0)
    selectedVariantId.value = inStock?.id || activeVariants.value[0]?.id || null
  } else {
    selectedVariantId.value = null
  }
})

function onCouponInput() {
  couponResult.value = null
  if (couponTimer) clearTimeout(couponTimer)
  const code = couponCode.value.trim()
  if (!code || !selectedProduct.value) return
  couponTimer = setTimeout(() => validateCoupon(code), 500)
}

async function validateCoupon(code: string) {
  if (!selectedProduct.value) return
  couponLoading.value = true
  try {
    const res = await publicApi.validateCoupon({
      code,
      product_id: selectedProduct.value.id,
      amount: currentPrice.value * orderQuantity.value,
    })
    couponResult.value = res.data
  } catch {
    couponResult.value = { valid: false, message: t('coupon.validate_failed') }
  } finally {
    couponLoading.value = false
  }
}

// Revalidate coupon when quantity or variant changes
watch([orderQuantity, selectedVariantId], () => {
  const code = couponCode.value.trim()
  if (code && selectedProduct.value) {
    couponResult.value = null
    if (couponTimer) clearTimeout(couponTimer)
    couponTimer = setTimeout(() => validateCoupon(code), 300)
  }
})

onUnmounted(() => {
  if (couponTimer) {
    clearTimeout(couponTimer)
    couponTimer = null
  }
})

function getProductDisplayPrice(product: Product): string {
  if (product.variants && product.variants.length > 0) {
    const active = product.variants.filter(v => v.is_active)
    if (active.length === 0) return `¥${product.price.toFixed(2)}`
    const prices = active.map(v => v.price)
    const min = Math.min(...prices)
    const max = Math.max(...prices)
    if (min === max) return `¥${min.toFixed(2)}`
    return `¥${min.toFixed(2)}~${max.toFixed(2)}`
  }
  return `¥${product.price.toFixed(2)}`
}

function truncate(text: string | null, len: number) {
  if (!text) return ''
  return text.length > len ? text.slice(0, len) + '...' : text
}

function openBuyModal(product: Product) {
  selectedProduct.value = product
  selectedVariantId.value = null
  orderEmail.value = ''
  orderPassword.value = ''
  orderQuantity.value = product.min_quantity || 1
  paymentMethod.value = ''
  orderError.value = ''
  orderLoading.value = false
  couponCode.value = ''
  couponResult.value = null
  showModal.value = true
}

function closeModal() {
  showModal.value = false
  selectedProduct.value = null
  selectedVariantId.value = null
}

function adjustQuantity(delta: number) {
  if (!selectedProduct.value) return
  const next = orderQuantity.value + delta
  if (next >= selectedProduct.value.min_quantity && next <= selectedProduct.value.max_quantity) {
    orderQuantity.value = next
  }
}

async function submitOrder() {
  if (!selectedProduct.value) return
  if (!orderEmail.value.trim()) {
    orderError.value = t('common.enter_email_required')
    return
  }
  if (hasVariants.value && !selectedVariantId.value) {
    orderError.value = t('product.select_variant_required')
    return
  }
  if (!paymentMethod.value) {
    orderError.value = t('product.select_payment_required')
    return
  }

  orderLoading.value = true
  orderError.value = ''

  const payWindow = window.open('about:blank', '_blank')

  try {
    const affCode = getAffCode()
    const res = await publicApi.createOrder({
      product_id: selectedProduct.value.id,
      variant_id: selectedVariantId.value || undefined,
      quantity: orderQuantity.value,
      email: orderEmail.value.trim(),
      payment_method: paymentMethod.value,
      aff_code: affCode || undefined,
      coupon_code: couponCode.value.trim() || undefined,
      query_password: orderPassword.value.trim() || undefined,
    })
    const data = res.data
    closeModal()
    if (data.payment_url) {
      if (payWindow) {
        payWindow.location.href = data.payment_url
      } else {
        // Popup was blocked — show fallback UI
        pendingOrderNo.value = data.order_no
        pendingOrderToken.value = data.query_token
        paymentFallbackUrl.value = data.payment_url
        return
      }
    } else {
      payWindow?.close()
    }
    router.push({ path: '/order', query: { no: data.order_no, token: data.query_token } })
  } catch (e: any) {
    payWindow?.close()
    orderError.value = e.response?.data?.message || e.response?.data?.error || t('product.create_order_failed')
  } finally {
    orderLoading.value = false
  }
}

function navigateToOrder() {
  const no = pendingOrderNo.value
  const token = pendingOrderToken.value
  paymentFallbackUrl.value = ''
  pendingOrderNo.value = ''
  pendingOrderToken.value = ''
  router.push({ path: '/order', query: { no, token } })
}

onMounted(async () => {
  try {
    const [catRes, prodRes] = await Promise.all([
      publicApi.getCategories(),
      publicApi.getProducts(),
    ])
    categories.value = catRes.data
    products.value = prodRes.data
  } catch (e: any) {
    error.value = t('common.load_failed')
  } finally {
    loading.value = false
  }
  getAffCode()
})
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16">
    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <div class="w-6 h-6 border-2 border-gray-300 dark:border-gray-600 border-t-blue-600 rounded-full animate-spin"></div>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-center py-20">
      <p class="text-red-500 text-sm">{{ error }}</p>
    </div>

    <template v-else>
      <!-- Category tabs -->
      <div class="flex gap-2 mb-8 flex-wrap">
        <button
          @click="activeCategory = null"
          :class="[
            'px-4 py-1.5 rounded-full text-sm font-medium transition-all',
            activeCategory === null
              ? 'bg-gray-900 dark:bg-white text-white dark:text-gray-900'
              : 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-700'
          ]"
        >{{ $t('common.all') }}</button>
        <button
          v-for="cat in categories"
          :key="cat.id"
          @click="activeCategory = cat.id"
          :class="[
            'px-4 py-1.5 rounded-full text-sm font-medium transition-all',
            activeCategory === cat.id
              ? 'bg-gray-900 dark:bg-white text-white dark:text-gray-900'
              : 'bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 border border-gray-200 dark:border-gray-700'
          ]"
        >{{ cat.name }}</button>
      </div>

      <!-- Empty state -->
      <div v-if="filteredProducts.length === 0" class="text-center py-20">
        <p class="text-gray-400 dark:text-gray-500 text-sm">{{ $t('common.no_data') }}</p>
      </div>

      <!-- Product grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="product in filteredProducts"
          :key="product.id"
          class="bg-white dark:bg-gray-800 rounded-xl border border-gray-100 dark:border-gray-700 shadow-sm dark:shadow-none hover:shadow-md transition-shadow flex flex-col"
        >
          <img v-if="product.image_url" :src="product.image_url" :alt="product.name" class="w-full h-40 object-cover rounded-t-xl" />
          <div class="p-5 flex flex-col flex-1">
            <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">{{ product.name }}</h3>
            <p class="text-xs text-gray-400 dark:text-gray-500 mb-3 flex-1 leading-relaxed">{{ truncate(product.description, 60) }}</p>
            <div class="flex items-end justify-between mt-auto">
              <div>
                <span class="text-lg font-bold text-gray-900 dark:text-white">{{ getProductDisplayPrice(product) }}</span>
                <span class="text-xs text-gray-400 dark:text-gray-500 ml-2">{{ $t('product.stock') }} {{ product.stock_count }}</span>
              </div>
              <button
                @click="openBuyModal(product)"
                :disabled="product.stock_count <= 0"
                :class="[
                  'px-4 py-1.5 rounded-lg text-sm font-medium transition-all',
                  product.stock_count > 0
                    ? 'bg-blue-600 text-white hover:bg-blue-700 active:bg-blue-800'
                    : 'bg-gray-100 dark:bg-gray-700 text-gray-400 dark:text-gray-500 cursor-not-allowed'
                ]"
              >{{ product.stock_count > 0 ? $t('product.buy_now') : $t('product.out_of_stock') }}</button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Purchase Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="fixed inset-0 bg-black/30 backdrop-blur-sm" @click="closeModal"></div>
        <div class="relative bg-white dark:bg-gray-800 rounded-2xl shadow-xl w-full max-w-md p-6 max-h-[90vh] overflow-y-auto">
          <!-- Header -->
          <div class="flex items-center justify-between mb-5">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">{{ selectedProduct?.name }}</h3>
            <button @click="closeModal" class="text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors p-1">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Product image in modal -->
          <img v-if="selectedProduct?.image_url" :src="selectedProduct.image_url" :alt="selectedProduct.name" class="w-full h-48 object-cover rounded-xl mb-5" />

          <!-- Variant selector -->
          <div v-if="hasVariants" class="mb-5">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-2">{{ $t('product.select_variant') }}</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="v in activeVariants"
                :key="v.id"
                @click="v.stock_count > 0 ? selectedVariantId = v.id : null"
                :disabled="v.stock_count <= 0"
                :class="[
                  'px-3 py-2 rounded-lg border text-sm font-medium transition-all',
                  v.stock_count <= 0
                    ? 'border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 text-gray-300 dark:text-gray-600 cursor-not-allowed'
                    : selectedVariantId === v.id
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 ring-1 ring-blue-500'
                      : 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500'
                ]"
              >
                <span>{{ v.name }}</span>
                <span class="ml-1 text-xs opacity-70">¥{{ v.price.toFixed(2) }}</span>
                <span class="ml-1 text-xs opacity-50">({{ v.stock_count }})</span>
              </button>
            </div>
            <p v-if="selectedVariant?.description" class="mt-2 text-xs text-gray-400 dark:text-gray-500">{{ selectedVariant.description }}</p>
          </div>

          <!-- Price -->
          <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 mb-5">
            <div class="text-sm text-gray-500 dark:text-gray-400">{{ $t('product.price') }}</div>
            <div class="text-2xl font-bold text-gray-900 dark:text-white">¥{{ currentPrice.toFixed(2) }}</div>
            <div class="text-xs text-gray-400 dark:text-gray-500 mt-1">{{ $t('product.stock') }} {{ currentStock }}</div>
          </div>

          <!-- Email -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1.5">{{ $t('common.email') }}</label>
            <input
              v-model="orderEmail"
              type="email"
              :placeholder="$t('order.email_placeholder')"
              class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
          </div>

          <!-- Query Password (optional) -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1.5">
              {{ $t('order.query_password') }}
              <span class="text-gray-400 dark:text-gray-500 font-normal ml-1">{{ $t('order.query_password_hint') }}</span>
            </label>
            <input
              v-model="orderPassword"
              type="text"
              :placeholder="$t('order.query_password_placeholder')"
              class="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
          </div>

          <!-- Quantity -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1.5">{{ $t('product.quantity') }}</label>
            <div class="flex items-center gap-3">
              <button
                @click="adjustQuantity(-1)"
                :disabled="orderQuantity <= (selectedProduct?.min_quantity || 1)"
                class="w-8 h-8 flex items-center justify-center rounded-lg border border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
              >−</button>
              <span class="text-lg font-semibold text-gray-900 dark:text-white w-8 text-center">{{ orderQuantity }}</span>
              <button
                @click="adjustQuantity(1)"
                :disabled="orderQuantity >= (selectedProduct?.max_quantity || 1)"
                class="w-8 h-8 flex items-center justify-center rounded-lg border border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
              >+</button>
              <span class="text-xs text-gray-400 dark:text-gray-500">（{{ selectedProduct?.min_quantity }} - {{ selectedProduct?.max_quantity }}）</span>
            </div>
          </div>

          <!-- Coupon Code -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1.5">{{ $t('coupon.code') }}</label>
            <div class="relative">
              <input
                v-model="couponCode"
                type="text"
                :placeholder="$t('coupon.placeholder')"
                @input="onCouponInput"
                class="w-full px-3 py-2 border rounded-lg text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
                :class="couponResult ? (couponResult.valid ? 'border-green-400 dark:border-green-500' : 'border-red-300 dark:border-red-500') : 'border-gray-200 dark:border-gray-600'"
              />
              <div v-if="couponLoading" class="absolute right-3 top-1/2 -translate-y-1/2">
                <svg class="w-4 h-4 animate-spin text-gray-400" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>
              </div>
            </div>
            <div v-if="couponResult && couponResult.valid" class="mt-1.5 text-xs text-green-600 dark:text-green-400">
              {{ couponResult.discount_type === 'percentage' ? `${couponResult.discount_value}% ${$t('coupon.off')}` : `¥${couponResult.discount_value?.toFixed(2)} ${$t('coupon.off')}` }}
              · {{ $t('coupon.save') }} ¥{{ couponResult.discount_amount?.toFixed(2) }}
            </div>
            <div v-else-if="couponResult && !couponResult.valid" class="mt-1.5 text-xs text-red-500 dark:text-red-400">
              {{ couponResult.message }}
            </div>
          </div>

          <!-- Total -->
          <div class="bg-gray-50 dark:bg-gray-700 rounded-xl p-4 mb-5">
            <div class="flex justify-between items-center">
              <span class="text-sm text-gray-500 dark:text-gray-400">{{ $t('common.total') }}</span>
              <div class="text-right">
                <span v-if="couponResult?.valid && couponResult.discount_amount" class="text-sm line-through text-gray-400 mr-2">¥{{ subtotalPrice.toFixed(2) }}</span>
                <span class="text-xl font-bold" :class="couponResult?.valid ? 'text-green-600 dark:text-green-400' : 'text-gray-900 dark:text-white'">¥{{ totalPrice.toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <!-- Payment methods -->
          <div class="mb-5">
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-2">{{ $t('product.payment_methods') }}</label>

            <div v-if="availableFiat.length" class="mb-3">
              <div class="text-xs text-gray-400 dark:text-gray-500 mb-1.5">{{ $t('product.fiat') }}</div>
              <div class="flex flex-wrap gap-2">
                <label
                  v-for="m in availableFiat"
                  :key="m.key"
                  :class="[
                    'flex items-center px-3 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                    paymentMethod === m.key
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
                      : 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500'
                  ]"
                >
                  <input type="radio" v-model="paymentMethod" :value="m.key" class="sr-only" />
                  {{ m.label }}
                </label>
              </div>
            </div>

            <div v-if="availableCrypto.length">
              <div class="text-xs text-gray-400 dark:text-gray-500 mb-1.5">{{ $t('product.crypto') }}</div>
              <div class="flex flex-wrap gap-2">
                <label
                  v-for="m in availableCrypto"
                  :key="m.key"
                  :class="[
                    'flex items-center px-3 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                    paymentMethod === m.key
                      ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400'
                      : 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-500'
                  ]"
                >
                  <input type="radio" v-model="paymentMethod" :value="m.key" class="sr-only" />
                  {{ m.label }}
                </label>
              </div>
            </div>
          </div>

          <!-- Error -->
          <p v-if="orderError" class="text-red-500 text-sm mb-3">{{ orderError }}</p>

          <!-- Submit -->
          <button
            @click="submitOrder"
            :disabled="orderLoading"
            class="w-full py-2.5 bg-blue-600 text-white rounded-xl text-sm font-medium hover:bg-blue-700 active:bg-blue-800 disabled:opacity-50 disabled:cursor-not-allowed transition-all flex items-center justify-center gap-2"
          >
            <div v-if="orderLoading" class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
            {{ orderLoading ? $t('common.processing') : $t('product.confirm_purchase') }}
          </button>
        </div>
      </div>
    </Teleport>

    <!-- Payment popup blocked fallback -->
    <div v-if="paymentFallbackUrl" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white dark:bg-gray-800 rounded-xl p-6 max-w-md mx-4 shadow-xl">
        <p class="text-sm text-gray-600 dark:text-gray-300 mb-4">{{ $t('product.popup_blocked_hint') }}</p>
        <a :href="paymentFallbackUrl" target="_blank" class="block w-full text-center px-4 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 mb-3">{{ $t('product.open_payment') }}</a>
        <button @click="navigateToOrder()" class="block w-full text-center px-4 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-200 rounded-lg text-sm hover:bg-gray-50 dark:hover:bg-gray-700">{{ $t('product.continue_to_order') }}</button>
      </div>
    </div>
  </main>
</template>
