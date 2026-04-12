<script setup lang="ts">
import { useConfirm } from '../../composables/useConfirm'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const { visible, message, handleConfirm, handleCancel } = useConfirm()
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]" @click.self="handleCancel">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg w-full max-w-sm mx-4 p-6">
        <p class="text-sm text-gray-700 dark:text-gray-200 mb-6 whitespace-pre-wrap">{{ message }}</p>
        <div class="flex justify-end gap-3">
          <button @click="handleCancel" class="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-white rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
            {{ t('common.cancel') }}
          </button>
          <button @click="handleConfirm" class="px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors">
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
