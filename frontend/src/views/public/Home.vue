<script setup lang="ts">
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <header class="py-8 text-center">
      <h1 class="text-2xl font-semibold text-gray-800">AFF Card Shop</h1>
      <p class="mt-1 text-sm text-gray-500">自动发卡商城</p>
      <nav class="mt-4 flex justify-center gap-4 text-sm">
        <router-link to="/" class="text-gray-600 hover:text-gray-900">商品</router-link>
        <router-link to="/order" class="text-gray-600 hover:text-gray-900">查询订单</router-link>
        <router-link to="/aff" class="text-gray-600 hover:text-gray-900">推广返佣</router-link>
      </nav>
    </header>
    <main class="mx-auto max-w-3xl px-4 pb-16">
      <p class="text-center text-gray-400">商品列表将在此显示</p>
    </main>
  </div>
</template>
