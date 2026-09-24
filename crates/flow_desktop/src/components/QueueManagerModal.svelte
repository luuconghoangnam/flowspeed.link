<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { Layers, Plus, Trash2, Clock, X, Check, Zap } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  const dispatch = createEventDispatcher();

  interface QueueItem {
    id: number;
    name: string;
    max_concurrent: number;
    speed_limit_kbps?: number | null;
    auto_start_time?: string | null;
    auto_stop_time?: string | null;
    is_active: boolean;
  }

  let queues: QueueItem[] = [
    {
      id: 1,
      name: "Hàng đợi Mặc định",
      max_concurrent: 3,
      speed_limit_kbps: null,
      auto_start_time: null,
      auto_stop_time: null,
      is_active: true,
    },
    {
      id: 2,
      name: "Tải Ban Đêm (00:00 - 06:00)",
      max_concurrent: 5,
      speed_limit_kbps: null,
      auto_start_time: "00:00",
      auto_stop_time: "06:00",
      is_active: false,
    },
  ];

  let newQueueName = "";
  let newMaxConcurrent = 3;
  let showCreateForm = false;

  function handleCreateQueue() {
    if (!newQueueName.trim()) return;
    const nextId = Math.max(...queues.map((q) => q.id), 0) + 1;
    queues = [
      ...queues,
      {
        id: nextId,
        name: newQueueName.trim(),
        max_concurrent: Number(newMaxConcurrent) || 3,
        speed_limit_kbps: null,
        auto_start_time: null,
        auto_stop_time: null,
        is_active: true,
      },
    ];
    newQueueName = "";
    showCreateForm = false;
  }

  function handleDeleteQueue(id: number) {
    if (id === 1) {
      alert("Không thể xóa hàng đợi mặc định!");
      return;
    }
    queues = queues.filter((q) => q.id !== id);
  }

  function closeModal() {
    show = false;
    dispatch("close");
  }
</script>

{#if show}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div class="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-900/95 shadow-2xl flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2.5">
          <div class="h-8 w-8 rounded-lg bg-cyan-600/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <Layers class="h-4 w-4" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Quản Lý Hàng Đợi (Queue Manager)</h2>
            <p class="text-xs text-slate-400">Lập lịch tải tự động và phân luồng độc lập</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4 text-sm">
        <!-- Nút thêm hàng đợi mới -->
        {#if !showCreateForm}
          <button
            on:click={() => (showCreateForm = true)}
            class="w-full py-2.5 px-4 rounded-xl border border-dashed border-slate-700 hover:border-cyan-500/80 bg-slate-950/40 hover:bg-slate-950/80 text-xs font-semibold text-slate-300 hover:text-cyan-300 flex items-center justify-center space-x-2 transition-all"
          >
            <Plus class="h-4 w-4" />
            <span>Tạo hàng đợi tải mới</span>
          </button>
        {:else}
          <div class="p-4 rounded-xl bg-slate-950 border border-slate-800 space-y-3 animate-in fade-in">
            <span class="text-xs font-bold text-white block">Tạo hàng đợi mới</span>
            <div class="grid grid-cols-3 gap-3">
              <div class="col-span-2 space-y-1">
                <label class="text-[11px] text-slate-400" for="new-queue-name">Tên hàng đợi:</label>
                <input
                  id="new-queue-name"
                  type="text"
                  bind:value={newQueueName}
                  placeholder="VD: Video HD, Khóa học..."
                  class="w-full px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-700 text-xs text-white focus:outline-none focus:border-cyan-500"
                />
              </div>
              <div class="space-y-1">
                <label class="text-[11px] text-slate-400" for="new-queue-concurrent">Tải đồng thời:</label>
                <input
                  id="new-queue-concurrent"
                  type="number"
                  min="1"
                  max="10"
                  bind:value={newMaxConcurrent}
                  class="w-full px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-700 text-xs text-white text-center font-mono focus:outline-none focus:border-cyan-500"
                />
              </div>
            </div>
            <div class="flex justify-end space-x-2 pt-1">
              <button
                on:click={() => (showCreateForm = false)}
                class="px-3 py-1.5 rounded-lg text-slate-400 hover:text-white text-xs"
              >
                Hủy
              </button>
              <button
                on:click={handleCreateQueue}
                disabled={!newQueueName.trim()}
                class="px-4 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 text-white text-xs font-semibold"
              >
                Tạo
              </button>
            </div>
          </div>
        {/if}

        <!-- Danh sách các hàng đợi -->
        <div class="space-y-2.5">
          {#each queues as q (q.id)}
            <div class="p-4 rounded-xl bg-slate-950/60 border border-slate-800/80 flex items-center justify-between">
              <div class="space-y-1">
                <div class="flex items-center space-x-2">
                  <span class="font-bold text-xs text-white">{q.name}</span>
                  {#if q.id === 1}
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-indigo-500/20 text-indigo-300 font-mono">Mặc định</span>
                  {/if}
                </div>
                <div class="flex items-center space-x-4 text-xs text-slate-400 font-mono">
                  <span>Tải tối đa: <strong class="text-cyan-400">{q.max_concurrent} file</strong></span>
                  {#if q.auto_start_time}
                    <span class="flex items-center gap-1 text-amber-300">
                      <Clock class="h-3 w-3" /> {q.auto_start_time} - {q.auto_stop_time || "Vô hạn"}
                    </span>
                  {/if}
                </div>
              </div>

              <div class="flex items-center space-x-2">
                {#if q.id !== 1}
                  <button
                    on:click={() => handleDeleteQueue(q.id)}
                    title="Xóa hàng đợi"
                    class="p-1.5 rounded-lg text-slate-500 hover:text-rose-400 hover:bg-slate-800 transition-colors"
                  >
                    <Trash2 class="h-4 w-4" />
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/60 flex justify-end">
        <button
          on:click={closeModal}
          class="px-5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition-colors"
        >
          Đóng
        </button>
      </div>
    </div>
  </div>
{/if}
