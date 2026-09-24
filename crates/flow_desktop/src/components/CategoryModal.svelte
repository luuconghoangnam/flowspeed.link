<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface FileCategory {
    id: string;
    name: string;
    icon: string;
    custom_folder?: string | null;
    file_extensions: string[];
    url_patterns: string[];
  }

  interface AppSettings {
    download_dir: string;
    default_threads: number;
    max_concurrent_downloads: number;
    auto_start: boolean;
    server_port: number;
    speed_limit_kbps?: number | null;
    notification_enabled: boolean;
    categories: FileCategory[];
    per_host_rules: any[];
    power_action?: any;
  }

  export let isOpen = false;
  export let onClose = () => {};

  let settings: AppSettings | null = null;
  let categories: FileCategory[] = [];
  let selectedCategory: FileCategory | null = null;
  let newExtension = "";
  let isSaving = false;
  let saveSuccess = false;

  async function loadSettings() {
    try {
      const res = await invoke<AppSettings>("get_settings");
      settings = res;
      categories = JSON.parse(JSON.stringify(res.categories || []));
      if (categories.length > 0) {
        selectedCategory = categories[0];
      }
    } catch (e) {
      console.error("Failed to load categories:", e);
    }
  }

  $: if (isOpen) {
    loadSettings();
  }

  function addExtension() {
    if (!newExtension.trim() || !selectedCategory) return;
    const cleanExt = newExtension.trim().replace(/^\./, "").toLowerCase();
    if (!selectedCategory.file_extensions.includes(cleanExt)) {
      selectedCategory.file_extensions = [...selectedCategory.file_extensions, cleanExt];
      // update list
      categories = [...categories];
    }
    newExtension = "";
  }

  function removeExtension(ext: string) {
    if (!selectedCategory) return;
    selectedCategory.file_extensions = selectedCategory.file_extensions.filter((e) => e !== ext);
    categories = [...categories];
  }

  async function saveCategories() {
    if (!settings) return;
    isSaving = true;
    saveSuccess = false;
    try {
      settings.categories = categories;
      await invoke("save_settings", { settings });
      saveSuccess = true;
      setTimeout(() => {
        saveSuccess = false;
        onClose();
      }, 800);
    } catch (e) {
      console.error("Failed to save categories:", e);
    } finally {
      isSaving = false;
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md p-4 animate-fade-in">
    <div class="bg-slate-900 border border-slate-700/80 rounded-2xl w-full max-w-3xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-purple-500/20 text-purple-400 flex items-center justify-center text-lg border border-purple-500/30">
            📁
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Phân Loại Tệp Tin Tự Động (File Categories)</h2>
            <p class="text-xs text-slate-400">Tự động định tuyến thư mục lưu theo đuôi tệp tin</p>
          </div>
        </div>
        <button on:click={onClose} class="text-slate-400 hover:text-white p-2 rounded-lg hover:bg-slate-800 transition">
          ✕
        </button>
      </div>

      <!-- Body: 2 Columns -->
      <div class="flex-1 flex overflow-hidden">
        <!-- Categories List -->
        <div class="w-1/3 border-r border-slate-800 p-3 space-y-1.5 overflow-y-auto bg-slate-950/30">
          <div class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider px-2 py-1">Danh mục</div>
          {#each categories as cat}
            <button
              on:click={() => (selectedCategory = cat)}
              class="w-full text-left px-3 py-2.5 rounded-xl text-xs font-medium transition flex items-center justify-between {selectedCategory?.id === cat.id ? 'bg-purple-600/20 text-purple-300 border border-purple-500/40' : 'text-slate-300 hover:bg-slate-800/60'}"
            >
              <div class="flex items-center gap-2 truncate">
                <span>{cat.icon === 'video' ? '🎬' : cat.icon === 'music' ? '🎵' : cat.icon === 'archive' ? '📦' : cat.icon === 'document' ? '📄' : '⚙️'}</span>
                <span class="truncate">{cat.name}</span>
              </div>
              <span class="text-[10px] bg-slate-800 px-1.5 py-0.5 rounded text-slate-400">
                {cat.file_extensions.length}
              </span>
            </button>
          {/each}
        </div>

        <!-- Detail / Settings for Selected Category -->
        <div class="flex-1 p-6 overflow-y-auto space-y-5 bg-slate-900">
          {#if selectedCategory}
            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1">Tên Danh Mục</label>
              <input
                type="text"
                bind:value={selectedCategory.name}
                class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white focus:outline-none focus:border-purple-500"
              />
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1">Thư Mục Lưu Riêng (Tùy chọn)</label>
              <input
                type="text"
                bind:value={selectedCategory.custom_folder}
                placeholder="Để trống để dùng thư mục tải mặc định"
                class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white font-mono text-xs focus:outline-none focus:border-purple-500"
              />
              <p class="text-[11px] text-slate-500 mt-1">Ví dụ: C:\Users\Username\Downloads\Videos</p>
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-2">Định Dạng Đuôi Tệp Tin Nhận Diện ({selectedCategory.file_extensions.length})</label>
              <div class="flex flex-wrap gap-1.5 mb-3 p-3 bg-slate-950/60 border border-slate-800 rounded-xl max-h-36 overflow-y-auto">
                {#each selectedCategory.file_extensions as ext}
                  <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-800 text-purple-300 text-xs font-mono border border-slate-700/60">
                    .{ext}
                    <button on:click={() => removeExtension(ext)} class="text-slate-400 hover:text-red-400 font-bold ml-1">
                      ×
                    </button>
                  </span>
                {/each}
              </div>

              <!-- Add extension -->
              <div class="flex gap-2">
                <input
                  type="text"
                  bind:value={newExtension}
                  on:keydown={(e) => e.key === 'Enter' && addExtension()}
                  placeholder="Thêm đuôi file (ví dụ: flac, webm)"
                  class="flex-1 bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-1.5 text-xs text-white focus:outline-none focus:border-purple-500 font-mono"
                />
                <button
                  on:click={addExtension}
                  class="px-4 py-1.5 rounded-xl bg-purple-600/30 hover:bg-purple-600 text-purple-200 text-xs font-semibold transition border border-purple-500/40"
                >
                  + Thêm
                </button>
              </div>
            </div>
          {:else}
            <div class="text-center text-slate-500 py-12 text-sm">Chọn một danh mục để chỉnh sửa</div>
          {/if}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 bg-slate-950/80 border-t border-slate-800 flex items-center justify-between">
        {#if saveSuccess}
          <span class="text-xs text-emerald-400 font-medium animate-pulse">✓ Đã lưu danh mục thành công!</span>
        {:else}
          <span class="text-xs text-slate-500">Quy tắc tự động kích hoạt khi nhận link mới</span>
        {/if}
        <div class="flex gap-2">
          <button
            on:click={onClose}
            class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            Đóng
          </button>
          <button
            on:click={saveCategories}
            disabled={isSaving}
            class="px-5 py-2 rounded-xl text-xs font-bold bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white shadow-lg shadow-purple-500/20 transition disabled:opacity-50"
          >
            {isSaving ? "Đang lưu..." : "Lưu Thay Đổi"}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
