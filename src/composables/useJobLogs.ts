import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

export function useJobLogs(jobId: string, category: string) {
    const logs = ref('');
    let offset = 0;
    let pollInterval: ReturnType<typeof setInterval> | null = null;

    const fetchLogs = async () => {
        try {
            // Define the expected shape of the Rust response
            const result = await invoke<{ logs: string; new_offset: number }>('get_job_logs', { 
                jobId,
                category,
                offset,
            });
            
            if (result.logs && result.logs.length > 0) {
                // 1. Escape HTML first (Order matters!)
                let safeText = result.logs
                    .replaceAll("&", "&amp;")
                    .replaceAll("<", "&lt;")
                    .replaceAll(">", "&gt;");

                // 2. Pure string find-and-replace for the colors
                safeText = safeText.replaceAll("[INFO]", '<span class="log-info">[INFO]</span>');
                safeText = safeText.replaceAll("[DEBUG]", '<span class="log-debug">[DEBUG]</span>');
                safeText = safeText.replaceAll("[WARN]", '<span class="log-warn">[WARN]</span>');
                safeText = safeText.replaceAll("[ERROR]", '<span class="log-error">[ERROR]</span>');
                
                safeText = safeText.replaceAll("[DOWNLOAD]", '<span class="log-tag">[DOWNLOAD]</span>');
                safeText = safeText.replaceAll("[MERGE]", '<span class="log-tag">[MERGE]</span>');

                logs.value += safeText;
                offset = result.new_offset;
            }
        } catch (e) {
            console.error(`Log error for ${jobId}:`, e);
        }
    };

    onMounted(() => {
        fetchLogs();
        pollInterval = setInterval(fetchLogs, 1000);
    });

    onUnmounted(() => {
        if (pollInterval) clearInterval(pollInterval);
    });

    return {
        logs
    };
}