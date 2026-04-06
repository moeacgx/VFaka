<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { adminApi } from '../../api/admin'
import { useAdminStore } from '../../stores/admin'

const admin = useAdminStore()
const loading = ref(true)
const admins = ref<any[]>([])
const showForm = ref(false)
const form = ref({ username: '', password: '', role: 'admin' })

async function load() {
  loading.value = true
  try {
    const res = await adminApi.getAdmins()
    admins.value = res.data || []
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

function openAdd() {
  form.value = { username: '', password: '', role: 'admin' }
  showForm.value = true
}

async function save() {
  if (!form.value.username || !form.value.password) {
    alert('请填写用户名和密码')
    return
  }
  try {
    await adminApi.createAdmin(form.value)
    showForm.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '创建失败')
  }
}

async function remove(a: any) {
  if (a.username === admin.username) {
    alert('不能删除当前登录的管理员')
    return
  }
  if (!confirm(`确定删除管理员 "${a.username}"？`)) return
  try {
    await adminApi.deleteAdmin(a.id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || '删除失败')
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800">管理员列表</h3>
      <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">添加管理员</button>
    </div>

    <!-- Add modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showForm = false">
      <div class="bg-white rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 mb-4">添加管理员</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">用户名</label>
            <input v-model="form.username" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">密码</label>
            <input v-model="form.password" type="password" required class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">角色</label>
            <select v-model="form.role" class="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500">
              <option value="admin">管理员</option>
              <option value="super_admin">超级管理员</option>
            </select>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 hover:text-gray-800">取消</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">创建</button>
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
            <th class="px-4 py-3 font-medium">用户名</th>
            <th class="px-4 py-3 font-medium">角色</th>
            <th class="px-4 py-3 font-medium">创建时间</th>
            <th class="px-4 py-3 font-medium">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="a in admins" :key="a.id" class="hover:bg-gray-50">
            <td class="px-4 py-3 text-gray-600">{{ a.id }}</td>
            <td class="px-4 py-3 text-gray-800">{{ a.username }}</td>
            <td class="px-4 py-3">
              <span :class="a.role === 'super_admin' ? 'text-purple-600 bg-purple-50' : 'text-blue-600 bg-blue-50'" class="inline-block px-2 py-0.5 rounded text-xs font-medium">
                {{ a.role === 'super_admin' ? '超级管理员' : '管理员' }}
              </span>
            </td>
            <td class="px-4 py-3 text-gray-500 text-xs">{{ a.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3">
              <button v-if="a.username !== admin.username" @click="remove(a)" class="text-red-600 hover:text-red-800 text-xs">删除</button>
              <span v-else class="text-gray-400 text-xs">当前用户</span>
            </td>
          </tr>
          <tr v-if="admins.length === 0">
            <td colspan="5" class="px-4 py-8 text-center text-gray-400">暂无管理员</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
