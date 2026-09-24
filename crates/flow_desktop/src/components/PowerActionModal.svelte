<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  export let isOpen = false;
  export let isAlertOpen = false; // When trigger fires
  export let onClose = () => {};
  export let onCancelAlert = () => {};

  let selectedAction: "None" | "Shutdown" | "Sleep" | "Hibernate" | "ExitApp" = "None";
  let countdownSeconds = 30;
  let remainingSeconds = 30;
  let timerInterval: any = null;
  let isSaving = false;

  async function loadSettings() {
    try {
      const res: any = await invoke("get_settings");
      if (res && res.power_action) {
        selectedAction = res.power_action.action_type || "None";
        countdownSeconds = res.power_action.countdown_seconds || 30;
      }
    } catch (e) {
      console.error(e);
    }
  }

  $: if (isOpen) {
    loadSettings();
  }

  $: if (isAlertOpen) {
    remainingSeconds = countdownSeconds;
    if (timerInterval) clearInterval(timerInterval);
    timerInterval = setInterval(() => {
      if (remainingSeconds > 0) {
        remainingSeconds--;
      } else {
        clearInterval(timerInterval);
        executePowerAction();
      }
    }, 1000);
  } else {
    if (timerInterval) clearInterval(timerInterval);
  }

  function executePowerAction() {
    console.log("Executing power action:", selectedAction);
    if (selectedAction === "ExitApp") {
      window.close();
    }
  }

  async function saveAction() {
    isSaving = true;
    try {
      const res: any = await invoke("get_settings");
      if (res) {
        if (selectedAction === "None") {
          res.power_action = null;
        } else {
          res.power_action = {
            action_type: selectedAction,
            force: false,
            countdown_seconds: countdownSeconds,
            is_active: true,
          };
        }
        await invoke("save_settings", { settings: res });
      }
      onClose();
    } catch (e) {
      console.error(e);
    } finally {
      isSaving = false;
    }
  }
</script>

<!-- Settings Modal -->
{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-md p-4 animate-fade-in">
    <div class="bg-slate-900 border border-slate-700/80 rounded-2xl w-full max-w-md shadow-2xl overflow-hidden flex flex-col">
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-xl bg-amber-500/20 text-amber-400 flex items-center justify-center text-lg border border-amber-500/30">
            ⚡
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Hành Động Nguồn (Power Actions)</h2>
            <p class="text-xs text-slate-400">Tự động xử lý khi hoàn tất toàn bộ tiến trình tải</p>
          </div>
        </div>
        <button on:click={onClose} class="text-slate-400 hover:text-white p-2 rounded-lg hover:bg-slate-800 transition">
          ✕
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 space-y-4">
        <div class="space-y-2">
          <label class="text-xs font-semibold text-slate-300">Chọn hành động khi tải xong:</label>
          <div class="grid grid-cols-2 gap-2">
            <button
              on:click={() => (selectedAction = "None")}
              class="p-3 rounded-xl border text-xs font-medium text-left transition {selectedAction === 'None' ? 'border-amber-500 bg-amber-500/20 text-amber-300' : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'}"
            >
              <div class="font-bold">❌ Không làm gì</div>
              <div class="text-[10px] text-slate-500">Giữ máy hoạt động bình thường</div>
            </button>

            <button
              on:click={() => (selectedAction = "Shutdown")}
              class="p-3 rounded-xl border text-xs font-medium text-left transition {selectedAction === 'Shutdown' ? 'border-amber-500 bg-amber-500/20 text-amber-300' : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'}"
            >
              <div class="font-bold">🛑 Tắt máy (Shutdown)</div>
              <div class="text-[10px] text-slate-500">Tắt nguồn máy tính an toàn</div>
            </button>

            <button
              on:click={() => (selectedAction = "Sleep")}
              class="p-3 rounded-xl border text-xs font-medium text-left transition {selectedAction === 'Sleep' ? 'border-amber-500 bg-amber-500/20 text-amber-300' : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'}"
            >
              <div class="font-bold">🌙 Ngủ (Sleep)</div>
              <div class="text-[10px] text-slate-500">Đưa máy vào chế độ tiết kiệm điện</div>
            </button>

            <button
              on:click={() => (selectedAction = "ExitApp")}
              class="p-3 rounded-xl border text-xs font-medium text-left transition {selectedAction === 'ExitApp' ? 'border-amber-500 bg-amber-500/20 text-amber-300' : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'}"
            >
              <div class="font-bold">🚪 Thoát ứng dụng</div>
              <div class="text-[10px] text-slate-500">Đóng Flow Speed Link</div>
            </button>
          </div>
        </div>

        {#if selectedAction !== "None"}
          <div>
            <label class="block text-xs font-semibold text-slate-300 mb-1">Thời gian đếm ngược cảnh báo (giây)</label>
            <input
              type="number"
              bind:value={countdownSeconds}
              min="5"
              max="120"
              class="w-full bg-slate-950 border border-slate-700/80 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-amber-500"
            />
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-6 py-3.5 bg-slate-950/80 border-t border-slate-800 flex justify-end gap-2">
        <button
          on:click={onClose}
          class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          Hủy
        </button>
        <button
          on:click={saveAction}
          disabled={isSaving}
          class="px-5 py-2 rounded-xl text-xs font-bold bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 text-white shadow-lg shadow-amber-500/20 transition disabled:opacity-50"
        >
          Lưu Cấu Hình
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Live Countdown Alert Modal -->
{#if isAlertOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-lg p-4 animate-fade-in">
    <div class="bg-slate-900 border-2 border-amber-500/80 rounded-3xl w-full max-w-sm shadow-2xl overflow-hidden flex flex-col items-center p-6 text-center space-y-4 animate-pulse-slow">
      <div class="w-16 h-16 rounded-full bg-amber-500/20 text-amber-400 flex items-center justify-center text-3xl border border-amber-500/40">
        ⚠️
      </div>
      <div>
        <h3 class="text-lg font-bold text-white">Tất cả tệp đã tải xong!</h3>
        <p class="text-xs text-slate-400 mt-1">
          Máy tính sẽ tự động <strong>{selectedAction}</strong> sau:
        </p>
      </div>

      <div class="text-5xl font-black font-mono text-amber-400 tracking-wider">
        {remainingSeconds}s
      </div>

      <button
        on:click={onCancelAlert}
        class="w-full py-3 rounded-2xl bg-red-600 hover:bg-red-500 text-white font-bold text-sm shadow-lg shadow-red-600/30 transition transform hover:scale-[1.02]"
      >
        ✕ HỦY BỎ TẮT MÁY
      </button>
    </div>
  </div>
{/if}
