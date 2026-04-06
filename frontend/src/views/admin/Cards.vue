<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const cards = ref<any[]>([])
const products = ref<any[]>([])
const filterProduct = ref<number | string>('')
const filterStatus = ref('')
const showImport = ref(false)
const importForm = ref({ product_id: null as number | null, cards: '' })

const statusMap: Record<string, string> = { available: '可用', sold: '已售', locked: '锁定' }

async function load() {
  loading.value = true
  try {
    const params: any = {}
    if (filterProduct.value) params.product_id = filterProduct.value
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

function maskContent(s: string) {
  if (!s) return '-'
  if (s.length <= 4) return '****'
  return s.slice(0, 2) + '****' + s.slice(-2)
}

function getProductName(id: number) {
  return products.value.find(p => p.id === id)?.name || '-'
}

function openImport() {
  importForm.value = { product_id: products.value[0]?.id || null, cards: '' }
  showImport.value = true
}

async function submitImport() {
  if (!importForm.value.product_id || !importForm.value.cards.trim()) {
    alert('请选择商品并输入卡密')
    return
  }
  try {
    const cardsList = importForm.value.cards.split('\n').map(s => s.trim()).filter(Boolean)
    await adminApi.importCards({ product_id: importForm.value.product_id, cards: cardsList.join('\n') })
    showImport.value = false
    await load()
    alert(`成功导入 ${cardsList.length} 张卡密`)
  } catch (e: any) {
    alert(e.response?.data?.error || '导入失败')
  }
}

async function remove(id: number) {
  if (!confirm('确定删除该卡密？')) return
  try {
    await adminApi.deleteCard(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '删除失败')
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
      <h3 class="text-base font-medium text-gray-800">卡密列表</h3>
      <div class="flex flex-wrap items-center gap-2">
        <select v-model="filterProduct" @change="load" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
          <option value="">全部商品</option>
          <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
        <select v-model="filterStatus" @change="load" class="px-3 py-1.5 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
          <option value="">全部状态</option>
          <option value="available">可用</option>
          <option value="sold">已售</option>
          <option value="locked">锁定</option>
        </select>
        <button @click="openImport" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">批量导入</button>
      </div>
    </div>

    <!-- Import modal -->
    <div v-if="showImport" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showImport = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-lg p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">批量导入卡密</h3>
        <form @submit.prevent="submitImport" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">选择商品</label>
            <select v-model="importForm.product_id" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
              <option v-for="p in products" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">卡密内容（每行一个）</label>
            <textarea v-model="importForm.cards" rows="8" placeholder="每行输入一个卡密" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showImport = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">导入</button>
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
            <th class="px-4 py-3 font-medium">商品</th>
            <th class="px-4 py-3 font-medium">卡密内容</th>
            <th class="px-4 py-3 font-medium">状态</th>
            <th class="px-4 py-3 font-medium">创建时间</th>
            <th class="px-4 py-3 font-medium">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="card in cards" :key="card.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 text-gray-600">{{ card.id }}</td>
            <td class="px-4 py-3 text-gray-800">{{ getProductName(card.product_id) }}</td>
            <td class="px-4 py-3 text-gray-600 font-mono text-xs">{{ maskContent(card.content) }}</td>
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
            <td class="px-4 py-3 text-gray-500 text-xs">{{ card.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3">
              <button v-if="card.status === 'available'" @click="remove(card.id)" class="text-red-600 hover:text-red-800 text-xs">删除</button>
            </td>
          </tr>
          <tr v-if="cards.length === 0">
            <td colspan="6" class="px-4 py-8 text-center text-gray-400">暂无卡密</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
