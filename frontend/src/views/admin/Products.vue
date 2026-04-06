<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const products = ref<any[]>([])
const categories = ref<any[]>([])
const showForm = ref(false)
const showRestock = ref(false)
const editing = ref<any>(null)

const defaultForm = () => ({
  name: '', category_id: null as number | null, price: 0, description: '',
  pay_alipay: true, pay_wechat: true, pay_qqpay: false,
  pay_usdt_trc20: false, pay_usdt_erc20: false, pay_usdt_polygon: false,
  post_action_type: 'none', post_action_value: '',
  aff_commission_rate: null as number | null,
  sort_order: 0, min_quantity: 1, max_quantity: 10, is_active: true,
})
const form = ref(defaultForm())
const restockForm = ref({ product_id: null as number | null, cards: '' })

async function load() {
  loading.value = true
  try {
    const [pRes, cRes] = await Promise.all([adminApi.getProducts(), adminApi.getCategories()])
    products.value = pRes.data || []
    categories.value = cRes.data || []
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
    pay_alipay: !!p.pay_alipay, pay_wechat: !!p.pay_wechat, pay_qqpay: !!p.pay_qqpay,
    pay_usdt_trc20: !!p.pay_usdt_trc20, pay_usdt_erc20: !!p.pay_usdt_erc20, pay_usdt_polygon: !!p.pay_usdt_polygon,
    post_action_type: p.post_action_type || 'none', post_action_value: p.post_action_value || '',
    aff_commission_rate: p.aff_commission_rate ?? null,
    sort_order: p.sort_order || 0, min_quantity: p.min_quantity || 1, max_quantity: p.max_quantity || 10, is_active: p.is_active !== false,
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
    alert(e.response?.data?.error || '操作失败')
  }
}

async function remove(id: number) {
  if (!confirm('确定删除该商品？')) return
  try {
    await adminApi.deleteProduct(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '删除失败')
  }
}

function openRestock() {
  restockForm.value = { product_id: products.value[0]?.id || null, cards: '' }
  showRestock.value = true
}

async function submitRestock() {
  if (!restockForm.value.product_id || !restockForm.value.cards.trim()) {
    alert('请选择商品并输入卡密')
    return
  }
  try {
    const cards = restockForm.value.cards.split('\n').map(s => s.trim()).filter(Boolean)
    await adminApi.restockProduct(restockForm.value.product_id, { cards: cards.join('\n') })
    showRestock.value = false
    await load()
    alert(`成功导入 ${cards.length} 张卡密`)
  } catch (e: any) {
    alert(e.response?.data?.error || '补货失败')
  }
}

function getCategoryName(id: number) {
  return categories.value.find(c => c.id === id)?.name || '-'
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800">商品列表</h3>
      <div class="flex gap-2">
        <button @click="openRestock" class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 transition-colors">补货</button>
        <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">添加商品</button>
      </div>
    </div>

    <!-- Product form modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showForm = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-2xl max-h-[90vh] overflow-y-auto p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">{{ editing ? '编辑商品' : '添加商品' }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">名称</label>
              <input v-model="form.name" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">分类</label>
              <select v-model="form.category_id" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
                <option v-for="c in categories" :key="c.id" :value="c.id">{{ c.name }}</option>
              </select>
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">价格 (¥)</label>
              <input v-model.number="form.price" type="number" step="0.01" min="0" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">排序</label>
              <input v-model.number="form.sort_order" type="number" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">最小购买数</label>
              <input v-model.number="form.min_quantity" type="number" min="1" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">最大购买数</label>
              <input v-model.number="form.max_quantity" type="number" min="1" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">描述</label>
            <textarea v-model="form.description" rows="3" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">支付方式</label>
            <div class="flex flex-wrap gap-4">
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_alipay" type="checkbox" /> 支付宝</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_wechat" type="checkbox" /> 微信</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_qqpay" type="checkbox" /> QQ钱包</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_usdt_trc20" type="checkbox" /> USDT-TRC20</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_usdt_erc20" type="checkbox" /> USDT-ERC20</label>
              <label class="flex items-center gap-1.5 text-sm text-gray-700"><input v-model="form.pay_usdt_polygon" type="checkbox" /> USDT-Polygon</label>
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">售后动作类型</label>
              <select v-model="form.post_action_type" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
                <option value="none">无</option>
                <option value="webhook">Webhook</option>
                <option value="command">命令</option>
              </select>
            </div>
            <div v-if="form.post_action_type !== 'none'">
              <label class="block text-sm font-medium text-gray-700 mb-1">动作值</label>
              <input v-model="form.post_action_value" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">推广佣金率覆盖 (%)</label>
              <input v-model.number="form.aff_commission_rate" type="number" step="0.1" min="0" max="100" placeholder="留空使用全局设置" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
            </div>
            <div class="flex items-end">
              <label class="flex items-center gap-2 text-sm text-gray-700 pb-2"><input v-model="form.is_active" type="checkbox" /> 启用商品</label>
            </div>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">保存</button>
          </div>
        </form>
      </div>
    </div>

    <!-- Restock modal -->
    <div v-if="showRestock" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showRestock = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-lg p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">补货 - 批量导入卡密</h3>
        <form @submit.prevent="submitRestock" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">选择商品</label>
            <select v-model="restockForm.product_id" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
              <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">卡密内容（每行一个）</label>
            <textarea v-model="restockForm.cards" rows="8" placeholder="每行输入一个卡密" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showRestock = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-green-600 text-white text-sm rounded-md hover:bg-green-700">导入</button>
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
            <th class="px-4 py-3 font-medium">名称</th>
            <th class="px-4 py-3 font-medium">分类</th>
            <th class="px-4 py-3 font-medium">价格</th>
            <th class="px-4 py-3 font-medium">库存</th>
            <th class="px-4 py-3 font-medium">已售</th>
            <th class="px-4 py-3 font-medium">状态</th>
            <th class="px-4 py-3 font-medium">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="p in products" :key="p.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 text-gray-600">{{ p.id }}</td>
            <td class="px-4 py-3 text-gray-800">{{ p.name }}</td>
            <td class="px-4 py-3 text-gray-600">{{ getCategoryName(p.category_id) }}</td>
            <td class="px-4 py-3 text-gray-800">¥{{ p.price?.toFixed(2) }}</td>
            <td class="px-4 py-3" :class="(p.stock ?? 0) < 5 ? 'text-red-600 font-medium' : 'text-gray-600'">{{ p.stock ?? 0 }}</td>
            <td class="px-4 py-3 text-gray-600">{{ p.sold ?? 0 }}</td>
            <td class="px-4 py-3">
              <span :class="p.is_active ? 'text-green-600 bg-green-50' : 'text-gray-500 bg-gray-100'" class="inline-block px-2 py-0.5 rounded text-xs font-medium">
                {{ p.is_active ? '上架' : '下架' }}
              </span>
            </td>
            <td class="px-4 py-3 space-x-2">
              <button @click="openEdit(p)" class="text-blue-600 hover:text-blue-800 text-xs">编辑</button>
              <button @click="remove(p.id)" class="text-red-600 hover:text-red-800 text-xs">删除</button>
            </td>
          </tr>
          <tr v-if="products.length === 0">
            <td colspan="8" class="px-4 py-8 text-center text-gray-400">暂无商品</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
