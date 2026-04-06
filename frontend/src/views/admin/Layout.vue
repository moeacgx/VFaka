<script setup lang="ts">
import { ref } from 'vue'
import { useRoute } from 'vue-router'
import { useAdminStore } from '../../stores/admin'

const route = useRoute()
const admin = useAdminStore()
const sidebarOpen = ref(false)

const navItems = [
  { icon: '📊', label: '仪表盘', to: '/admin/dashboard' },
  { icon: '📦', label: '商品管理', to: '/admin/products' },
  { icon: '🏷️', label: '分类管理', to: '/admin/categories' },
  { icon: '🔑', label: '卡密管理', to: '/admin/cards' },
  { icon: '📋', label: '订单管理', to: '/admin/orders' },
  { icon: '💳', label: '支付通道', to: '/admin/payment' },
  { icon: '👥', label: '推广管理', to: '/admin/aff' },
  { icon: '💰', label: '提现管理', to: '/admin/withdrawals' },
  { icon: '👤', label: '管理员', to: '/admin/admins' },
  { icon: '⚙️', label: '系统设置', to: '/admin/settings' },
]

function isActive(to: string) {
  return route.path === to
}

function handleLogout() {
  admin.logout()
}
</script>

<template>
  <div class="flex min-h-screen bg-gray-100">
    <!-- Mobile overlay -->
    <div v-if="sidebarOpen" class="fixed inset-0 bg-black/50 z-20 lg:hidden" @click="sidebarOpen = false" />

    <!-- Sidebar -->
    <aside
      :class="[
        'fixed lg:static inset-y-0 left-0 z-30 w-56 bg-gray-900 text-white flex flex-col transition-transform lg:translate-x-0',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full'
      ]"
    >
      <div class="px-4 py-5 border-b border-gray-800">
        <h1 class="text-lg font-bold tracking-wide">AFF Admin</h1>
      </div>
      <nav class="flex-1 px-3 py-4 space-y-0.5 overflow-y-auto">
        <router-link
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          :class="[
            'flex items-center gap-2.5 px-3 py-2 rounded-md text-sm transition-colors',
            isActive(item.to)
              ? 'bg-blue-600 text-white'
              : 'text-gray-300 hover:bg-gray-800 hover:text-white'
          ]"
          @click="sidebarOpen = false"
        >
          <span class="text-base leading-none">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </router-link>
      </nav>
      <div class="px-3 py-4 border-t border-gray-800">
        <div class="px-3 py-1.5 text-xs text-gray-400 truncate mb-2">{{ admin.username || '管理员' }}</div>
        <button
          class="flex items-center gap-2.5 w-full px-3 py-2 rounded-md text-sm text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
          @click="handleLogout"
        >
          <span class="text-base leading-none">🚪</span>
          <span>退出登录</span>
        </button>
      </div>
    </aside>

    <!-- Main -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Top bar -->
      <header class="bg-white border-b border-gray-200 px-4 lg:px-6 py-3 flex items-center gap-3 shrink-0">
        <button class="lg:hidden p-1 -ml-1 text-gray-500 hover:text-gray-700" @click="sidebarOpen = !sidebarOpen">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
        </button>
        <h2 class="text-base font-medium text-gray-800">
          {{ navItems.find(i => isActive(i.to))?.label || '管理后台' }}
        </h2>
      </header>

      <!-- Content -->
      <main class="flex-1 p-4 lg:p-6 overflow-auto">
        <router-view />
      </main>
    </div>
  </div>
</template>
