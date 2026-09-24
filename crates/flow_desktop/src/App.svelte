<script lang="ts">
  import { onMount } from "svelte";
  import {
    Download,
    Plus,
    Settings,
    ListOrdered,
    CheckCircle2,
    Pause,
    Trash2,
    X,
    FolderOpen,
    AlertCircle,
    Zap,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  interface DownloadItem {
    id: string;
    url: string;
    name: string;
    sizeFormatted: string;
    totalBytes: number | null;
    downloadedBytes: number;
    progress: number;
    speedFormatted: string;
    etaFormatted: string;
    status: "downloading" | "completed" | "error" | "paused";
    errorMessage?: string;
  }

  let activeTab: "all" | "downloading" | "completed" = "all";
  let showAddModal = false;
  let inputUrl = "";
  let threadCount = 8;
  let isSubmitting = false;

  let downloads: DownloadItem[] = [];

  // Tính tổng tốc độ thời gian thực
  $: totalSpeed = downloads
    .filter((d) => d.status === "downloading" && d.downloadedBytes > 0)
    .map((d) => d.speedFormatted)
    .filter(Boolean)
    .join(", ") || "0 B/s";

  // Lọc danh sách theo tab
  $: filteredDownloads = downloads.filter((d) => {
    if (activeTab === "downloading") return d.status === "downloading";
    if (activeTab === "completed") return d.status === "completed";
    return true;
  });

  onMount(async () => {
    // 1. Lắng nghe tiến độ tải từ Rust Core qua Tauri Events
    const unlistenProgress = await listen<any>("download-progress", (event) => {
      const payload = event.payload;
      const idx = downloads.findIndex((d) => d.id === payload.id);
      if (idx !== -1) {
        const item = downloads[idx];
        item.downloadedBytes = payload.downloaded_bytes;
        item.totalBytes = payload.total_bytes;

        if (payload.total_bytes && payload.total_bytes > 0) {
          item.progress = Math.min(
            100,
            Math.round((payload.downloaded_bytes / payload.total_bytes) * 100)
          );
          item.sizeFormatted = formatBytes(payload.total_bytes);
        } else {
          item.progress = 50; // indeterminate
          item.sizeFormatted = formatBytes(payload.downloaded_bytes);
        }

        item.speedFormatted = formatSpeed(payload.speed_bps);
        item.etaFormatted = payload.eta_seconds ? `${payload.eta_seconds}s` : "--";
        downloads = [...downloads];
      }
    });

    // 2. Lắng nghe sự kiện tải hoàn tất
    const unlistenCompleted = await listen<any>("download-completed", (event) => {
      const payload = event.payload;
      const idx = downloads.findIndex((d) => d.id === payload.id);
      if (idx !== -1) {
        downloads[idx].status = "completed";
        downloads[idx].progress = 100;
        downloads[idx].speedFormatted = "0 B/s";
        downloads[idx].etaFormatted = "Hoàn tất";
        if (downloads[idx].totalBytes && downloads[idx].totalBytes > 0) {
          downloads[idx].sizeFormatted = formatBytes(downloads[idx].totalBytes);
        } else if (downloads[idx].downloadedBytes && downloads[idx].downloadedBytes > 0) {
          downloads[idx].sizeFormatted = formatBytes(downloads[idx].downloadedBytes);
        }
        downloads = [...downloads];
      }
    });

    // 3. Lắng nghe lỗi
    const unlistenError = await listen<any>("download-error", (event) => {
      const payload = event.payload;
      const idx = downloads.findIndex((d) => d.id === payload.id);
      if (idx !== -1) {
        downloads[idx].status = "error";
        downloads[idx].errorMessage = payload.error;
        downloads = [...downloads];
      }
    });

    return () => {
      unlistenProgress();
      unlistenCompleted();
      unlistenError();
    };
  });

  // Gọi Tauri Command bắt đầu tải
  async function handleStartDownload() {
    if (!inputUrl.trim()) return;
    isSubmitting = true;

    try {
      const task: any = await invoke("start_download", {
        url: inputUrl.trim(),
        saveFolder: null,
        threads: Number(threadCount) || 8,
      });

      downloads = [
        {
          id: task.id,
          url: task.url,
          name: task.filename,
          sizeFormatted: "Đang kết nối...",
          totalBytes: null,
          downloadedBytes: 0,
          progress: 0,
          speedFormatted: "0 B/s",
          etaFormatted: "--",
          status: "downloading",
        },
        ...downloads,
      ];

      inputUrl = "";
      showAddModal = false;
      activeTab = "downloading";
    } catch (err) {
      alert("Không thể bắt đầu tải: " + err);
    } finally {
      isSubmitting = false;
    }
  }

  // Hủy tải
  async function handleCancel(id: string) {
    try {
      await invoke("cancel_download", { taskId: id });
    } catch (e) {
      console.error(e);
    }
    downloads = downloads.filter((d) => d.id !== id);
  }

  function formatBytes(bytes: number): string {
    if (bytes <= 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function formatSpeed(bps: number): string {
    if (bps <= 0) return "0 B/s";
    return formatBytes(bps) + "/s";
  }

  function setTestUrl(type: '10mb' | '100mb' | 'small') {
    if (type === '10mb') {
      inputUrl = "https://speed.cloudflare.com/__down?bytes=10485760";
    } else if (type === '100mb') {
      inputUrl = "https://speed.cloudflare.com/__down?bytes=52428800";
    } else {
      inputUrl = "https://raw.githubusercontent.com/rust-lang/rust/master/README.md";
    }
  }
</script>

<div class="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-100 font-sans select-none">
  <!-- Sidebar -->
  <aside class="w-64 border-r border-slate-800/80 bg-slate-900/60 flex flex-col p-4 backdrop-blur-md justify-between">
    <div>
      <div class="flex items-center space-x-3 px-2 py-3 mb-6">
        <div class="h-10 w-10 rounded-xl bg-gradient-to-tr from-indigo-500 to-cyan-400 flex items-center justify-center shadow-lg shadow-indigo-500/30">
          <Download class="h-5 w-5 text-white" />
        </div>
        <div>
          <h1 class="font-bold text-base tracking-wide bg-gradient-to-r from-white to-slate-300 bg-clip-text text-transparent">Flow Speed</h1>
          <p class="text-xs text-indigo-400 font-medium flex items-center gap-1">
            <Zap class="h-3 w-3" /> Tauri v2 • Rust Core
          </p>
        </div>
      </div>

      <!-- Navigation tabs -->
      <nav class="space-y-1.5">
        <button
          on:click={() => (activeTab = "all")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'all'
            ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-sm'
            : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <ListOrdered class="h-4 w-4" />
          <span>Tất cả file ({downloads.length})</span>
        </button>

        <button
          on:click={() => (activeTab = "downloading")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'downloading'
            ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-sm'
            : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <Download class="h-4 w-4" />
          <span>Đang tải ({downloads.filter((d) => d.status === 'downloading').length})</span>
        </button>

        <button
          on:click={() => (activeTab = "completed")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all {activeTab === 'completed'
            ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-sm'
            : 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200'}"
        >
          <CheckCircle2 class="h-4 w-4" />
          <span>Hoàn thành ({downloads.filter((d) => d.status === 'completed').length})</span>
        </button>
      </nav>
    </div>

    <!-- Bottom Info -->
    <div class="border-t border-slate-800/80 pt-4 space-y-2">
      <div class="px-3 py-2 rounded-lg bg-slate-950/60 border border-slate-800/60 text-xs font-mono space-y-1">
        <div class="flex justify-between text-slate-400">
          <span>RAM Idle:</span>
          <strong class="text-emerald-400">~24 MB</strong>
        </div>
        <div class="flex justify-between text-slate-400">
          <span>Engine:</span>
          <strong class="text-indigo-400">Zero-Copy Rust</strong>
        </div>
      </div>

      <button class="w-full flex items-center space-x-3 px-3 py-2 rounded-lg text-sm font-medium text-slate-400 hover:bg-slate-800/60 hover:text-slate-200 transition-all">
        <Settings class="h-4 w-4" />
        <span>Cài đặt</span>
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Top Header -->
    <header class="h-16 border-b border-slate-800/80 px-6 flex items-center justify-between bg-slate-900/40 backdrop-blur-md">
      <div class="flex items-center space-x-4">
        <button
          on:click={() => (showAddModal = true)}
          class="flex items-center space-x-2 px-4 py-2 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white rounded-lg text-sm font-semibold shadow-md shadow-indigo-600/30 transition-all active:scale-95"
        >
          <Plus class="h-4 w-4" />
          <span>Thêm đường dẫn URL</span>
        </button>
      </div>

      <div class="flex items-center space-x-6 text-xs font-mono">
        <div class="flex items-center space-x-2">
          <span class="text-slate-400">Tốc độ:</span>
          <strong class="text-indigo-400 font-bold text-sm">{totalSpeed}</strong>
        </div>
      </div>
    </header>

    <!-- Downloads List Area -->
    <div class="flex-1 overflow-y-auto p-6 space-y-3">
      {#if filteredDownloads.length === 0}
        <div class="h-64 flex flex-col items-center justify-center text-slate-500 space-y-2">
          <Download class="h-10 w-10 text-slate-700" />
          <p class="text-sm">Chưa có tác vụ tải nào trong danh mục này.</p>
        </div>
      {/if}

      {#each filteredDownloads as item (item.id)}
        <div class="p-4 rounded-xl border border-slate-800/70 bg-slate-900/40 hover:bg-slate-900/70 transition-all hover:border-slate-700/80 group">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center space-x-3 truncate">
              <span class="font-semibold text-sm text-slate-100 truncate">{item.name}</span>
              <span class="text-xs px-2 py-0.5 rounded-full bg-slate-800 text-slate-400 font-mono">{item.sizeFormatted}</span>
              {#if item.status === "error"}
                <span class="text-xs px-2 py-0.5 rounded-full bg-rose-500/20 text-rose-400 flex items-center gap-1 font-mono">
                  <AlertCircle class="h-3 w-3" /> Lỗi
                </span>
              {/if}
            </div>

            <div class="flex items-center space-x-2">
              <button
                on:click={() => handleCancel(item.id)}
                title="Hủy & Xóa"
                class="p-1.5 rounded-lg text-slate-400 hover:text-rose-400 hover:bg-slate-800 transition-colors"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
          </div>

          <!-- Progress Bar -->
          <div class="w-full bg-slate-800/80 rounded-full h-2.5 overflow-hidden mb-2">
            <div
              class="h-full rounded-full transition-all duration-300 {item.status === 'completed'
                ? 'bg-emerald-500'
                : item.status === 'error'
                ? 'bg-rose-500'
                : 'bg-gradient-to-r from-indigo-500 via-purple-500 to-cyan-400'}"
              style="width: {item.progress}%"
            ></div>
          </div>

          <div class="flex items-center justify-between text-xs text-slate-400 font-mono">
            <span>{item.progress}% hoàn thành</span>
            <div class="space-x-3">
              {#if item.status === "downloading"}
                <span class="text-indigo-400 font-medium">{item.speedFormatted}</span>
                <span>ETA: {item.etaFormatted}</span>
              {:else if item.status === "completed"}
                <span class="text-emerald-400 font-medium">Hoàn tất 100%</span>
              {:else if item.status === "error"}
                <span class="text-rose-400 font-medium">{item.errorMessage || "Thất bại"}</span>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  </main>

  <!-- Modal Thêm Link Tải -->
  {#if showAddModal}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div class="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/95 p-6 shadow-2xl space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-center justify-between">
          <h2 class="text-base font-bold text-white flex items-center gap-2">
            <Download class="h-5 w-5 text-indigo-400" />
            Thêm đường dẫn tải file mới
          </h2>
          <button on:click={() => (showAddModal = false)} class="p-1 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800">
            <X class="h-5 w-5" />
          </button>
        </div>

        <div class="space-y-4 text-sm">
          <div>
            <label class="block text-xs font-semibold text-slate-300 mb-1.5" for="download-url-input">Đường dẫn URL tải về:</label>
            <input
              id="download-url-input"
              type="text"
              bind:value={inputUrl}
              placeholder="https://example.com/file.zip"
              class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 focus:border-indigo-500 focus:outline-none text-white text-sm font-mono placeholder:text-slate-600 transition-all"
            />
            <div class="mt-2 flex items-center justify-between">
              <span class="text-xs text-slate-500">Mẫu kiểm tra nhanh:</span>
              <div class="flex space-x-2">
                <button type="button" on:click={() => setTestUrl('small')} class="text-xs px-2 py-0.5 rounded bg-slate-800 text-indigo-300 hover:bg-slate-700">README (3KB)</button>
                <button type="button" on:click={() => setTestUrl('10mb')} class="text-xs px-2 py-0.5 rounded bg-slate-800 text-cyan-300 hover:bg-slate-700">10MB File</button>
                <button type="button" on:click={() => setTestUrl('100mb')} class="text-xs px-2 py-0.5 rounded bg-slate-800 text-emerald-300 hover:bg-slate-700">100MB File</button>
              </div>
            </div>
          </div>

          <div>
            <label class="block text-xs font-semibold text-slate-300 mb-1.5" for="thread-count-input">Số luồng tải song song (Threads):</label>
            <input
              id="thread-count-input"
              type="number"
              bind:value={threadCount}
              min="1"
              max="32"
              class="w-24 px-3 py-1.5 rounded-lg bg-slate-950 border border-slate-800 focus:border-indigo-500 text-white font-mono text-sm"
            />
            <span class="text-xs text-slate-500 ml-2">Mặc định 8 luồng siêu tốc</span>
          </div>
        </div>

        <div class="flex items-center justify-end space-x-3 pt-3 border-t border-slate-800/80">
          <button
            on:click={() => (showAddModal = false)}
            class="px-4 py-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 text-sm font-medium transition-all"
          >
            Hủy
          </button>
          <button
            on:click={handleStartDownload}
            disabled={!inputUrl.trim() || isSubmitting}
            class="px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-50 text-white text-sm font-semibold shadow-lg shadow-indigo-600/30 transition-all"
          >
            {isSubmitting ? "Đang kết nối..." : "Bắt đầu tải ngay"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
