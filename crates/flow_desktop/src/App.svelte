<script lang="ts">
  import { onMount } from "svelte";
  import { Download, Plus, Settings, ListOrdered, CheckCircle2, Pause, Play, Trash2 } from "lucide-svelte";

  let activeTab = "downloading";
  let downloads = [
    {
      id: "1",
      name: "ubuntu-24.04-desktop-amd64.iso",
      size: "4.8 GB",
      progress: 68,
      speed: "42.5 MB/s",
      eta: "38s",
      status: "downloading"
    },
    {
      id: "2",
      name: "Rust-Book-Edition-2024.pdf",
      size: "15.2 MB",
      progress: 100,
      speed: "0 KB/s",
      eta: "Done",
      status: "completed"
    }
  ];
</script>

<div class="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-100 font-sans">
  <!-- Sidebar -->
  <aside class="w-64 border-r border-slate-800/80 bg-slate-900/50 flex flex-col p-4 backdrop-blur-md justify-between">
    <div>
      <div class="flex items-center space-x-3 px-2 py-3 mb-6">
        <div class="h-9 w-9 rounded-xl bg-gradient-to-tr from-indigo-500 to-cyan-400 flex items-center justify-center shadow-lg shadow-indigo-500/30">
          <Download class="h-5 w-5 text-white" />
        </div>
        <div>
          <h1 class="font-bold text-base tracking-wide bg-gradient-to-r from-white to-slate-300 bg-clip-text text-transparent">Flow Speed</h1>
          <p class="text-xs text-indigo-400 font-medium">Tauri v2 • Native Rust</p>
        </div>
      </div>

      <!-- Navigation tabs -->
      <nav class="space-y-1.5">
        <button
          on:click={() => activeTab = "all"}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'all' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <ListOrdered class="h-4 w-4" />
          <span>Tất cả file</span>
        </button>

        <button
          on:click={() => activeTab = "downloading"}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'downloading' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <Download class="h-4 w-4" />
          <span>Đang tải (1)</span>
        </button>

        <button
          on:click={() => activeTab = "completed"}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'completed' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <CheckCircle2 class="h-4 w-4" />
          <span>Đã hoàn thành (1)</span>
        </button>
      </nav>
    </div>

    <!-- Bottom Settings -->
    <div class="border-t border-slate-800/80 pt-4">
      <button class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium text-slate-400 hover:bg-slate-800/60 hover:text-slate-200 transition-all">
        <Settings class="h-4 w-4" />
        <span>Cài đặt hệ thống</span>
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Top Header -->
    <header class="h-16 border-b border-slate-800/80 px-6 flex items-center justify-between bg-slate-900/30 backdrop-blur-md">
      <div class="flex items-center space-x-4">
        <button class="flex items-center space-x-2 px-4 py-2 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white rounded-lg text-sm font-semibold shadow-md shadow-indigo-600/30 transition-all active:scale-95">
          <Plus class="h-4 w-4" />
          <span>Thêm đường dẫn URL</span>
        </button>
      </div>

      <div class="flex items-center space-x-4 text-xs font-mono">
        <span class="text-slate-400">RAM: <strong class="text-emerald-400">~24 MB</strong></span>
        <span class="text-slate-400">Tổng tốc độ: <strong class="text-indigo-400">42.5 MB/s</strong></span>
      </div>
    </header>

    <!-- Downloads List Area -->
    <div class="flex-1 overflow-y-auto p-6 space-y-3">
      {#each downloads as item}
        <div class="p-4 rounded-xl border border-slate-800/70 bg-slate-900/40 hover:bg-slate-900/70 transition-all hover:border-slate-700/80 group">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center space-x-3 truncate">
              <span class="font-semibold text-sm text-slate-100 truncate">{item.name}</span>
              <span class="text-xs px-2 py-0.5 rounded-full bg-slate-800 text-slate-400 font-mono">{item.size}</span>
            </div>

            <div class="flex items-center space-x-2">
              {#if item.status === "downloading"}
                <button class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
                  <Pause class="h-4 w-4" />
                </button>
              {:else}
                <button class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
                  <Play class="h-4 w-4" />
                </button>
              {/if}
              <button class="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-slate-800 transition-colors">
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="w-full bg-slate-800/80 rounded-full h-2 overflow-hidden mb-2">
            <div
              class="h-full rounded-full transition-all duration-300 {item.status === 'completed' ? 'bg-emerald-500' : 'bg-gradient-to-r from-indigo-500 to-cyan-400'}"
              style="width: {item.progress}%"
            ></div>
          </div>

          <div class="flex items-center justify-between text-xs text-slate-400 font-mono">
            <span>{item.progress}% hoàn thành</span>
            <div class="space-x-3">
              {#if item.status === "downloading"}
                <span class="text-indigo-400 font-medium">{item.speed}</span>
                <span>ETA: {item.eta}</span>
              {:else}
                <span class="text-emerald-400 font-medium">Hoàn tất</span>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  </main>
</div>
