<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { publicApi } from '../../api/public'
import { useAffCode } from '../../composables/useAffCode'

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
const orderEmail = ref('')
const orderQuantity = ref(1)
const paymentMethod = ref('')
const orderLoading = ref(false)
const orderError = ref('')

const filteredProducts = computed(() => {
  if (activeCategory.value === null) return products.value
  return products.value.filter(p => p.category_id === activeCategory.value)
})

const availableFiat = computed(() => {
  if (!selectedProduct.value) return []
  const p = selectedProduct.value
  const methods: { key: string; label: string }[] = []
  if (p.allow_alipay) methods.push({ key: 'alipay', label: '支付宝' })
  if (p.allow_wxpay) methods.push({ key: 'wxpay', label: '微信' })
  if (p.allow_qqpay) methods.push({ key: 'qqpay', label: 'QQ钱包' })
  return methods
})

const availableCrypto = computed(() => {
  if (!selectedProduct.value) return []
  const p = selectedProduct.value
  const methods: { key: string; label: string }[] = []
  if (p.allow_usdt_trc20) methods.push({ key: 'usdt_trc20', label: 'USDT(TRC20)' })
  if (p.allow_trx) methods.push({ key: 'trx', label: 'TRX' })
  if (p.allow_usdt_erc20) methods.push({ key: 'usdt_erc20', label: 'USDT(ERC20)' })
  return methods
})

const totalPrice = computed(() => {
  if (!selectedProduct.value) return 0
  return selectedProduct.value.price * orderQuantity.value
})

function truncate(text: string | null, len: number) {
  if (!text) return ''
  return text.length > len ? text.slice(0, len) + '...' : text
}

function openBuyModal(product: Product) {
  selectedProduct.value = product
  orderEmail.value = ''
  orderQuantity.value = product.min_quantity || 1
  paymentMethod.value = ''
  orderError.value = ''
  orderLoading.value = false
  showModal.value = true
}

function closeModal() {
  showModal.value = false
  selectedProduct.value = null
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
    orderError.value = '请输入邮箱地址'
    return
  }
  if (!paymentMethod.value) {
    orderError.value = '请选择支付方式'
    return
  }

  orderLoading.value = true
  orderError.value = ''

  try {
    const affCode = getAffCode()
    const res = await publicApi.createOrder({
      product_id: selectedProduct.value.id,
      quantity: orderQuantity.value,
      email: orderEmail.value.trim(),
      payment_method: paymentMethod.value,
      aff_code: affCode || undefined,
    })
    const data = res.data
    closeModal()
    if (data.payment_url) {
      window.location.href = data.payment_url
    } else {
      router.push({ path: '/order', query: { no: data.order_no, email: orderEmail.value.trim() } })
    }
  } catch (e: any) {
    orderError.value = e.response?.data?.message || e.response?.data?.error || '创建订单失败，请重试'
  } finally {
    orderLoading.value = false
  }
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
    error.value = '加载商品失败，请刷新重试'
  } finally {
    loading.value = false
  }
  // Save aff code from URL if present
  getAffCode()
})
</script>

<template>
  <main class="max-w-4xl mx-auto px-4 py-8 pb-16">
    <!-- Loading -->
    <div v-if="loading" class="flex justify-center py-20">
      <div class="w-6 h-6 border-2 border-gray-300 border-t-blue-600 rounded-full animate-spin"></div>
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
              ? 'bg-gray-900 text-white'
              : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'
          ]"
        >全部</button>
        <button
          v-for="cat in categories"
          :key="cat.id"
          @click="activeCategory = cat.id"
          :class="[
            'px-4 py-1.5 rounded-full text-sm font-medium transition-all',
            activeCategory === cat.id
              ? 'bg-gray-900 text-white'
              : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'
          ]"
        >{{ cat.name }}</button>
      </div>

      <!-- Empty state -->
      <div v-if="filteredProducts.length === 0" class="text-center py-20">
        <p class="text-gray-400 text-sm">暂无商品</p>
      </div>

      <!-- Product grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="product in filteredProducts"
          :key="product.id"
          class="bg-white rounded-xl border border-gray-100 shadow-sm hover:shadow-md transition-shadow p-5 flex flex-col"
        >
          <h3 class="text-base font-semibold text-gray-900 mb-1">{{ product.name }}</h3>
          <p class="text-xs text-gray-400 mb-3 flex-1 leading-relaxed">{{ truncate(product.description, 60) }}</p>
          <div class="flex items-end justify-between mt-auto">
            <div>
              <span class="text-lg font-bold text-gray-900">¥{{ product.price.toFixed(2) }}</span>
              <span class="text-xs text-gray-400 ml-2">库存 {{ product.stock_count }}</span>
            </div>
            <button
              @click="openBuyModal(product)"
              :disabled="product.stock_count <= 0"
              :class="[
                'px-4 py-1.5 rounded-lg text-sm font-medium transition-all',
                product.stock_count > 0
                  ? 'bg-blue-600 text-white hover:bg-blue-700 active:bg-blue-800'
                  : 'bg-gray-100 text-gray-400 cursor-not-allowed'
              ]"
            >{{ product.stock_count > 0 ? '购买' : '售罄' }}</button>
          </div>
        </div>
      </div>
    </template>

    <!-- Purchase Modal -->
    <Teleport to="body">
      <div v-if="showModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div class="fixed inset-0 bg-black/30 backdrop-blur-sm" @click="closeModal"></div>
        <div class="relative bg-white rounded-2xl shadow-xl w-full max-w-md p-6 max-h-[90vh] overflow-y-auto">
          <!-- Header -->
          <div class="flex items-center justify-between mb-5">
            <h3 class="text-lg font-semibold text-gray-900">{{ selectedProduct?.name }}</h3>
            <button @click="closeModal" class="text-gray-400 hover:text-gray-600 transition-colors p-1">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- Price -->
          <div class="bg-gray-50 rounded-xl p-4 mb-5">
            <div class="text-sm text-gray-500">单价</div>
            <div class="text-2xl font-bold text-gray-900">¥{{ selectedProduct?.price.toFixed(2) }}</div>
          </div>

          <!-- Email -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1.5">邮箱地址</label>
            <input
              v-model="orderEmail"
              type="email"
              placeholder="用于接收订单信息"
              class="w-full px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all"
            />
          </div>

          <!-- Quantity -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-gray-700 mb-1.5">数量</label>
            <div class="flex items-center gap-3">
              <button
                @click="adjustQuantity(-1)"
                :disabled="orderQuantity <= (selectedProduct?.min_quantity || 1)"
                class="w-8 h-8 flex items-center justify-center rounded-lg border border-gray-200 text-gray-600 hover:bg-gray-50 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
              >−</button>
              <span class="text-lg font-semibold text-gray-900 w-8 text-center">{{ orderQuantity }}</span>
              <button
                @click="adjustQuantity(1)"
                :disabled="orderQuantity >= (selectedProduct?.max_quantity || 1)"
                class="w-8 h-8 flex items-center justify-center rounded-lg border border-gray-200 text-gray-600 hover:bg-gray-50 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
              >+</button>
              <span class="text-xs text-gray-400">（{{ selectedProduct?.min_quantity }} - {{ selectedProduct?.max_quantity }}）</span>
            </div>
          </div>

          <!-- Total -->
          <div class="bg-gray-50 rounded-xl p-4 mb-5">
            <div class="flex justify-between items-center">
              <span class="text-sm text-gray-500">合计</span>
              <span class="text-xl font-bold text-gray-900">¥{{ totalPrice.toFixed(2) }}</span>
            </div>
          </div>

          <!-- Payment methods -->
          <div class="mb-5">
            <label class="block text-sm font-medium text-gray-700 mb-2">支付方式</label>

            <div v-if="availableFiat.length" class="mb-3">
              <div class="text-xs text-gray-400 mb-1.5">法币支付</div>
              <div class="flex flex-wrap gap-2">
                <label
                  v-for="m in availableFiat"
                  :key="m.key"
                  :class="[
                    'flex items-center px-3 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                    paymentMethod === m.key
                      ? 'border-blue-500 bg-blue-50 text-blue-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'
                  ]"
                >
                  <input type="radio" v-model="paymentMethod" :value="m.key" class="sr-only" />
                  {{ m.label }}
                </label>
              </div>
            </div>

            <div v-if="availableCrypto.length">
              <div class="text-xs text-gray-400 mb-1.5">加密货币</div>
              <div class="flex flex-wrap gap-2">
                <label
                  v-for="m in availableCrypto"
                  :key="m.key"
                  :class="[
                    'flex items-center px-3 py-2 rounded-lg border text-sm cursor-pointer transition-all',
                    paymentMethod === m.key
                      ? 'border-blue-500 bg-blue-50 text-blue-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'
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
            {{ orderLoading ? '处理中...' : '确认购买' }}
          </button>
        </div>
      </div>
    </Teleport>
  </main>
</template>
