<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'
import { useAdminStore } from '../../stores/admin'
import { useConfirm } from '../../composables/useConfirm'

const { t } = useI18n()
const { confirm } = useConfirm()
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
    alert(t('auth.fill_credentials'))
    return
  }
  try {
    await adminApi.createAdmin(form.value)
    showForm.value = false
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function remove(a: any) {
  if (a.username === admin.username) {
    alert(t('common.current_user'))
    return
  }
  if (!await confirm(t('common.confirm_delete'))) return
  try {
    await adminApi.deleteAdmin(a.id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

onMounted(load)
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('admin.title') }}</h3>
      <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">{{ $t('admin.add_admin') }}</button>
    </div>

    <!-- Add modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" @click.self="showForm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ $t('admin.add_admin') }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('common.username') }}</label>
            <input v-model="form.username" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('common.password') }}</label>
            <input v-model="form.password" type="password" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('admin.role') }}</label>
            <select v-model="form.role" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white">
              <option value="admin">{{ $t('admin.admin_role') }}</option>
              <option value="super_admin">{{ $t('admin.super_admin') }}</option>
            </select>
          </div>
          <div class="flex justify-end gap-2 pt-2">
            <button type="button" @click="showForm = false" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white">{{ $t('common.cancel') }}</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700">{{ $t('common.confirm') }}</button>
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
            <th class="px-4 py-3 font-medium">{{ $t('common.username') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('admin.role') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.created_at') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="a in admins" :key="a.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ a.id }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ a.username }}</td>
            <td class="px-4 py-3">
              <span :class="a.role === 'super_admin' ? 'text-purple-600 bg-purple-50' : 'text-blue-600 bg-blue-50'" class="inline-block px-2 py-0.5 rounded text-xs font-medium">
                {{ a.role === 'super_admin' ? $t('admin.super_admin') : $t('admin.admin_role') }}
              </span>
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 text-xs">{{ a.created_at?.replace('T', ' ').slice(0, 19) }}</td>
            <td class="px-4 py-3">
              <button v-if="a.username !== admin.username" @click="remove(a)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('common.delete') }}</button>
              <span v-else class="text-gray-400 dark:text-gray-500 text-xs">{{ $t('common.current_user') }}</span>
            </td>
          </tr>
          <tr v-if="admins.length === 0">
            <td colspan="5" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
