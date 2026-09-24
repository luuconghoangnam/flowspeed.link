<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface PerHostRule {
    id: string;
    domain_pattern: string;
    max_threads?: number | null;
    speed_limit_kbps?: number | null;
    custom_user_agent?: string | null;
    custom_headers: Record<string, string>;
  }

  export let isOpen = false;
  export let onClose = () => {};

  let rules: PerHostRule[] = [];
  let selectedRule: PerHostRule | null = null;
  let isSaving = false;
  let saveSuccess = false;

  async function loadSettings() {
    try {
      const res: any = await invoke("get_settings");
      if (res) {
        rules = res.per_host_rules || [];
        if (rules.length > 0) {
          selectedRule = rules[0];
        } else {
          selectedRule = null;
        }
      }
    } catch (e) {
      console.error(e);
    }
  }

  $: if (isOpen) {
    loadSettings();
  }

  function addRule() {
    const newRule: PerHostRule = {
      id: "rule_" + Date.now(),
      domain_pattern: "example.com",
      max_threads: 4,
      speed_limit_kbps: null,
      custom_user_agent: null,
      custom_headers: {},
    };
    rules = [...rules, newRule];
    selectedRule = newRule;
  }

  function deleteRule(id: string) {
    rules = rules.filter((r) => r.id !== id);
    if (selectedRule?.id === id) {
      selectedRule = rules.length > 0 ? rules[0] : null;
    }
  }

  async function saveRules() {
    isSaving = true;
    saveSuccess = false;
    try {
      const res: any = await invoke("get_settings");
      if (res) {
        res.per_host_rules = rules;
        await invoke("save_settings", { settings: res });
        saveSuccess = true;
        setTimeout(() => {
          saveSuccess = false;
          onClose();
        }, 800);
      }
    } catch (e) {
      console.error(e);
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
          <div class="w-9 h-9 rounded-xl bg-cyan-500/20 text-cyan-400 flex items-center justify-center text-lg border border-cyan-500/30">
            🌐
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Quy Tắc Theo Tên Miền (Per-Host Rules)</h2>
            <p class="text-xs text-slate-400">Giới hạn số luồng, tốc độ và User-Agent riêng cho từng Host</p>
          </div>
        </div>
        <button on:click={onClose} class="text-slate-400 hover:text-white p-2 rounded-lg hover:bg-slate-800 transition">
          ✕
        </button>
      </div>

      <!-- Body: 2 Columns -->
      <div class="flex-1 flex overflow-hidden">
        <!-- List -->
        <div class="w-1/3 border-r border-slate-800 p-3 space-y-1.5 overflow-y-auto bg-slate-950/30">
          <div class="flex items-center justify-between px-2 py-1">
            <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Danh sách Host</span>
            <button
              on:click={addRule}
              class="text-[11px] text-cyan-400 hover:text-cyan-300 font-bold bg-cyan-500/10 hover:bg-cyan-500/20 px-2 py-0.5 rounded-md border border-cyan-500/30 transition"
            >
              + Thêm
            </button>
          </div>
          {#each rules as r}
            <button
              on:click={() => (selectedRule = r)}
              class="w-full text-left px-3 py-2.5 rounded-xl text-xs font-medium transition flex items-center justify-between {selectedRule?.id === r.id ? 'bg-cyan-600/20 text-cyan-300 border border-cyan-500/40' : 'text-slate-300 hover:bg-slate-800/60'}"
            >
              <span class="truncate font-mono">{r.domain_pattern}</span>
              <span class="text-[10px] text-slate-500">{r.max_threads || 8}T</span>
            </button>
          {/each}
          {#if rules.length === 0}
            <div class="text-center text-slate-600 py-6 text-xs">Chưa có quy tắc host nào</div>
          {/if}
        </div>

        <!-- Detail -->
        <div class="flex-1 p-6 overflow-y-auto space-y-5 bg-slate-900">
          {#if selectedRule}
            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1">Mẫu Tên Miền / Domain Pattern</label>
              <input
                type="text"
                bind:value={selectedRule.domain_pattern}
                placeholder="vd: drive.google.com hoặc *.rapidgator.net"
                class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-cyan-500"
              />
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1">Số Luồng Tối Đa (Threads)</label>
                <input
                  type="number"
                  bind:value={selectedRule.max_threads}
                  min="1"
                  max="32"
                  class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-cyan-500"
                />
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1">Giới Hạn Tốc Độ (KB/s)</label>
                <input
                  type="number"
                  bind:value={selectedRule.speed_limit_kbps}
                  placeholder="Không giới hạn"
                  class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-cyan-500"
                />
              </div>
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1">Custom User-Agent (Tùy chọn)</label>
              <input
                type="text"
                bind:value={selectedRule.custom_user_agent}
                placeholder="Để trống để dùng User-Agent mặc định"
                class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-xs text-white font-mono focus:outline-none focus:border-cyan-500"
              />
            </div>

            <div class="pt-4 border-t border-slate-800">
              <button
                on:click={() => selectedRule && deleteRule(selectedRule.id)}
                class="text-xs text-red-400 hover:text-red-300 font-semibold transition"
              >
                🗑️ Xóa Quy Tắc Này
              </button>
            </div>
          {:else}
            <div class="text-center text-slate-500 py-12 text-sm">Chọn hoặc tạo một quy tắc host để cấu hình</div>
          {/if}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 bg-slate-950/80 border-t border-slate-800 flex items-center justify-between">
        {#if saveSuccess}
          <span class="text-xs text-emerald-400 font-medium animate-pulse">✓ Đã lưu quy tắc host thành công!</span>
        {:else}
          <span class="text-xs text-slate-500">Áp dụng tức thì cho các kết nối mới</span>
        {/if}
        <div class="flex gap-2">
          <button
            on:click={onClose}
            class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            Đóng
          </button>
          <button
            on:click={saveRules}
            disabled={isSaving}
            class="px-5 py-2 rounded-xl text-xs font-bold bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white shadow-lg shadow-cyan-500/20 transition disabled:opacity-50"
          >
            {isSaving ? "Đang lưu..." : "Lưu Cấu Hình"}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
