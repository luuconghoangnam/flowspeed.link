<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ListPlus, X, Play, Info, Sparkles } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  const dispatch = createEventDispatcher();

  let batchText = "";
  let patternUrl = "";
  let threadCount = 8;
  let isSubmitting = false;

  $: parsedUrls = batchText
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.startsWith("http://") || l.startsWith("https://"));

  function expandPattern() {
    if (!patternUrl.trim()) return;
    const url = patternUrl.trim();

    // Regex check for [01-20] or [1-100]
    const numMatch = url.match(/\[(\d+)-(\d+)\]/);
    if (numMatch) {
      const startStr = numMatch[1];
      const endStr = numMatch[2];
      const start = parseInt(startStr, 10);
      const end = parseInt(endStr, 10);
      const padLen = startStr.length;

      if (start <= end && end - start <= 500) {
        const generated: string[] = [];
        for (let i = start; i <= end; i++) {
          const numFormatted = String(i).padStart(padLen, "0");
          generated.push(url.replace(numMatch[0], numFormatted));
        }
        batchText = (batchText.trim() ? batchText.trim() + "\n" : "") + generated.join("\n");
        patternUrl = "";
        return;
      }
    }

    // Regex check for [a-z]
    const charMatch = url.match(/\[([a-zA-Z])-([a-zA-Z])\]/);
    if (charMatch) {
      const startChar = charMatch[1].charCodeAt(0);
      const endChar = charMatch[2].charCodeAt(0);
      if (startChar <= endChar && endChar - startChar <= 52) {
        const generated: string[] = [];
        for (let code = startChar; code <= endChar; code++) {
          generated.push(url.replace(charMatch[0], String.fromCharCode(code)));
        }
        batchText = (batchText.trim() ? batchText.trim() + "\n" : "") + generated.join("\n");
        patternUrl = "";
        return;
      }
    }

    alert("Mẫu không hợp lệ! Vui lòng dùng định dạng [01-20], [1-50] hoặc [a-z].");
  }

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
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4 animate-fade-in">
    <div class="w-full max-w-xl rounded-2xl border border-slate-700/80 bg-slate-900 shadow-2xl flex flex-col overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center space-x-2.5">
          <div class="h-8 w-8 rounded-lg bg-cyan-600/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <ListPlus class="h-4 w-4" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Tải Hàng Loạt (Batch Download)</h2>
            <p class="text-xs text-slate-400">Dán danh sách URL hoặc sinh link tự động theo mẫu</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition-colors">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-4 text-sm">
        <!-- Pattern Expander -->
        <div class="p-3.5 rounded-xl bg-slate-950/70 border border-cyan-500/30 space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-xs font-semibold text-cyan-300 flex items-center gap-1.5" for="pattern-input">
              <Sparkles class="h-3.5 w-3.5 text-cyan-400" />
              Sinh link tự động theo mẫu (Pattern):
            </label>
            <span class="text-[10px] text-slate-400 font-mono">vd: [01-20] hoặc [a-f]</span>
          </div>
          <div class="flex gap-2">
            <input
              id="pattern-input"
              type="text"
              bind:value={patternUrl}
              on:keydown={(e) => e.key === 'Enter' && expandPattern()}
              placeholder="https://example.com/ep_[01-12].mp4"
              class="flex-1 px-3 py-1.5 rounded-lg bg-slate-900 border border-slate-700/80 focus:border-cyan-500 text-xs font-mono text-white placeholder:text-slate-600 focus:outline-none"
            />
            <button
              type="button"
              on:click={expandPattern}
              class="px-3.5 py-1.5 rounded-lg bg-cyan-600/30 hover:bg-cyan-600 text-cyan-200 text-xs font-semibold border border-cyan-500/40 transition"
            >
              + Khai triển
            </button>
          </div>
        </div>

        <div class="space-y-1.5">
          <div class="flex justify-between items-center text-xs">
            <label class="font-semibold text-slate-300" for="batch-urls-input">Danh sách URL cần tải (mỗi dòng 1 link):</label>
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
          class="flex items-center space-x-2 px-5 py-2 rounded-xl bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white text-xs font-bold shadow-lg shadow-cyan-600/30 disabled:opacity-50 transition-all"
        >
          <Play class="h-4 w-4" />
          <span>{isSubmitting ? "Đang xử lý..." : `Bắt đầu tải ${parsedUrls.length} file`}</span>
        </button>
      </div>
    </div>
  </div>
{/if}
