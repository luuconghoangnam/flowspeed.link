<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import {
    Settings,
    Folder,
    Cpu,
    Zap,
    Palette,
    Bell,
    Share2,
    Save,
    RotateCcw,
    X,
    ShieldAlert,
    FileText,
    Volume2,
    Monitor,
  } from "lucide-svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let show = false;
  const dispatch = createEventDispatcher();

  interface AppSettings {
    download_dir: string;
    default_threads: number;
    max_concurrent_downloads: number;
    max_retry_count: number;
    dynamic_part_creation: boolean;
    use_sparse_file_allocation: boolean;
    ignore_ssl_certificates: boolean;
    delete_partial_on_cancel: boolean;
    append_extension_to_incomplete: boolean;
    user_agent: string;

    speed_limit_kbps: number | null;
    use_average_speed: boolean;
    size_unit: string;
    speed_unit: string;

    theme: string;
    language: string;
    ui_scale: number;
    use_relative_datetime: boolean;
    use_system_tray: boolean;

    notification_enabled: boolean;
    notification_sound: boolean;

    auto_start: boolean;
    browser_integration_enabled: boolean;
    server_port: number;

    categories?: any[];
    per_host_rules?: any[];
    power_action?: any;
  }

  let activeTab: "download" | "speed" | "ui" | "notification" | "system" = "download";

  let settings: AppSettings = {
    download_dir: "",
    default_threads: 8,
    max_concurrent_downloads: 3,
    max_retry_count: 3,
    dynamic_part_creation: true,
    use_sparse_file_allocation: true,
    ignore_ssl_certificates: false,
    delete_partial_on_cancel: false,
    append_extension_to_incomplete: false,
    user_agent: "",

    speed_limit_kbps: null,
    use_average_speed: true,
    size_unit: "binary",
    speed_unit: "bytes",

    theme: "dark",
    language: "vi",
    ui_scale: 1.0,
    use_relative_datetime: true,
    use_system_tray: true,

    notification_enabled: true,
    notification_sound: true,

    auto_start: false,
    browser_integration_enabled: true,
    server_port: 15151,
  };

  let speedLimitInput = "";
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
        settings = { ...settings, ...data };
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
      ...settings,
      default_threads: 8,
      max_concurrent_downloads: 3,
      max_retry_count: 3,
      dynamic_part_creation: true,
      use_sparse_file_allocation: true,
      ignore_ssl_certificates: false,
      delete_partial_on_cancel: false,
      append_extension_to_incomplete: false,
      user_agent: "",

      speed_limit_kbps: null,
      use_average_speed: true,
      size_unit: "binary",
      speed_unit: "bytes",

      theme: "dark",
      language: "vi",
      ui_scale: 1.0,
      use_relative_datetime: true,
      use_system_tray: true,

      notification_enabled: true,
      notification_sound: true,

      auto_start: false,
      browser_integration_enabled: true,
      server_port: 15151,
    };
    speedLimitInput = "";
  }

  function closeModal() {
    show = false;
    dispatch("close");
  }
</script>

{#if show}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-md p-4 animate-fade-in">
    <div class="w-full max-w-3xl rounded-2xl border border-slate-700/80 bg-slate-900 shadow-2xl flex flex-col max-h-[88vh] overflow-hidden">
      <!-- Modal Header -->
      <div class="px-6 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/60">
        <div class="flex items-center space-x-3">
          <div class="h-9 w-9 rounded-xl bg-indigo-600/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
            <Settings class="h-5 w-5" />
          </div>
          <div>
            <h2 class="text-base font-bold text-white tracking-wide">Cài Đặt Hệ Thống (Settings)</h2>
            <p class="text-xs text-slate-400">Tùy chỉnh toàn bộ thông số hoạt động của Flow Speed Link</p>
          </div>
        </div>
        <button on:click={closeModal} class="p-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition">
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- Navigation Tabs -->
      <div class="flex border-b border-slate-800 bg-slate-950/40 px-6 gap-2 text-xs font-semibold overflow-x-auto">
        <button
          on:click={() => (activeTab = "download")}
          class="py-3 px-3.5 border-b-2 transition flex items-center gap-2 {activeTab === 'download' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Folder class="h-4 w-4" /> Tải về & Mạng
        </button>
        <button
          on:click={() => (activeTab = "speed")}
          class="py-3 px-3.5 border-b-2 transition flex items-center gap-2 {activeTab === 'speed' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Zap class="h-4 w-4" /> Băng thông & Đơn vị
        </button>
        <button
          on:click={() => (activeTab = "ui")}
          class="py-3 px-3.5 border-b-2 transition flex items-center gap-2 {activeTab === 'ui' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Palette class="h-4 w-4" /> Giao diện & Hiển thị
        </button>
        <button
          on:click={() => (activeTab = "notification")}
          class="py-3 px-3.5 border-b-2 transition flex items-center gap-2 {activeTab === 'notification' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Bell class="h-4 w-4" /> Thông báo & Âm thanh
        </button>
        <button
          on:click={() => (activeTab = "system")}
          class="py-3 px-3.5 border-b-2 transition flex items-center gap-2 {activeTab === 'system' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
        >
          <Share2 class="h-4 w-4" /> Tích hợp & Hệ thống
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 space-y-5 overflow-y-auto flex-1 bg-slate-900 text-sm">
        <!-- TAB 1: Tải về & Mạng -->
        {#if activeTab === "download"}
          <div class="space-y-4">
            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1" for="download-dir">Thư mục tải mặc định</label>
              <input
                id="download-dir"
                type="text"
                bind:value={settings.download_dir}
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3.5 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div class="grid grid-cols-3 gap-4">
              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="default-threads">Số luồng tải (Threads)</label>
                <input
                  id="default-threads"
                  type="number"
                  min="1"
                  max="32"
                  bind:value={settings.default_threads}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="max-concurrent">Tải đồng thời tối đa</label>
                <input
                  id="max-concurrent"
                  type="number"
                  min="0"
                  max="20"
                  bind:value={settings.max_concurrent_downloads}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-indigo-500"
                />
                <span class="text-[10px] text-slate-500">0 = Không giới hạn</span>
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="max-retry">Số lần thử lại khi lỗi</label>
                <input
                  id="max-retry"
                  type="number"
                  min="0"
                  max="10"
                  bind:value={settings.max_retry_count}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1" for="custom-user-agent">Custom User-Agent</label>
              <input
                id="custom-user-agent"
                type="text"
                bind:value={settings.user_agent}
                placeholder="Để trống để dùng User-Agent chuẩn của Flow Speed"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3.5 py-2 text-xs font-mono text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div class="space-y-2.5 pt-2 border-t border-slate-800">
              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.dynamic_part_creation} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Tự động phân bổ luồng động (Dynamic Slicing)</div>
                  <div class="text-[11px] text-slate-400">File nhỏ tải 1 luồng, file lớn tự động tăng tới 32 luồng siêu tốc</div>
                </div>
              </label>

              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.use_sparse_file_allocation} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Cấp phát đĩa tức thì (Windows NTFS Sparse File)</div>
                  <div class="text-[11px] text-slate-400">Không cần chờ zero-fill đĩa cứng khi bắt đầu tải file dung lượng lớn</div>
                </div>
              </label>

              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.ignore_ssl_certificates} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Bỏ qua lỗi chứng chỉ SSL / HTTPS</div>
                  <div class="text-[11px] text-slate-400">Cho phép tải từ các server nội bộ hoặc chứng chỉ tự ký (Self-signed)</div>
                </div>
              </label>

              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.delete_partial_on_cancel} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Xóa tệp tạm khi người dùng bấm Hủy tải</div>
                  <div class="text-[11px] text-slate-400">Giải phóng dung lượng đĩa ngay lập tức khi hủy tác vụ</div>
                </div>
              </label>

              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.append_extension_to_incomplete} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Thêm phần mở rộng <code class="text-indigo-300 font-mono">.flow</code> khi đang tải dở</div>
                  <div class="text-[11px] text-slate-400">Đổi lại tên file gốc chỉ khi toàn bộ dữ liệu đã tải xong và xác minh</div>
                </div>
              </label>
            </div>
          </div>
        {/if}

        <!-- TAB 2: Băng thông & Đơn vị -->
        {#if activeTab === "speed"}
          <div class="space-y-4">
            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1" for="speed-limit">Giới hạn tốc độ tải tổng thể (KB/s)</label>
              <input
                id="speed-limit"
                type="number"
                bind:value={speedLimitInput}
                placeholder="Để trống = Không giới hạn tốc độ"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3.5 py-2 text-sm font-mono text-white focus:outline-none focus:border-indigo-500"
              />
              <span class="text-[11px] text-slate-500">Ví dụ: 10240 = 10 MB/s</span>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="size-unit">Đơn vị đo dung lượng file</label>
                <select
                  id="size-unit"
                  bind:value={settings.size_unit}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="binary">Binary Bytes (1024 KiB / MiB / GiB)</option>
                  <option value="decimal">Decimal Bytes (1000 KB / MB / GB)</option>
                </select>
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="speed-unit">Đơn vị đo tốc độ truyền tải</label>
                <select
                  id="speed-unit"
                  bind:value={settings.speed_unit}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="bytes">Byte / Giây (KB/s, MB/s)</option>
                  <option value="bits">Bit / Giây (Kbps, Mbps)</option>
                </select>
              </div>
            </div>

            <div class="pt-2 border-t border-slate-800">
              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.use_average_speed} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Làm mượt đo tốc độ bằng Sliding Window Average</div>
                  <div class="text-[11px] text-slate-400">Tính toán tốc độ trung bình 2.5 giây giúp số đo không bị nhảy giật cục</div>
                </div>
              </label>
            </div>
          </div>
        {/if}

        <!-- TAB 3: Giao diện & Hiển thị -->
        {#if activeTab === "ui"}
          <div class="space-y-4">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="theme-select">Giao diện màu chủ đạo</label>
                <select
                  id="theme-select"
                  bind:value={settings.theme}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="dark">Tối Hiện Đại (Dark Cyber)</option>
                  <option value="light">Sáng Thanh Lịch (Light Minimal)</option>
                  <option value="cyberpunk">Cyberpunk Neon</option>
                </select>
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-300 mb-1" for="lang-select">Ngôn ngữ ứng dụng</label>
                <select
                  id="lang-select"
                  bind:value={settings.language}
                  class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  <option value="vi">Tiếng Việt (Mặc định)</option>
                  <option value="en">English</option>
                </select>
              </div>
            </div>

            <div class="space-y-2.5 pt-2 border-t border-slate-800">
              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.use_relative_datetime} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Hiển thị mốc thời gian tương đối</div>
                  <div class="text-[11px] text-slate-400">Ví dụ: "2 phút trước", "hôm qua" thay vì ngày giờ cố định</div>
                </div>
              </label>

              <label class="flex items-center space-x-3 cursor-pointer">
                <input type="checkbox" bind:checked={settings.use_system_tray} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
                <div>
                  <div class="text-xs font-semibold text-white">Thu nhỏ xuống Khay hệ thống (System Tray)</div>
                  <div class="text-[11px] text-slate-400">Ứng dụng tiếp tục tải ngầm khi bấm nút Đóng/Ẩn</div>
                </div>
              </label>
            </div>
          </div>
        {/if}

        <!-- TAB 4: Thông báo & Âm thanh -->
        {#if activeTab === "notification"}
          <div class="space-y-4">
            <label class="flex items-center space-x-3 cursor-pointer">
              <input type="checkbox" bind:checked={settings.notification_enabled} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
              <div>
                <div class="text-xs font-semibold text-white">Bật thông báo Toast hệ thống Windows</div>
                <div class="text-[11px] text-slate-400">Hiển thị popup thông báo khi có file tải xong hoặc gặp lỗi</div>
              </div>
            </label>

            <label class="flex items-center space-x-3 cursor-pointer">
              <input type="checkbox" bind:checked={settings.notification_sound} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
              <div>
                <div class="text-xs font-semibold text-white">Phát âm thanh chuông khi hoàn tất</div>
                <div class="text-[11px] text-slate-400">Phát tín hiệu âm thanh báo hiệu tải xong toàn bộ danh sách</div>
              </div>
            </label>
          </div>
        {/if}

        <!-- TAB 5: Tích hợp & Hệ thống -->
        {#if activeTab === "system"}
          <div class="space-y-4">
            <label class="flex items-center space-x-3 cursor-pointer">
              <input type="checkbox" bind:checked={settings.auto_start} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
              <div>
                <div class="text-xs font-semibold text-white">Khởi động cùng hệ điều hành Windows (Autostart)</div>
                <div class="text-[11px] text-slate-400">Tự động kích hoạt Flow Speed Link ngay khi đăng nhập máy tính</div>
              </div>
            </label>

            <label class="flex items-center space-x-3 cursor-pointer">
              <input type="checkbox" bind:checked={settings.browser_integration_enabled} class="w-4 h-4 rounded text-indigo-600 focus:ring-0 bg-slate-950 border-slate-700" />
              <div>
                <div class="text-xs font-semibold text-white">Bật máy chủ bắt link Extension (Chrome / Firefox)</div>
                <div class="text-[11px] text-slate-400">Nhận diện và bắt link tải trực tiếp từ trình duyệt web</div>
              </div>
            </label>

            <div>
              <label class="block text-xs font-semibold text-slate-300 mb-1" for="server-port">Cổng kết nối Extension REST API (Port)</label>
              <input
                id="server-port"
                type="number"
                bind:value={settings.server_port}
                class="w-32 bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm font-mono text-white focus:outline-none focus:border-indigo-500"
              />
              <span class="text-[11px] text-slate-500 ml-2">Mặc định: 15151 (Khớp với Extension manifest)</span>
            </div>
          </div>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 border-t border-slate-800 bg-slate-950/80 flex items-center justify-between">
        <button
          on:click={handleResetDefault}
          class="flex items-center space-x-1.5 px-3 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition"
        >
          <RotateCcw class="h-3.5 w-3.5" />
          <span>Khôi phục mặc định</span>
        </button>

        <div class="flex items-center space-x-3">
          {#if saveSuccessMessage}
            <span class="text-xs text-emerald-400 font-medium animate-pulse">✓ Đã lưu cài đặt thành công!</span>
          {/if}
          <button
            on:click={closeModal}
            class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            Hủy
          </button>
          <button
            on:click={handleSave}
            disabled={isSaving}
            class="flex items-center space-x-2 px-5 py-2 rounded-xl bg-gradient-to-r from-indigo-600 to-cyan-500 hover:from-indigo-500 hover:to-cyan-400 text-white text-xs font-bold shadow-lg shadow-indigo-600/30 transition disabled:opacity-50"
          >
            <Save class="h-4 w-4" />
            <span>{isSaving ? "Đang lưu..." : "Lưu Thay Đổi"}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
