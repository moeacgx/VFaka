<template>
  <div class="flex min-h-screen">
    <aside class="w-56 bg-gray-900 text-white p-4">
      <h2 class="text-lg font-semibold mb-6">AFF Admin</h2>
      <nav class="space-y-1">
        <router-link to="/admin" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">仪表盘</router-link>
        <router-link to="/admin/categories" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">分类管理</router-link>
        <router-link to="/admin/products" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">商品管理</router-link>
        <router-link to="/admin/cards" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">卡密管理</router-link>
        <router-link to="/admin/orders" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">订单管理</router-link>
        <router-link to="/admin/payment" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">支付通道</router-link>
        <router-link to="/admin/aff" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">AFF管理</router-link>
        <router-link to="/admin/withdrawals" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">提现管理</router-link>
        <router-link to="/admin/admins" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">管理员</router-link>
        <router-link to="/admin/settings" class="block px-3 py-2 rounded text-sm hover:bg-gray-700">系统设置</router-link>
      </nav>
    </aside>
    <main class="flex-1 bg-gray-50 p-6">
      <router-view />
    </main>
  </div>
</template>
