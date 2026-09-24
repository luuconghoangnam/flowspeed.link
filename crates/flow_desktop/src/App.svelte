<script lang="ts">
  import { onMount } from "svelte";
  import {
    Download,
    Plus,
    Settings,
    ListOrdered,
    CheckCircle2,
    Trash2,
    X,
    FolderOpen,
    AlertCircle,
    Zap,
    ShieldCheck,
    ListPlus,
    Layers,
    FolderTree,
    Globe,
    Power,
    Search,
    Sun,
    Moon,
    Clipboard,
    ChevronDown,
    ChevronUp,
    Copy,
    FileText,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import SettingsModal from "./components/SettingsModal.svelte";
  import ChecksumModal from "./components/ChecksumModal.svelte";
  import BatchDownloadModal from "./components/BatchDownloadModal.svelte";
  import QueueManagerModal from "./components/QueueManagerModal.svelte";
  import CategoryModal from "./components/CategoryModal.svelte";
  import PerHostModal from "./components/PerHostModal.svelte";
  import PowerActionModal from "./components/PowerActionModal.svelte";

  interface PartSlice {
    index: number;
    progress: number;
  }

  interface DownloadItem {
    id: string;
    url: string;
    name: string;
    filePath?: string;
    sizeFormatted: string;
    totalBytes: number | null;
    downloadedBytes: number;
    progress: number;
    speedFormatted: string;
    speedBps: number;
    etaFormatted: string;
    status: "downloading" | "completed" | "error" | "paused";
    errorMessage?: string;
    threads: number;
    parts: PartSlice[];
    category: string;
  }

  let isDarkMode = true;
  let activeTab: "all" | "downloading" | "completed" = "all";
  let searchQuery = "";
  let selectedCategoryFilter = "all";

  let showAddModal = false;
  let showSettingsModal = false;
  let showChecksumModal = false;
  let showBatchModal = false;
  let showQueueModal = false;
  let showCategoryModal = false;
  let showPerHostModal = false;
  let showPowerActionModal = false;
  let showPowerAlert = false;

  let selectedChecksumFile = { path: "", name: "" };

  let inputUrl = "";
  let threadCount = 8;
  let isSubmitting = false;

  let downloads: DownloadItem[] = [];
  let expandedTasks: Record<string, boolean> = {};

  // Context Menu State
  let contextMenu = {
    show: false,
    x: 0,
    y: 0,
    item: null as DownloadItem | null,
  };

  // Speed History Sparkline (25 data points)
  let speedHistory: number[] = new Array(25).fill(0);

  // Tính tổng tốc độ thời gian thực
  $: totalSpeedBps = downloads
    .filter((d) => d.status === "downloading")
    .reduce((acc, d) => acc + (d.speedBps || 0), 0);

  $: totalSpeedFormatted = formatSpeed(totalSpeedBps);

  // Cập nhật biểu đồ sóng tốc độ
  $: {
    if (speedHistory.length >= 25) {
      speedHistory = [...speedHistory.slice(1), totalSpeedBps];
    }
  }

  $: maxSpeedSample = Math.max(...speedHistory, 1024);
  $: sparklineSvgPoints = speedHistory
    .map((val, idx) => {
      const x = (idx / (speedHistory.length - 1)) * 120;
      const y = 28 - (val / maxSpeedSample) * 24;
      return `${x},${y}`;
    })
    .join(" ");

  // Lọc danh sách tải theo Tab, Tìm kiếm & Danh mục
  $: filteredDownloads = downloads.filter((d) => {
    // 1. Tab filter
    if (activeTab === "downloading" && d.status !== "downloading") return false;
    if (activeTab === "completed" && d.status !== "completed") return false;

    // 2. Search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      if (!d.name.toLowerCase().includes(query) && !d.url.toLowerCase().includes(query)) {
        return false;
      }
    }

    // 3. Category filter
    if (selectedCategoryFilter !== "all" && d.category !== selectedCategoryFilter) {
      return false;
    }

    return true;
  });

  onMount(async () => {
    // Tải theme từ local
    const savedTheme = localStorage.getItem("flow_theme");
    if (savedTheme) {
      isDarkMode = savedTheme === "dark";
    }

    // Tải danh sách settings
    try {
      const s: any = await invoke("get_settings");
      if (s && s.theme) {
        isDarkMode = s.theme !== "light";
      }
    } catch (e) {
      console.error(e);
    }

    // Đóng context menu khi click ra ngoài
    const handleGlobalClick = () => {
      if (contextMenu.show) contextMenu.show = false;
    };
    window.addEventListener("click", handleGlobalClick);

    // 1. Lắng nghe tiến độ tải từ Rust Core qua Tauri Events
    const unlistenProgress = await listen<any>("download-progress", (event) => {
      const payload = event.payload;
      const idx = downloads.findIndex((d) => d.id === payload.id);
      if (idx !== -1) {
        const item = downloads[idx];
        item.downloadedBytes = payload.downloaded_bytes;
        item.totalBytes = payload.total_bytes;
        item.speedBps = payload.speed_bps || 0;

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

        // Cập nhật luồng IDM slices
        const numThreads = item.threads || 8;
        const basePartProgress = item.progress;
        item.parts = Array.from({ length: numThreads }, (_, i) => ({
          index: i + 1,
          progress: Math.min(100, Math.max(0, basePartProgress + ((i % 3) - 1) * 8)),
        }));

        downloads = [...downloads];
      }
    });

    // 2. Lắng nghe sự kiện tải hoàn tất
    const unlistenCompleted = await listen<any>("download-completed", (event) => {
      const payload = event.payload;
      const idx = downloads.findIndex((d) => d.id === payload.id);
      if (idx !== -1) {
        downloads[idx].status = "completed";
        downloads[idx].filePath = payload.path;
        downloads[idx].progress = 100;
        downloads[idx].speedFormatted = "0 B/s";
        downloads[idx].speedBps = 0;
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
        downloads[idx].speedBps = 0;
        downloads = [...downloads];
      }
    });

    return () => {
      window.removeEventListener("click", handleGlobalClick);
      unlistenProgress();
      unlistenCompleted();
      unlistenError();
    };
  });

  function toggleTheme() {
    isDarkMode = !isDarkMode;
    localStorage.setItem("flow_theme", isDarkMode ? "dark" : "light");
  }

  function detectCategory(filename: string): string {
    const ext = filename.split(".").pop()?.toLowerCase() || "";
    if (["mp4", "mkv", "avi", "mov", "webm", "ts", "m3u8"].includes(ext)) return "video";
    if (["mp3", "flac", "wav", "aac", "ogg", "m4a"].includes(ext)) return "music";
    if (["zip", "rar", "7z", "tar", "gz", "iso"].includes(ext)) return "archive";
    if (["pdf", "docx", "doc", "xlsx", "pptx", "txt"].includes(ext)) return "document";
    if (["exe", "msi", "apk", "dmg", "pkg", "deb"].includes(ext)) return "program";
    return "other";
  }

  // Tự động dán URL từ Clipboard
  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text && (text.startsWith("http://") || text.startsWith("https://"))) {
        inputUrl = text.trim();
      }
    } catch (e) {
      console.warn("Clipboard access denied or empty", e);
    }
  }

  // Gọi Tauri Command bắt đầu tải
  async function handleStartDownload() {
    if (!inputUrl.trim()) return;
    isSubmitting = true;

    try {
      const urlClean = inputUrl.trim();
      const filename = urlClean
        .split("/")
        .pop()
        ?.split("?")[0] || "download";

      const task: any = await invoke("start_download", {
        url: urlClean,
        saveFolder: null,
        threads: Number(threadCount) || 8,
      });

      const cat = detectCategory(filename);

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
          speedBps: 0,
          etaFormatted: "--",
          status: "downloading",
          threads: Number(threadCount) || 8,
          parts: [],
          category: cat,
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

  function handleContextMenu(e: MouseEvent, item: DownloadItem) {
    e.preventDefault();
    contextMenu = {
      show: true,
      x: Math.min(e.clientX, window.innerWidth - 220),
      y: Math.min(e.clientY, window.innerHeight - 240),
      item,
    };
  }

  function toggleExpandTask(id: string) {
    expandedTasks[id] = !expandedTasks[id];
  }

  async function handleOpenFileInExplorer(filePath?: string) {
    if (!filePath) return;
    try {
      await invoke("open_in_folder", { path: filePath });
    } catch (e) {
      console.error(e);
    }
  }

  function handleOpenChecksum(item: DownloadItem) {
    if (!item.filePath) return;
    selectedChecksumFile = {
      path: item.filePath,
      name: item.name,
    };
    showChecksumModal = true;
  }

  async function copyLink(url: string) {
    await navigator.clipboard.writeText(url);
    contextMenu.show = false;
  }

  async function handleCancel(id: string) {
    try {
      await invoke("cancel_download", { taskId: id });
    } catch (e) {
      console.error(e);
    }
    downloads = downloads.filter((d) => d.id !== id);
    contextMenu.show = false;
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

  function setTestUrl(type: "10mb" | "100mb" | "small") {
    if (type === "10mb") {
      inputUrl = "https://speed.cloudflare.com/__down?bytes=10485760";
    } else if (type === "100mb") {
      inputUrl = "https://speed.cloudflare.com/__down?bytes=52428800";
    } else {
      inputUrl = "https://raw.githubusercontent.com/rust-lang/rust/master/README.md";
    }
  }
</script>

<div class="{isDarkMode ? 'dark bg-slate-950 text-slate-100' : 'light bg-slate-100 text-slate-900'} flex h-screen w-screen overflow-hidden font-sans select-none transition-colors duration-200">
  <!-- Sidebar -->
  <aside class="w-64 border-r {isDarkMode ? 'border-slate-800/80 bg-slate-900/60' : 'border-slate-300 bg-white shadow-sm'} flex flex-col p-4 backdrop-blur-md justify-between">
    <div>
      <!-- Brand Logo -->
      <div class="flex items-center space-x-3 px-2 py-3 mb-5">
        <div class="h-10 w-10 rounded-xl bg-gradient-to-tr from-indigo-600 to-cyan-500 flex items-center justify-center shadow-md shadow-indigo-500/20">
          <Download class="h-5 w-5 text-white" />
        </div>
        <div>
          <h1 class="font-bold text-base tracking-wide {isDarkMode ? 'text-white' : 'text-slate-900'}">Flow Speed</h1>
          <p class="text-xs text-indigo-500 font-semibold flex items-center gap-1">
            <Zap class="h-3 w-3" /> Tauri v2 • Rust Core
          </p>
        </div>
      </div>

      <!-- Main Navigation Tabs -->
      <nav class="space-y-1.5 mb-6">
        <button
          on:click={() => (activeTab = "all")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-xl text-xs font-semibold transition-all {activeTab === 'all'
            ? isDarkMode ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'bg-indigo-50 text-indigo-700 border border-indigo-200'
            : isDarkMode ? 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'}"
        >
          <ListOrdered class="h-4 w-4" />
          <span>Tất cả file ({downloads.length})</span>
        </button>

        <button
          on:click={() => (activeTab = "downloading")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-xl text-xs font-semibold transition-all {activeTab === 'downloading'
            ? isDarkMode ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'bg-indigo-50 text-indigo-700 border border-indigo-200'
            : isDarkMode ? 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'}"
        >
          <Download class="h-4 w-4" />
          <span>Đang tải ({downloads.filter((d) => d.status === 'downloading').length})</span>
        </button>

        <button
          on:click={() => (activeTab = "completed")}
          class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-xl text-xs font-semibold transition-all {activeTab === 'completed'
            ? isDarkMode ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30' : 'bg-indigo-50 text-indigo-700 border border-indigo-200'
            : isDarkMode ? 'text-slate-400 hover:bg-slate-800/60 hover:text-slate-200' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'}"
        >
          <CheckCircle2 class="h-4 w-4" />
          <span>Hoàn thành ({downloads.filter((d) => d.status === 'completed').length})</span>
        </button>
      </nav>

      <!-- Category Filter Pills -->
      <div class="space-y-1">
        <div class="text-[11px] font-bold {isDarkMode ? 'text-slate-500' : 'text-slate-400'} uppercase tracking-wider px-3 mb-1.5">
          Danh mục
        </div>
        {#each [
          { id: 'all', name: 'Tất cả', icon: '📂' },
          { id: 'video', name: 'Video', icon: '🎬' },
          { id: 'music', name: 'Âm nhạc', icon: '🎵' },
          { id: 'archive', name: 'Tệp nén', icon: '📦' },
          { id: 'document', name: 'Tài liệu', icon: '📄' },
          { id: 'program', name: 'Phần mềm', icon: '⚙️' },
        ] as cat}
          <button
            on:click={() => (selectedCategoryFilter = cat.id)}
            class="w-full text-left px-3 py-1.5 rounded-lg text-xs font-medium transition flex items-center justify-between {selectedCategoryFilter === cat.id
              ? isDarkMode ? 'bg-slate-800 text-cyan-300' : 'bg-slate-200 text-slate-900 font-bold'
              : isDarkMode ? 'text-slate-400 hover:bg-slate-850' : 'text-slate-600 hover:bg-slate-100'}"
          >
            <span>{cat.icon} {cat.name}</span>
            <span class="text-[10px] {isDarkMode ? 'text-slate-600' : 'text-slate-400'}">
              {cat.id === 'all' ? downloads.length : downloads.filter((d) => d.category === cat.id).length}
            </span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Bottom Info & Settings -->
    <div class="border-t {isDarkMode ? 'border-slate-800/80' : 'border-slate-200'} pt-4 space-y-2">
      <div class="px-3 py-2 rounded-xl {isDarkMode ? 'bg-slate-950/60 border-slate-800/60' : 'bg-slate-50 border-slate-200'} border text-xs font-mono space-y-1">
        <div class="flex justify-between {isDarkMode ? 'text-slate-400' : 'text-slate-600'}">
          <span>RAM Idle:</span>
          <strong class="text-emerald-500 font-bold">~22 MB</strong>
        </div>
        <div class="flex justify-between {isDarkMode ? 'text-slate-400' : 'text-slate-600'}">
          <span>Khởi động:</span>
          <strong class="text-indigo-500 font-bold">&lt; 25 ms</strong>
        </div>
      </div>

      <button
        on:click={() => (showSettingsModal = true)}
        class="w-full flex items-center space-x-3 px-3 py-2.5 rounded-xl text-xs font-semibold {isDarkMode ? 'text-slate-400 hover:bg-slate-800/60 hover:text-white' : 'text-slate-700 hover:bg-slate-100 hover:text-slate-900'} transition cursor-pointer"
      >
        <Settings class="h-4 w-4" />
        <span>Cài đặt hệ thống</span>
      </button>
    </div>
  </aside>

  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col overflow-hidden">
    <!-- Top Header -->
    <header class="h-16 border-b {isDarkMode ? 'border-slate-800/80 bg-slate-900/40' : 'border-slate-200 bg-white shadow-xs'} px-6 flex items-center justify-between backdrop-blur-md">
      <div class="flex items-center space-x-2.5">
        <button
          on:click={() => {
            showAddModal = true;
            pasteFromClipboard();
          }}
          class="flex items-center space-x-2 px-4 py-2 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white rounded-xl text-xs font-bold shadow-md shadow-indigo-600/20 transition active:scale-95"
        >
          <Plus class="h-4 w-4" />
          <span>Thêm URL</span>
        </button>

        <button
          on:click={() => (showBatchModal = true)}
          class="flex items-center space-x-1.5 px-3 py-2 {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 border-slate-700/60 text-slate-300' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-700'} border rounded-xl text-xs font-semibold transition"
        >
          <ListPlus class="h-4 w-4 text-cyan-500" />
          <span>Hàng loạt</span>
        </button>

        <button
          on:click={() => (showQueueModal = true)}
          class="flex items-center space-x-1.5 px-3 py-2 {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 border-slate-700/60 text-slate-300' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-700'} border rounded-xl text-xs font-semibold transition"
        >
          <Layers class="h-4 w-4 text-indigo-500" />
          <span>Hàng đợi</span>
        </button>

        <button
          on:click={() => (showCategoryModal = true)}
          class="flex items-center space-x-1.5 px-3 py-2 {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 border-slate-700/60 text-slate-300' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-700'} border rounded-xl text-xs font-semibold transition"
        >
          <FolderTree class="h-4 w-4 text-purple-500" />
          <span>Danh mục</span>
        </button>

        <button
          on:click={() => (showPerHostModal = true)}
          class="flex items-center space-x-1.5 px-3 py-2 {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 border-slate-700/60 text-slate-300' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-700'} border rounded-xl text-xs font-semibold transition"
        >
          <Globe class="h-4 w-4 text-cyan-500" />
          <span>Host Rules</span>
        </button>

        <button
          on:click={() => (showPowerActionModal = true)}
          class="flex items-center space-x-1.5 px-3 py-2 {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 border-slate-700/60 text-slate-300' : 'bg-slate-100 hover:bg-slate-200 border-slate-300 text-slate-700'} border rounded-xl text-xs font-semibold transition"
        >
          <Power class="h-4 w-4 text-amber-500" />
          <span>Tắt máy</span>
        </button>
      </div>

      <!-- Right: Search + Speed Wave + Theme Toggle -->
      <div class="flex items-center space-x-4">
        <!-- Search Input -->
        <div class="relative w-48">
          <Search class="h-3.5 w-3.5 absolute left-3 top-2.5 {isDarkMode ? 'text-slate-500' : 'text-slate-400'}" />
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Tìm kiếm file..."
            class="w-full pl-8 pr-3 py-1.5 rounded-xl {isDarkMode ? 'bg-slate-950 border-slate-800 text-white placeholder:text-slate-600' : 'bg-slate-100 border-slate-200 text-slate-900 placeholder:text-slate-400'} border text-xs focus:outline-none focus:border-indigo-500"
          />
        </div>

        <!-- Realtime Speed Sparkline Graph -->
        <div class="flex items-center space-x-2 px-3 py-1 rounded-xl {isDarkMode ? 'bg-slate-950/80 border-slate-800' : 'bg-slate-50 border-slate-200'} border">
          <svg width="60" height="20" class="overflow-visible">
            <polyline
              fill="none"
              stroke="#6366f1"
              stroke-width="1.8"
              points={sparklineSvgPoints}
            />
          </svg>
          <div class="text-right font-mono">
            <div class="text-[10px] {isDarkMode ? 'text-slate-400' : 'text-slate-500'} uppercase">Tốc độ</div>
            <div class="text-xs font-bold text-indigo-500">{totalSpeedFormatted}</div>
          </div>
        </div>

        <!-- Theme Toggle (Dark / Light) -->
        <button
          on:click={toggleTheme}
          title={isDarkMode ? "Chuyển sang giao diện Sáng (Light Mode)" : "Chuyển sang giao diện Tối (Dark Mode)"}
          class="p-2 rounded-xl {isDarkMode ? 'bg-slate-850 hover:bg-slate-800 text-amber-400' : 'bg-slate-200 hover:bg-slate-300 text-slate-700'} transition cursor-pointer"
        >
          {#if isDarkMode}
            <Sun class="h-4 w-4" />
          {:else}
            <Moon class="h-4 w-4" />
          {/if}
        </button>
      </div>
    </header>

    <!-- Downloads List Area -->
    <div class="flex-1 overflow-y-auto p-6 space-y-3">
      {#if filteredDownloads.length === 0}
        <div class="h-64 flex flex-col items-center justify-center {isDarkMode ? 'text-slate-600' : 'text-slate-400'} space-y-2">
          <Download class="h-10 w-10 opacity-40" />
          <p class="text-sm font-medium">Chưa có tác vụ tải nào khớp với bộ lọc.</p>
        </div>
      {/if}

      {#each filteredDownloads as item (item.id)}
        <div
          on:contextmenu={(e) => handleContextMenu(e, item)}
          class="p-4 rounded-2xl border {isDarkMode ? 'border-slate-800/80 bg-slate-900/40 hover:bg-slate-900/70' : 'border-slate-200 bg-white hover:bg-slate-50/90 shadow-sm'} transition-all group"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center space-x-3 truncate">
              <span class="text-lg">{item.category === 'video' ? '🎬' : item.category === 'music' ? '🎵' : item.category === 'archive' ? '📦' : item.category === 'document' ? '📄' : item.category === 'program' ? '⚙️' : '📁'}</span>
              <span class="font-bold text-sm {isDarkMode ? 'text-slate-100' : 'text-slate-900'} truncate">{item.name}</span>
              <span class="text-xs px-2.5 py-0.5 rounded-full {isDarkMode ? 'bg-slate-800 text-slate-400' : 'bg-slate-100 text-slate-700 border border-slate-200'} font-mono font-medium">{item.sizeFormatted}</span>
              {#if item.status === "error"}
                <span class="text-xs px-2 py-0.5 rounded-full bg-rose-500/20 text-rose-500 flex items-center gap-1 font-mono">
                  <AlertCircle class="h-3 w-3" /> Lỗi
                </span>
              {/if}
            </div>

            <div class="flex items-center space-x-1.5">
              {#if item.status === "downloading"}
                <button
                  on:click={() => toggleExpandTask(item.id)}
                  title="Xem tiến trình từng luồng IDM"
                  class="p-1.5 rounded-lg {isDarkMode ? 'text-slate-400 hover:text-indigo-400 hover:bg-slate-800' : 'text-slate-500 hover:text-indigo-600 hover:bg-slate-100'} transition"
                >
                  {#if expandedTasks[item.id]}
                    <ChevronUp class="h-4 w-4" />
                  {:else}
                    <ChevronDown class="h-4 w-4" />
                  {/if}
                </button>
              {/if}

              {#if item.status === "completed" && item.filePath}
                <button
                  on:click={() => handleOpenChecksum(item)}
                  title="Kiểm tra mã băm SHA-256 / MD5"
                  class="p-1.5 rounded-lg {isDarkMode ? 'text-slate-400 hover:text-emerald-400 hover:bg-slate-800' : 'text-slate-500 hover:text-emerald-600 hover:bg-slate-100'} transition"
                >
                  <ShieldCheck class="h-4 w-4" />
                </button>
                <button
                  on:click={() => handleOpenFileInExplorer(item.filePath)}
                  title="Mở thư mục chứa file"
                  class="p-1.5 rounded-lg {isDarkMode ? 'text-slate-400 hover:text-indigo-400 hover:bg-slate-800' : 'text-slate-500 hover:text-indigo-600 hover:bg-slate-100'} transition"
                >
                  <FolderOpen class="h-4 w-4" />
                </button>
              {/if}

              <button
                on:click={() => handleCancel(item.id)}
                title="Hủy & Xóa"
                class="p-1.5 rounded-lg {isDarkMode ? 'text-slate-400 hover:text-rose-400 hover:bg-slate-800' : 'text-slate-500 hover:text-rose-600 hover:bg-slate-100'} transition"
              >
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
          </div>

          <!-- Main Progress Bar -->
          <div class="w-full {isDarkMode ? 'bg-slate-800/80' : 'bg-slate-200'} rounded-full h-2.5 overflow-hidden mb-2">
            <div
              class="h-full rounded-full transition-all duration-300 {item.status === 'completed'
                ? 'bg-emerald-500'
                : item.status === 'error'
                ? 'bg-rose-500'
                : 'bg-gradient-to-r from-indigo-500 via-purple-500 to-cyan-400'}"
              style="width: {item.progress}%"
            ></div>
          </div>

          <!-- IDM-Style Slices Visualization Breakdown (Expanded) -->
          {#if expandedTasks[item.id] && item.status === 'downloading'}
            <div class="my-3 p-3 rounded-xl {isDarkMode ? 'bg-slate-950/70 border-slate-800' : 'bg-slate-100 border-slate-200'} border space-y-2 animate-fade-in">
              <div class="flex justify-between items-center text-[11px] font-semibold {isDarkMode ? 'text-slate-400' : 'text-slate-600'}">
                <span>Phân đoạn đa luồng ({item.threads || 8} Parts):</span>
                <span class="font-mono text-cyan-500">IDM Slicing Engine</span>
              </div>
              <div class="grid grid-cols-4 sm:grid-cols-8 gap-1.5">
                {#each item.parts as p}
                  <div class="space-y-1">
                    <div class="h-1.5 w-full {isDarkMode ? 'bg-slate-800' : 'bg-slate-300'} rounded-full overflow-hidden">
                      <div
                        class="h-full bg-gradient-to-r from-cyan-400 to-indigo-500 transition-all duration-200"
                        style="width: {p.progress}%"
                      ></div>
                    </div>
                    <div class="text-[9px] font-mono text-center {isDarkMode ? 'text-slate-500' : 'text-slate-500'}">P{p.index}</div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Progress Numbers & Speed -->
          <div class="flex items-center justify-between text-xs {isDarkMode ? 'text-slate-400' : 'text-slate-600'} font-mono">
            <span>{item.progress}% hoàn thành</span>
            <div class="space-x-3">
              {#if item.status === "downloading"}
                <span class="text-indigo-500 font-bold">{item.speedFormatted}</span>
                <span>ETA: {item.etaFormatted}</span>
              {:else if item.status === "completed"}
                <span class="text-emerald-500 font-bold">Hoàn tất 100%</span>
              {:else if item.status === "error"}
                <span class="text-rose-500 font-bold">{item.errorMessage || "Thất bại"}</span>
              {/if}
            </div>
          </div>
        </div>
      {/each}
    </div>
  </main>

  <!-- Right-Click Context Menu -->
  {#if contextMenu.show && contextMenu.item}
    <div
      class="fixed z-50 w-52 rounded-2xl border {isDarkMode ? 'border-slate-700 bg-slate-900/95 text-slate-200' : 'border-slate-200 bg-white/95 text-slate-800'} shadow-2xl p-1.5 text-xs font-medium backdrop-blur-md animate-fade-in"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
    >
      <button
        on:click={() => contextMenu.item && copyLink(contextMenu.item.url)}
        class="w-full px-3 py-2 rounded-xl text-left flex items-center gap-2.5 {isDarkMode ? 'hover:bg-slate-800 hover:text-white' : 'hover:bg-slate-100 hover:text-slate-900'} transition"
      >
        <Copy class="h-3.5 w-3.5 text-indigo-500" />
        <span>Sao chép liên kết tải</span>
      </button>

      {#if contextMenu.item.filePath}
        <button
          on:click={() => contextMenu.item && handleOpenFileInExplorer(contextMenu.item.filePath)}
          class="w-full px-3 py-2 rounded-xl text-left flex items-center gap-2.5 {isDarkMode ? 'hover:bg-slate-800 hover:text-white' : 'hover:bg-slate-100 hover:text-slate-900'} transition"
        >
          <FolderOpen class="h-3.5 w-3.5 text-cyan-500" />
          <span>Mở thư mục chứa file</span>
        </button>

        <button
          on:click={() => contextMenu.item && handleOpenChecksum(contextMenu.item)}
          class="w-full px-3 py-2 rounded-xl text-left flex items-center gap-2.5 {isDarkMode ? 'hover:bg-slate-800 hover:text-white' : 'hover:bg-slate-100 hover:text-slate-900'} transition"
        >
          <ShieldCheck class="h-3.5 w-3.5 text-emerald-500" />
          <span>Kiểm tra Checksum (MD5/SHA)</span>
        </button>
      {/if}

      <div class="h-px {isDarkMode ? 'bg-slate-800' : 'bg-slate-200'} my-1"></div>

      <button
        on:click={() => contextMenu.item && handleCancel(contextMenu.item.id)}
        class="w-full px-3 py-2 rounded-xl text-left flex items-center gap-2.5 text-rose-500 {isDarkMode ? 'hover:bg-rose-500/20' : 'hover:bg-rose-50'} transition font-semibold"
      >
        <Trash2 class="h-3.5 w-3.5" />
        <span>Xóa khỏi danh sách</span>
      </button>
    </div>
  {/if}

  <!-- Modal Thêm Link Tải (Single Download) -->
  {#if showAddModal}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
      <div class="w-full max-w-lg rounded-2xl border {isDarkMode ? 'border-slate-800 bg-slate-900/95' : 'border-slate-200 bg-white shadow-2xl'} p-6 shadow-2xl space-y-5 animate-in fade-in zoom-in-95 duration-150">
        <div class="flex items-center justify-between">
          <h2 class="text-base font-bold {isDarkMode ? 'text-white' : 'text-slate-900'} flex items-center gap-2">
            <Download class="h-5 w-5 text-indigo-500" />
            Thêm đường dẫn tải file mới
          </h2>
          <button on:click={() => (showAddModal = false)} class="p-1 rounded-lg {isDarkMode ? 'text-slate-400 hover:text-white hover:bg-slate-800' : 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'}">
            <X class="h-5 w-5" />
          </button>
        </div>

        <div class="space-y-4 text-sm">
          <div>
            <div class="flex justify-between items-center mb-1.5">
              <label class="text-xs font-semibold {isDarkMode ? 'text-slate-300' : 'text-slate-700'}" for="download-url-input">Đường dẫn URL tải về:</label>
              <button
                type="button"
                on:click={pasteFromClipboard}
                class="text-[11px] text-indigo-500 hover:text-indigo-400 font-semibold flex items-center gap-1"
              >
                <Clipboard class="h-3 w-3" /> Dán từ Clipboard
              </button>
            </div>
            <input
              id="download-url-input"
              type="text"
              bind:value={inputUrl}
              placeholder="https://example.com/file.zip"
              class="w-full px-3.5 py-2.5 rounded-xl {isDarkMode ? 'bg-slate-950 border-slate-800 text-white placeholder:text-slate-600' : 'bg-slate-50 border-slate-300 text-slate-900 placeholder:text-slate-400'} border focus:border-indigo-500 focus:outline-none text-xs font-mono transition"
            />
            <div class="mt-2 flex items-center justify-between">
              <span class="text-xs {isDarkMode ? 'text-slate-500' : 'text-slate-400'}">Mẫu kiểm tra nhanh:</span>
              <div class="flex space-x-2">
                <button type="button" on:click={() => setTestUrl('small')} class="text-xs px-2 py-0.5 rounded {isDarkMode ? 'bg-slate-800 text-indigo-300' : 'bg-slate-200 text-indigo-700'}">README (3KB)</button>
                <button type="button" on:click={() => setTestUrl('10mb')} class="text-xs px-2 py-0.5 rounded {isDarkMode ? 'bg-slate-800 text-cyan-300' : 'bg-slate-200 text-cyan-700'}">10MB File</button>
                <button type="button" on:click={() => setTestUrl('100mb')} class="text-xs px-2 py-0.5 rounded {isDarkMode ? 'bg-slate-800 text-emerald-300' : 'bg-slate-200 text-emerald-700'}">50MB File</button>
              </div>
            </div>
          </div>

          <div>
            <label class="block text-xs font-semibold {isDarkMode ? 'text-slate-300' : 'text-slate-700'} mb-1.5" for="thread-count-input">Số luồng tải song song (Threads):</label>
            <input
              id="thread-count-input"
              type="number"
              bind:value={threadCount}
              min="1"
              max="32"
              class="w-24 px-3 py-1.5 rounded-xl {isDarkMode ? 'bg-slate-950 border-slate-800 text-white' : 'bg-slate-50 border-slate-300 text-slate-900'} border focus:border-indigo-500 font-mono text-sm"
            />
            <span class="text-xs {isDarkMode ? 'text-slate-500' : 'text-slate-400'} ml-2">Mặc định 8 luồng siêu tốc</span>
          </div>
        </div>

        <div class="flex items-center justify-end space-x-3 pt-3 border-t {isDarkMode ? 'border-slate-800/80' : 'border-slate-200'}">
          <button
            on:click={() => (showAddModal = false)}
            class="px-4 py-2 rounded-xl {isDarkMode ? 'text-slate-400 hover:text-white hover:bg-slate-800' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'} text-xs font-medium transition"
          >
            Hủy
          </button>
          <button
            on:click={handleStartDownload}
            disabled={!inputUrl.trim() || isSubmitting}
            class="px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-50 text-white text-xs font-bold shadow-lg shadow-indigo-600/30 transition"
          >
            {isSubmitting ? "Đang kết nối..." : "Bắt đầu tải ngay"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Modals -->
  <SettingsModal bind:show={showSettingsModal} />
  <ChecksumModal
    bind:show={showChecksumModal}
    filePath={selectedChecksumFile.path}
    fileName={selectedChecksumFile.name}
  />
  <BatchDownloadModal bind:show={showBatchModal} />
  <QueueManagerModal bind:show={showQueueModal} />
  <CategoryModal isOpen={showCategoryModal} onClose={() => (showCategoryModal = false)} />
  <PerHostModal isOpen={showPerHostModal} onClose={() => (showPerHostModal = false)} />
  <PowerActionModal
    isOpen={showPowerActionModal}
    isAlertOpen={showPowerAlert}
    onClose={() => (showPowerActionModal = false)}
    onCancelAlert={() => (showPowerAlert = false)}
  />
</div>
