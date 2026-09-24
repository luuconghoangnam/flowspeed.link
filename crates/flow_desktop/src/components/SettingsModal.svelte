<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { Settings, Folder, Cpu, Layers, Bell, Globe, Save, RotateCcw, X, Zap } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  const dispatch = createEventDispatcher();

  interface AppSettings {
    download_dir: string;
    default_threads: number;
    max_concurrent_downloads: number;
    auto_start: boolean;
    server_port: number;
    speed_limit_kbps: number | null;
    notification_enabled: boolean;
  }

  let settings: AppSettings = {
    download_dir: "",
    default_threads: 8,
    max_concurrent_downloads: 3,
    auto_start: false,
    server_port: 15151,
    speed_limit_kbps: null,
    notification_enabled: true,
  };

  let speedLimitInput: string = "";
  let isSaving = false;
  let saveSuccessMessage = false;

  onMount(async () => {
    await loadSettings();
  });

  $: if (show) {
    loadSettings();
  }

  async function loadSettings() {
    try {
      const data: AppSettings = await invoke("get_settings");
      if (data) {
        settings = { ...data };
        speedLimitInput = data.speed_limit_kbps ? String(data.speed_limit_kbps) : "";
      }
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function handleSave() {
    isSaving = true;
    try {
      const limit = parseInt(speedLimitInput);
      settings.speed_limit_kbps = isNaN(limit) || limit <= 0 ? null : limit;

      await invoke("save_settings", { settings });
      saveSuccessMessage = true;
      setTimeout(() => {
        saveSuccessMessage = false;
        closeModal();
      }, 800);
    } catch (e) {
      alert("Lỗi khi lưu cài đặt: " + e);
    } finally {
      isSaving = false;
    }
  }

  async function handleResetDefault() {
    settings = {
      download_dir: settings.download_dir,
      default_threads: 8,
      max_concurrent_downloads: 3,
      auto_start: false,
      server_port: 15151,
      speed_limit_kbps: null,
      notification_enabled: true,
    };
    speedLimitInput = "";
  }

  function closeModal() {
    show = false;
    dispatch("close");
  }
</script>

{#if show}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div class="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-900/95 shadow-2xl flex flex-col max-h-[90vh] overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2.5">
          <div class="h-8 w-8 rounded-lg bg-indigo-600/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
            <Settings class="h-4 w-4" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Cài Đặt Hệ Thống</h2>
            <p class="text-xs text-slate-400">Tùy chỉnh cấu hình tải đa luồng và tích hợp trình duyệt</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="flex-1 overflow-y-auto p-6 space-y-6 text-sm text-slate-200">
        <!-- 1. Thư mục lưu trữ -->
        <div class="space-y-2">
          <label class="text-xs font-semibold text-slate-300 uppercase tracking-wider flex items-center gap-1.5" for="settings-download-dir">
            <Folder class="h-3.5 w-3.5 text-indigo-400" /> Thư mục tải về mặc định
          </label>
          <div class="flex space-x-2">
            <input
              id="settings-download-dir"
              type="text"
              bind:value={settings.download_dir}
              class="flex-1 px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 focus:border-indigo-500 focus:outline-none font-mono text-xs text-slate-300"
            />
          </div>
          <p class="text-xs text-slate-500">Các file tải về sẽ được lưu tự động vào đường dẫn này.</p>
        </div>

        <!-- 2. Hiệu năng tải & Luồng tải -->
        <div class="grid grid-cols-2 gap-4">
          <div class="p-4 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2">
            <label class="text-xs font-semibold text-slate-300 flex items-center justify-between" for="settings-threads">
              <span class="flex items-center gap-1.5"><Cpu class="h-3.5 w-3.5 text-indigo-400" /> Số luồng tải/file:</span>
              <strong class="text-indigo-400 font-mono text-sm">{settings.default_threads}</strong>
            </label>
            <input
              id="settings-threads"
              type="range"
              min="1"
              max="32"
              bind:value={settings.default_threads}
              class="w-full accent-indigo-500 cursor-pointer"
            />
            <div class="flex justify-between text-[10px] text-slate-500 font-mono">
              <span>1 luồng</span>
              <span>8 (Chuẩn)</span>
              <span>32 (Max)</span>
            </div>
          </div>

          <div class="p-4 rounded-xl bg-slate-950/60 border border-slate-800/80 space-y-2">
            <label class="text-xs font-semibold text-slate-300 flex items-center justify-between" for="settings-max-concurrent">
              <span class="flex items-center gap-1.5"><Layers class="h-3.5 w-3.5 text-cyan-400" /> Tải đồng thời:</span>
              <strong class="text-cyan-400 font-mono text-sm">{settings.max_concurrent_downloads} file</strong>
            </label>
            <input
              id="settings-max-concurrent"
              type="range"
              min="1"
              max="10"
              bind:value={settings.max_concurrent_downloads}
              class="w-full accent-cyan-500 cursor-pointer"
            />
            <div class="flex justify-between text-[10px] text-slate-500 font-mono">
              <span>1 file</span>
              <span>3 (Khuyên dùng)</span>
              <span>10 files</span>
            </div>
          </div>
        </div>

        <!-- 3. Giới hạn tốc độ & Cổng Server tích hợp Extension -->
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-300 flex items-center gap-1.5" for="settings-speed-limit">
              <Zap class="h-3.5 w-3.5 text-amber-400" /> Giới hạn tốc độ (KB/s):
            </label>
            <input
              id="settings-speed-limit"
              type="number"
              bind:value={speedLimitInput}
              placeholder="Không giới hạn"
              class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 focus:border-indigo-500 text-xs font-mono text-white placeholder:text-slate-600"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-slate-300 flex items-center gap-1.5" for="settings-server-port">
              <Globe class="h-3.5 w-3.5 text-emerald-400" /> Cổng Extension (REST API):
            </label>
            <input
              id="settings-server-port"
              type="number"
              bind:value={settings.server_port}
              class="w-full px-3.5 py-2 rounded-xl bg-slate-950 border border-slate-800 focus:border-indigo-500 text-xs font-mono text-emerald-400"
            />
          </div>
        </div>

        <!-- 4. Tùy chọn chuyển đổi (Toggles) -->
        <div class="space-y-3 pt-2 border-t border-slate-800/80">
          <label class="flex items-center justify-between p-3 rounded-xl bg-slate-950/40 border border-slate-800/60 cursor-pointer hover:bg-slate-950/80 transition-colors">
            <div class="flex items-center space-x-3">
              <Bell class="h-4 w-4 text-indigo-400" />
              <div>
                <span class="text-xs font-semibold text-slate-200 block">Thông báo khi tải hoàn tất</span>
                <span class="text-[11px] text-slate-500">Hiển thị thông báo trên màn hình Windows khi file tải xong</span>
              </div>
            </div>
            <input
              type="checkbox"
              bind:checked={settings.notification_enabled}
              class="h-4 w-4 rounded bg-slate-900 border-slate-700 text-indigo-600 focus:ring-0 cursor-pointer accent-indigo-500"
            />
          </label>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/60 flex items-center justify-between">
        <button
          on:click={handleResetDefault}
          class="flex items-center space-x-1.5 px-3 py-2 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 text-xs font-medium transition-colors"
        >
          <RotateCcw class="h-3.5 w-3.5" />
          <span>Đặt lại mặc định</span>
        </button>

        <div class="flex items-center space-x-3">
          {#if saveSuccessMessage}
            <span class="text-xs text-emerald-400 font-medium animate-pulse">✓ Đã lưu thành công!</span>
          {/if}
          <button
            on:click={closeModal}
            class="px-4 py-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 text-xs font-medium transition-all"
          >
            Đóng
          </button>
          <button
            on:click={handleSave}
            disabled={isSaving}
            class="flex items-center space-x-1.5 px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-50 text-white text-xs font-semibold shadow-lg shadow-indigo-600/30 transition-all active:scale-95"
          >
            <Save class="h-3.5 w-3.5" />
            <span>{isSaving ? "Đang lưu..." : "Lưu cài đặt"}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
