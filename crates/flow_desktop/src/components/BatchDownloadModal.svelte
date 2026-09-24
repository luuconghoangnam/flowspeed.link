<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ListPlus, X, Play, Info } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  const dispatch = createEventDispatcher();

  let batchText = "";
  let threadCount = 8;
  let isSubmitting = false;

  $: parsedUrls = batchText
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.startsWith("http://") || l.startsWith("https://"));

  async function handleStartBatch() {
    if (parsedUrls.length === 0) return;
    isSubmitting = true;

    try {
      for (const url of parsedUrls) {
        await invoke("start_download", {
          url,
          saveFolder: null,
          threads: Number(threadCount) || 8,
        });
      }
      batchText = "";
      closeModal();
      dispatch("batch-added");
    } catch (e) {
      alert("Lỗi khi thêm danh sách tải: " + e);
    } finally {
      isSubmitting = false;
    }
  }

  function closeModal() {
    show = false;
    dispatch("close");
  }
</script>

{#if show}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
    <div class="w-full max-w-xl rounded-2xl border border-slate-800 bg-slate-900/95 shadow-2xl flex flex-col overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between">
        <div class="flex items-center space-x-2.5">
          <div class="h-8 w-8 rounded-lg bg-cyan-600/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <ListPlus class="h-4 w-4" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Tải Hàng Loạt (Batch Download)</h2>
            <p class="text-xs text-slate-400">Dán nhiều đường dẫn URL, mỗi dòng một link</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-4 text-sm">
        <div class="space-y-1.5">
          <div class="flex justify-between items-center text-xs">
            <label class="font-semibold text-slate-300" for="batch-urls-input">Danh sách URL cần tải:</label>
            <span class="text-cyan-400 font-mono">Đã nhận diện: {parsedUrls.length} links</span>
          </div>
          <textarea
            id="batch-urls-input"
            rows="6"
            bind:value={batchText}
            placeholder="https://example.com/file1.zip&#10;https://example.com/file2.zip&#10;https://example.com/file3.zip"
            class="w-full px-3.5 py-2.5 rounded-xl bg-slate-950 border border-slate-800 focus:border-cyan-500 text-xs font-mono text-white placeholder:text-slate-600 focus:outline-none resize-none"
          ></textarea>
        </div>

        <div class="flex items-center justify-between p-3 rounded-xl bg-slate-950/60 border border-slate-800/80 text-xs">
          <div class="flex items-center space-x-2 text-slate-400">
            <Info class="h-4 w-4 text-cyan-400" />
            <span>Số luồng tải cho mỗi file:</span>
          </div>
          <input
            type="number"
            min="1"
            max="32"
            bind:value={threadCount}
            class="w-20 px-2.5 py-1 rounded-lg bg-slate-900 border border-slate-700 text-white font-mono text-center"
          />
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/60 flex items-center justify-end space-x-3">
        <button
          on:click={closeModal}
          class="px-4 py-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 text-xs font-medium transition-all"
        >
          Hủy
        </button>
        <button
          on:click={handleStartBatch}
          disabled={parsedUrls.length === 0 || isSubmitting}
          class="flex items-center space-x-1.5 px-5 py-2 rounded-xl bg-gradient-to-r from-cyan-600 to-indigo-600 hover:from-cyan-500 hover:to-indigo-500 disabled:opacity-50 text-white text-xs font-semibold shadow-lg shadow-cyan-600/20 transition-all active:scale-95"
        >
          <Play class="h-3.5 w-3.5 fill-current" />
          <span>{isSubmitting ? "Đang xử lý..." : `Tải ngay ${parsedUrls.length} file`}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
