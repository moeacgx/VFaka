<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { adminApi } from '../../api/admin'
import { useConfirm } from '../../composables/useConfirm'

const { t } = useI18n()
const { confirm } = useConfirm()
const loading = ref(true)
const categories = ref<any[]>([])
const showForm = ref(false)
const editing = ref<any>(null)
const form = ref({ name: '', sort_order: 0, is_active: true })
const selectedIds = ref<number[]>([])

const allSelected = computed({
  get: () => categories.value.length > 0 && selectedIds.value.length === categories.value.length,
  set: (val: boolean) => {
    selectedIds.value = val ? categories.value.map(c => c.id) : []
  }
})

async function load() {
  loading.value = true
  try {
    const res = await adminApi.getCategories()
    categories.value = res.data || []
    selectedIds.value = []
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
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function remove(id: number) {
  if (!await confirm(t('common.confirm_delete'))) return
  try {
    await adminApi.deleteCategory(id)
    await load()
  } catch (e: any) {
    alert(e.response?.data?.error || t('common.operation_failed'))
  }
}

async function batchDelete() {
  if (selectedIds.value.length === 0) return
  if (!await confirm(t('common.confirm_delete') + ` (${selectedIds.value.length})`)) return
  try {
    await adminApi.batchDeleteCategories(selectedIds.value)
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

async function toggleStatus(cat: any) {
  try {
    await adminApi.updateCategory(cat.id, { is_active: !cat.is_active })
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
      <h3 class="text-base font-medium text-gray-800 dark:text-gray-100">{{ $t('category.category_list') }}</h3>
      <div class="flex gap-2">
        <button v-if="selectedIds.length > 0" @click="batchDelete" class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors">
          {{ $t('common.batch_delete') }} ({{ selectedIds.length }})
        </button>
        <button @click="openAdd" class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors">
          {{ $t('category.add_category') }}
        </button>
      </div>
    </div>

    <!-- Form modal -->
    <div v-if="showForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showForm = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-md p-6">
        <h3 class="text-base font-medium text-gray-800 dark:text-gray-100 mb-4">{{ editing ? $t('category.edit_category') : $t('category.add_category') }}</h3>
        <form @submit.prevent="save" class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('category.name') }}</label>
            <input v-model="form.name" required class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-200 mb-1">{{ $t('category.sort_order') }}</label>
            <input v-model.number="form.sort_order" type="number" class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400" />
          </div>
          <div class="flex items-center gap-2">
            <input v-model="form.is_active" type="checkbox" id="cat-active" class="rounded" />
            <label for="cat-active" class="text-sm text-gray-700 dark:text-gray-200">{{ $t('category.active') }}</label>
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
            <th class="px-4 py-3 font-medium">ID</th>
            <th class="px-4 py-3 font-medium">{{ $t('category.name') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('category.sort_order') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.status') }}</th>
            <th class="px-4 py-3 font-medium">{{ $t('common.actions') }}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
          <tr v-for="cat in categories" :key="cat.id" class="hover:bg-gray-50 dark:hover:bg-gray-700">
            <td class="px-4 py-3"><input type="checkbox" :checked="selectedIds.includes(cat.id)" @change="toggleSelect(cat.id)" /></td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ cat.id }}</td>
            <td class="px-4 py-3 text-gray-800 dark:text-gray-100">{{ cat.name }}</td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">{{ cat.sort_order }}</td>
            <td class="px-4 py-3">
              <button @click="toggleStatus(cat)" :class="cat.is_active ? 'text-green-600 bg-green-50' : 'text-gray-500 bg-gray-100'" class="px-2 py-0.5 rounded text-xs font-medium">
                {{ cat.is_active ? $t('common.enabled') : $t('common.disabled') }}
              </button>
            </td>
            <td class="px-4 py-3 space-x-2">
              <button @click="openEdit(cat)" class="text-blue-600 hover:text-blue-800 text-xs">{{ $t('common.edit') }}</button>
              <button @click="remove(cat.id)" class="text-red-600 hover:text-red-800 text-xs">{{ $t('common.delete') }}</button>
            </td>
          </tr>
          <tr v-if="categories.length === 0">
            <td colspan="6" class="px-4 py-8 text-center text-gray-400 dark:text-gray-500">{{ $t('common.no_data') }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
