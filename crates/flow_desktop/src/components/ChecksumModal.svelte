<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ShieldCheck, CheckCircle2, XCircle, Copy, Check, X, RefreshCw } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  export let filePath = "";
  export let fileName = "";

  const dispatch = createEventDispatcher();

  let selectedAlgo: "sha256" | "sha1" | "md5" = "sha256";
  let targetHash = "";
  let calculatedHash = "";
  let isCalculating = false;
  let copied = false;

  $: if (show && filePath) {
    calculateHash();
  }

  async function calculateHash() {
    if (!filePath) return;
    isCalculating = true;
    calculatedHash = "";
    try {
      calculatedHash = await invoke("calculate_file_checksum", {
        path: filePath,
        algorithm: selectedAlgo,
      });
    } catch (e) {
      calculatedHash = "Lỗi tính mã băm: " + e;
    } finally {
      isCalculating = false;
    }
  }

  $: isMatched =
    targetHash.trim() &&
    calculatedHash &&
    !isCalculating &&
    targetHash.trim().toLowerCase() === calculatedHash.toLowerCase();

  $: isMismatch =
    targetHash.trim() &&
    calculatedHash &&
    !isCalculating &&
    targetHash.trim().toLowerCase() !== calculatedHash.toLowerCase();

  function copyHash() {
    if (!calculatedHash) return;
    navigator.clipboard.writeText(calculatedHash);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function closeModal() {
    show = false;
    dispatch("close");
  }
</script>

{#if show}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div class="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/95 shadow-2xl flex flex-col overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2.5">
          <div class="h-8 w-8 rounded-lg bg-emerald-600/20 border border-emerald-500/30 flex items-center justify-center text-emerald-400">
            <ShieldCheck class="h-4 w-4" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Kiểm Tra Mã Băm (Checksum)</h2>
            <p class="text-xs text-slate-400 truncate max-w-xs">{fileName || filePath}</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-5 text-sm">
        <!-- Thuật toán -->
        <div>
          <label class="text-xs font-semibold text-slate-300 mb-2 block">Thuật toán băm:</label>
          <div class="grid grid-cols-3 gap-2">
            {#each ["sha256", "sha1", "md5"] as algo}
              <button
                type="button"
                on:click={() => {
                  selectedAlgo = algo as any;
                  calculateHash();
                }}
                class="py-2 px-3 rounded-xl border text-xs font-mono font-semibold transition-all {selectedAlgo === algo
                  ? 'border-emerald-500 bg-emerald-500/10 text-emerald-300'
                  : 'border-slate-800 bg-slate-950 text-slate-400 hover:border-slate-700'}"
              >
                {algo.toUpperCase()}
              </button>
            {/each}
          </div>
        </div>

        <!-- Mã băm thực tế -->
        <div class="space-y-1.5">
          <div class="flex justify-between items-center text-xs">
            <span class="font-semibold text-slate-300">Mã băm thực tế của file:</span>
            <button
              on:click={calculateHash}
              disabled={isCalculating}
              class="text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-mono text-[11px]"
            >
              <RefreshCw class="h-3 w-3 {isCalculating ? 'animate-spin' : ''}" /> Tính lại
            </button>
          </div>
          <div class="relative">
            <input
              type="text"
              readonly
              value={isCalculating ? "Đang tính toán mã băm..." : calculatedHash}
              class="w-full pl-3.5 pr-10 py-2.5 rounded-xl bg-slate-950 border border-slate-800 text-xs font-mono text-emerald-400 focus:outline-none select-all"
            />
            {#if calculatedHash && !isCalculating}
              <button
                on:click={copyHash}
                title="Sao chép"
                class="absolute right-2.5 top-1/2 -translate-y-1/2 p-1 text-slate-400 hover:text-white"
              >
                {#if copied}
                  <Check class="h-4 w-4 text-emerald-400" />
                {:else}
                  <Copy class="h-4 w-4" />
                {/if}
              </button>
            {/if}
          </div>
        </div>

        <!-- Ô đối chiếu mã băm mong muốn -->
        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-slate-300 block" for="checksum-target-input">
            Nhập mã băm mẫu từ tác giả để đối chiếu:
          </label>
          <input
            id="checksum-target-input"
            type="text"
            bind:value={targetHash}
            placeholder="Dán mã SHA-256 / MD5 mong muốn vào đây..."
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 focus:border-indigo-500 text-xs font-mono text-white placeholder:text-slate-600 focus:outline-none"
          />
        </div>

        <!-- Kết quả so khớp -->
        {#if isMatched}
          <div class="p-3.5 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-center space-x-3 text-emerald-300 animate-in fade-in">
            <CheckCircle2 class="h-5 w-5 flex-shrink-0 text-emerald-400" />
            <div class="text-xs">
              <strong class="font-bold block">Khớp 100% — File Toàn Vẹn!</strong>
              <span>Mã băm hoàn toàn trùng khớp với nguồn gốc phát hành.</span>
            </div>
          </div>
        {:else if isMismatch}
          <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 flex items-center space-x-3 text-rose-300 animate-in fade-in">
            <XCircle class="h-5 w-5 flex-shrink-0 text-rose-400" />
            <div class="text-xs">
              <strong class="font-bold block">Không Khớp — Cảnh Báo File Bị Thay Đổi!</strong>
              <span>Mã băm của file không khớp với chuỗi đối chiếu.</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/60 flex justify-end">
        <button
          on:click={closeModal}
          class="px-5 py-2 rounded-xl bg-slate-800 hover:bg-slate-700 text-white text-xs font-semibold transition-colors"
        >
          Xong
        </button>
      </div>
    </div>
  </div>
{/if}
