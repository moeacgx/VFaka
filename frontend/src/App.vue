<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const isAdmin = computed(() => route.path.startsWith('/admin'))
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <header v-if="!isAdmin" class="bg-white border-b border-gray-100">
      <div class="max-w-4xl mx-auto px-4 py-4 flex items-center justify-between">
        <router-link to="/" class="text-xl font-semibold text-gray-900 hover:text-gray-900 no-underline">AFF Card Shop</router-link>
        <nav class="flex gap-4 text-sm text-gray-500">
          <router-link to="/" class="hover:text-gray-900 transition-colors">首页</router-link>
          <router-link to="/order" class="hover:text-gray-900 transition-colors">订单查询</router-link>
          <router-link to="/aff" class="hover:text-gray-900 transition-colors">推广</router-link>
        </nav>
      </div>
    </header>
    <router-view />
  </div>
</template>
