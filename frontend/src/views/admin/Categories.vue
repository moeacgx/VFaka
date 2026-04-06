<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'

const loading = ref(true)
const categories = ref<any[]>([])
const showForm = ref(false)
const editing = ref<any>(null)
const form = ref({ name: '', sort_order: 0, is_active: true })

async function load() {
  loading.value = true
  try {
    const res = await adminApi.getCategories()
    categories.value = res.data || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function openAdd() {
  editing.value = null
  form.value = { name: '', sort_order: 0, is_active: true }
  showForm.value = true
}

function openEdit(cat: any) {
  editing.value = cat
  form.value = { name: cat.name, sort_order: cat.sort_order || 0, is_active: cat.is_active !== false }
  showForm.value = true
}

async function save() {
  try {
    if (editing.value) {
      await adminApi.updateCategory(editing.value.id, form.value)
    } else {
      await adminApi.createCategory(form.value)
    }
    showForm.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '操作失败')
  }
}

async function remove(id: number) {
  if (!confirm('确定删除该分类？')) return
  try {
    await adminApi.deleteCategory(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '删除失败')
  }
}

async function toggleStatus(cat: any) {
  try {
    await adminApi.updateCategory(cat.id, { ...cat, is_active: !cat.is_active })
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
      <h3 class="text-base font-medium text-gray-800">分类列表</h3>
      <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">
        添加分类
      </button>
    </div>

    <!-- Form modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showForm = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">{{ editing ? '编辑分类' : '添加分类' }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">名称</label>
            <input v-model="form.name" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">排序</label>
            <input v-model.number="form.sort_order" type="number" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div class="flex items-center gap-2">
            <input v-model="form.is_active" type="checkbox" id="cat-active" class="rounded" />
            <label for="cat-active" class="text-sm text-gray-700">启用</label>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">保存</button>
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
            <th class="px-4 py-3 font-medium">排序</th>
            <th class="px-4 py-3 font-medium">状态</th>
            <th class="px-4 py-3 font-medium">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="cat in categories" :key="cat.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 text-gray-600">{{ cat.id }}</td>
            <td class="px-4 py-3 text-gray-800">{{ cat.name }}</td>
            <td class="px-4 py-3 text-gray-600">{{ cat.sort_order }}</td>
            <td class="px-4 py-3">
              <button @click="toggleStatus(cat)" :class="cat.is_active ? 'text-green-600 bg-green-50' : 'text-gray-500 bg-gray-100'" class="px-2 py-0.5 rounded text-xs font-medium">
                {{ cat.is_active ? '启用' : '禁用' }}
              </button>
            </td>
            <td class="px-4 py-3 space-x-2">
              <button @click="openEdit(cat)" class="text-blue-600 hover:text-blue-800 text-xs">编辑</button>
              <button @click="remove(cat.id)" class="text-red-600 hover:text-red-800 text-xs">删除</button>
            </td>
          </tr>
          <tr v-if="categories.length === 0">
            <td colspan="5" class="px-4 py-8 text-center text-gray-400">暂无分类</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
